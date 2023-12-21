#![allow(missing_docs)]
use std::{
    collections::HashMap, num::NonZeroUsize, path::PathBuf, process::ExitStatus, time::Duration,
};

use exitcode::ExitCode;
use futures::StreamExt;
#[cfg(feature = "enterprise")]
use futures_util::future::BoxFuture;
use once_cell::race::OnceNonZeroUsize;
use openssl::provider::Provider;
use std::time::Instant;
use tokio::{
    runtime::{self, Runtime},
    sync::mpsc,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use vector_core::usage_metrics::{start_publishing_metrics, UsageMetrics};

#[cfg(feature = "enterprise")]
use crate::config::enterprise::{
    attach_enterprise_components, report_configuration, EnterpriseError, EnterpriseMetadata,
    EnterpriseReporter,
};
#[cfg(not(feature = "enterprise-tests"))]
use crate::metrics;
#[cfg(feature = "api")]
use crate::{api, internal_events::ApiStarted};
use crate::{
    cli::{handle_config_errors, LogFormat, Opts, RootOpts},
    config::{self, Config, ConfigPath},
    heartbeat,
    internal_events::mezmo_config::{
        MezmoConfigCompile, MezmoConfigReload, MezmoConfigReloadSignalReceive,
    },
    mezmo,
    signal::{ShutdownError, SignalHandler, SignalPair, SignalRx, SignalTo},
    topology::{
        self, ReloadOutcome, RunningTopology, SharedTopologyController, TopologyController,
    },
    trace,
};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use tokio::runtime::Handle;

pub static WORKER_THREADS: OnceNonZeroUsize = OnceNonZeroUsize::new();

use crate::internal_events::{VectorQuit, VectorStarted, VectorStopped};

use tokio::sync::broadcast::error::RecvError;

pub struct ApplicationConfig {
    pub config_paths: Vec<config::ConfigPath>,
    pub topology: RunningTopology,
    pub graceful_crash_sender: mpsc::UnboundedSender<ShutdownError>,
    pub graceful_crash_receiver: mpsc::UnboundedReceiver<ShutdownError>,
    #[cfg(feature = "api")]
    pub api: config::api::Options,
    #[cfg(feature = "enterprise")]
    pub enterprise: Option<EnterpriseReporter<BoxFuture<'static, ()>>>,
    pub metrics_tx: mpsc::UnboundedSender<UsageMetrics>,
}

pub struct Application {
    pub require_healthy: Option<bool>,
    pub config: ApplicationConfig,
    pub signals: SignalPair,
    pub openssl_providers: Option<Vec<Provider>>,
}

impl ApplicationConfig {
    pub async fn from_opts(
        opts: &RootOpts,
        signal_handler: &mut SignalHandler,
    ) -> Result<Self, ExitCode> {
        let config_paths = opts.config_paths_with_formats();

        let graceful_shutdown_duration = (!opts.no_graceful_shutdown_limit)
            .then(|| Duration::from_secs(u64::from(opts.graceful_shutdown_limit_secs)));

        let config = load_configs(
            &config_paths,
            opts.watch_config,
            opts.require_healthy,
            graceful_shutdown_duration,
            signal_handler,
        )
        .await?;

        Self::from_config(config_paths, config).await
    }

    pub async fn from_config(
        config_paths: Vec<ConfigPath>,
        config: Config,
    ) -> Result<Self, ExitCode> {
        // This is ugly, but needed to allow `config` to be mutable for building the enterprise
        // features, but also avoid a "does not need to be mutable" warning when the enterprise
        // feature is not enabled.
        #[cfg(feature = "enterprise")]
        let mut config = config;
        #[cfg(feature = "enterprise")]
        let enterprise = build_enterprise(&mut config, config_paths.clone())?;

        let (metrics_tx, metrics_rx) = mpsc::unbounded_channel::<UsageMetrics>();
        start_publishing_metrics(metrics_rx)
            .await
            .map_err(|_| handle_config_errors(vec!["Usage metrics publishing error".into()]))?;

        let diff = config::ConfigDiff::initial(&config);
        let pieces =
            topology::build_or_log_errors(&config, &diff, Some(metrics_tx.clone()), HashMap::new())
                .await
                .ok_or(exitcode::CONFIG)?;

        #[cfg(feature = "api")]
        let api = config.api;

        let result = topology::start_validated(config, diff, pieces).await;
        let (topology, (graceful_crash_sender, graceful_crash_receiver)) =
            result.ok_or(exitcode::CONFIG)?;

        Ok(Self {
            config_paths,
            topology,
            graceful_crash_sender,
            graceful_crash_receiver,
            #[cfg(feature = "api")]
            api,
            #[cfg(feature = "enterprise")]
            enterprise,
            metrics_tx,
        })
    }

    /// Configure the API server, if applicable
    #[cfg(feature = "api")]
    pub fn setup_api(&self, handle: &Handle) -> Option<api::Server> {
        if self.api.enabled {
            match api::Server::start(
                self.topology.config(),
                self.topology.watch(),
                std::sync::Arc::clone(&self.topology.running),
                handle,
            ) {
                Ok(api_server) => {
                    emit!(ApiStarted {
                        addr: self.api.address.unwrap(),
                        playground: self.api.playground
                    });

                    Some(api_server)
                }
                Err(error) => {
                    let error = error.to_string();
                    error!("An error occurred that Vector couldn't handle: {}.", error);
                    _ = self
                        .graceful_crash_sender
                        .send(ShutdownError::ApiFailed { error });
                    None
                }
            }
        } else {
            info!(message="API is disabled, enable by setting `api.enabled` to `true` and use commands like `vector top`.");
            None
        }
    }
}

impl Application {
    pub fn run() -> ExitStatus {
        let (runtime, app) = Self::prepare_start().unwrap_or_else(|code| std::process::exit(code));

        runtime.block_on(app.run())
    }

    pub fn prepare_start() -> Result<(Runtime, StartedApplication), ExitCode> {
        Self::prepare()
            .and_then(|(runtime, app)| app.start(runtime.handle()).map(|app| (runtime, app)))
    }

    pub fn prepare() -> Result<(Runtime, Self), ExitCode> {
        let opts = Opts::get_matches().map_err(|error| {
            // Printing to stdout/err can itself fail; ignore it.
            _ = error.print();
            exitcode::USAGE
        })?;

        Self::prepare_from_opts(opts)
    }

    pub fn prepare_from_opts(opts: Opts) -> Result<(Runtime, Self), ExitCode> {
        init_global();

        let color = opts.root.color.use_color();

        init_logging(
            color,
            opts.root.log_format,
            opts.log_level(),
            opts.root.internal_log_rate_limit,
        );
        mezmo::user_trace::init(opts.root.user_log_rate_limit);

        let openssl_providers = opts
            .root
            .openssl_legacy_provider
            .then(load_openssl_legacy_providers)
            .transpose()?;

        let runtime = build_runtime(opts.root.threads, "vector-worker")?;

        // Signal handler for OS and provider messages.
        let mut signals = SignalPair::new(&runtime);

        if let Some(sub_command) = &opts.sub_command {
            return Err(runtime.block_on(sub_command.execute(signals, color)));
        }

        let config = runtime.block_on(ApplicationConfig::from_opts(
            &opts.root,
            &mut signals.handler,
        ))?;

        #[cfg(feature = "api-client")]
        start_remote_task_execution(&runtime, &config)?;

        Ok((
            runtime,
            Self {
                require_healthy: opts.root.require_healthy,
                config,
                signals,
                openssl_providers,
            },
        ))
    }

    pub fn start(self, handle: &Handle) -> Result<StartedApplication, ExitCode> {
        // Any internal_logs sources will have grabbed a copy of the
        // early buffer by this point and set up a subscriber.
        crate::trace::stop_early_buffering();

        emit!(VectorStarted);
        handle.spawn(heartbeat::heartbeat());

        let Self {
            require_healthy,
            config,
            signals,
            openssl_providers,
        } = self;

        let topology_controller = SharedTopologyController::new(TopologyController {
            #[cfg(feature = "api")]
            api_server: config.setup_api(handle),
            topology: config.topology,
            config_paths: config.config_paths.clone(),
            require_healthy,
            #[cfg(feature = "enterprise")]
            enterprise_reporter: config.enterprise,
        });

        Ok(StartedApplication {
            config_paths: config.config_paths,
            graceful_crash_receiver: config.graceful_crash_receiver,
            signals,
            topology_controller,
            metrics_tx: config.metrics_tx,
            openssl_providers,
        })
    }
}

#[cfg(feature = "api-client")]
fn start_remote_task_execution(
    runtime: &Runtime,
    _config: &ApplicationConfig,
) -> Result<(), ExitCode> {
    use std::env;

    #[cfg(feature = "api")]
    let api_config = _config.api;
    #[cfg(not(feature = "api"))]
    let api_config: config::api::Options = Default::default();

    let auth_token = env::var("MEZMO_LOCAL_DEPLOY_AUTH_TOKEN").ok();
    if let Some(auth_token) = auth_token {
        let get_endpoint_url = env::var("MEZMO_TASKS_FETCH_ENDPOINT_URL").ok();
        let post_endpoint_url = env::var("MEZMO_TASKS_POST_ENDPOINT_URL").ok();
        match (get_endpoint_url, post_endpoint_url) {
            (Some(get_endpoint_url), Some(post_endpoint_url)) => {
                if !api_config.enabled {
                    error!("API is disabled");
                    return Err(exitcode::USAGE);
                }

                runtime.spawn(async move {
                    mezmo::remote_task_execution::start_polling_for_tasks(
                        api_config,
                        auth_token,
                        get_endpoint_url,
                        post_endpoint_url,
                    )
                    .await;
                });
            }
            (_, _) => {
                error!("Mezmo tasks endpoints not set");
                return Err(exitcode::USAGE);
            }
        }
    }

    Ok(())
}

pub struct StartedApplication {
    pub config_paths: Vec<ConfigPath>,
    pub graceful_crash_receiver: mpsc::UnboundedReceiver<ShutdownError>,
    pub signals: SignalPair,
    pub topology_controller: SharedTopologyController,
    pub metrics_tx: mpsc::UnboundedSender<UsageMetrics>,
    pub openssl_providers: Option<Vec<Provider>>,
}

impl StartedApplication {
    pub async fn run(self) -> ExitStatus {
        self.main().await.shutdown().await
    }

    pub async fn main(self) -> FinishedApplication {
        let Self {
            config_paths,
            graceful_crash_receiver,
            signals,
            topology_controller,
            metrics_tx,
            openssl_providers,
        } = self;

        let mut graceful_crash = UnboundedReceiverStream::new(graceful_crash_receiver);

        let mut signal_handler = signals.handler;
        let mut signal_rx = signals.receiver;

        let signal = loop {
            tokio::select! {
                signal = signal_rx.recv() => if let Some(signal) = handle_signal(
                    signal,
                    &topology_controller,
                    &config_paths,
                    &mut signal_handler,
                    &metrics_tx,
                ).await {
                    break signal;
                },
                // Trigger graceful shutdown if a component crashed, or all sources have ended.
                error = graceful_crash.next() => break SignalTo::Shutdown(error),
                _ = TopologyController::sources_finished(topology_controller.clone()) => {
                    info!("All sources have finished.");
                    break SignalTo::Shutdown(None)
                } ,
                else => unreachable!("Signal streams never end"),
            }
        };

        FinishedApplication {
            signal,
            signal_rx,
            topology_controller,
            openssl_providers,
        }
    }
}

async fn handle_signal(
    signal: Result<SignalTo, RecvError>,
    topology_controller: &SharedTopologyController,
    config_paths: &[ConfigPath],
    signal_handler: &mut SignalHandler,
    metrics_tx: &mpsc::UnboundedSender<UsageMetrics>,
) -> Option<SignalTo> {
    match signal {
        Ok(SignalTo::ReloadFromConfigBuilder(config_builder)) => {
            emit!(MezmoConfigReloadSignalReceive {});
            let start = Instant::now();
            let mut topology_controller = topology_controller.lock().await;

            // We use build_no_validation() to speed up building
            // Configs were fully validated when generated and better errors
            // won't help us much at this point (as it will blow up anyway)
            let new_config = config_builder
                .build_no_validation()
                .map_err(handle_config_errors)
                .ok();

            emit!(MezmoConfigCompile {
                elapsed: Instant::now() - start
            });

            let mut reload_outcome = ReloadOutcome::NoConfig;
            let reload_future =
                topology_controller.reload_with_metrics(new_config, Some(metrics_tx.clone()));

            tokio::pin!(reload_future);

            // LOG-17772: Set a maximum amount of time to wait for configuration reloading before
            // triggering corrective action. By default, this will wait for 150 seconds / 2.5 minutes.
            let config_reload_max_sec = std::env::var("CONFIG_RELOAD_MAX_SEC").unwrap_or_else(|_| {
                warn!("couldn't read value for CONFIG_RELOAD_MAX_SEC env var, default will be used");
                "150".to_owned()
            });
            let config_reload_max_sec = config_reload_max_sec.parse::<usize>().unwrap_or_else(|_| {
                warn!("failed to parse CONFIG_RELOAD_MAX_SEC value {config_reload_max_sec}, default will be used");
                150
            });
            let mut reload_future_done = false;
            for i in 1..=config_reload_max_sec {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        info!("Waiting for topology to be reloaded after {i} secs")
                    },
                    outcome = &mut reload_future => {
                        reload_outcome = outcome;
                        reload_future_done = true;
                        break;
                    }
                }
            }

            let elapsed = Instant::now() - start;

            // LOG-17772: If the config reload future doesn't resolve in the allotted
            // time, then crash the vector process and allow k8s to respawn the process
            // in order to get config reloading to work again. Note that panic! doesn't
            // work to terminate the process since the higher level code traps the panic
            // and prevents termination.
            if !reload_future_done {
                emit!(MezmoConfigReload {
                    elapsed,
                    success: false
                });
                error!("New topology reload future failed to resolved within the limit.");
                std::process::abort();
            }

            match reload_outcome {
                ReloadOutcome::NoConfig => {
                    emit!(MezmoConfigReload {
                        elapsed,
                        success: false
                    });
                    warn!("Config reload resulted in no config");
                }
                ReloadOutcome::MissingApiKey => {
                    emit!(MezmoConfigReload {
                        elapsed,
                        success: false
                    });
                    warn!("Config reload missing API key");
                }
                ReloadOutcome::Success => {
                    emit!(MezmoConfigReload {
                        elapsed,
                        success: true
                    });
                    info!("Config reload succeeded, took {:?}", elapsed);
                }
                ReloadOutcome::RolledBack => {
                    emit!(MezmoConfigReload {
                        elapsed,
                        success: false
                    });
                    warn!("Config reload rolled back");
                }
                ReloadOutcome::FatalError(error) => {
                    emit!(MezmoConfigReload {
                        elapsed,
                        success: false
                    });
                    error!("Config reload fatal error");
                    return Some(SignalTo::Shutdown(Some(error)));
                }
            }
            None
        }
        Ok(SignalTo::ReloadFromDisk) => {
            let mut topology_controller = topology_controller.lock().await;

            // Reload paths
            if let Some(paths) = config::process_paths(config_paths) {
                topology_controller.config_paths = paths;
            }

            // Reload config
            let new_config = config::load_from_paths_with_provider_and_secrets(
                &topology_controller.config_paths,
                signal_handler,
            )
            .await
            .map_err(handle_config_errors)
            .ok();

            if let ReloadOutcome::FatalError(error) = topology_controller.reload(new_config).await {
                return Some(SignalTo::Shutdown(Some(error)));
            }
            None
        }
        Err(RecvError::Lagged(amt)) => {
            warn!("Overflow, dropped {} signals.", amt);
            None
        }
        Err(RecvError::Closed) => Some(SignalTo::Shutdown(None)),
        Ok(signal) => Some(signal),
    }
}

pub struct FinishedApplication {
    pub signal: SignalTo,
    pub signal_rx: SignalRx,
    pub topology_controller: SharedTopologyController,
    pub openssl_providers: Option<Vec<Provider>>,
}

impl FinishedApplication {
    pub async fn shutdown(self) -> ExitStatus {
        let FinishedApplication {
            signal,
            signal_rx,
            topology_controller,
            openssl_providers,
        } = self;

        // At this point, we'll have the only reference to the shared topology controller and can
        // safely remove it from the wrapper to shut down the topology.
        let topology_controller = topology_controller
            .try_into_inner()
            .expect("fail to unwrap topology controller")
            .into_inner();

        let status = match signal {
            SignalTo::Shutdown(_) => Self::stop(topology_controller, signal_rx).await,
            SignalTo::Quit => Self::quit(),
            _ => unreachable!(),
        };
        drop(openssl_providers);
        status
    }

    async fn stop(topology_controller: TopologyController, mut signal_rx: SignalRx) -> ExitStatus {
        emit!(VectorStopped);
        tokio::select! {
            _ = topology_controller.stop() => ExitStatus::from_raw({
                #[cfg(windows)]
                {
                    exitcode::OK as u32
                }
                #[cfg(unix)]
                exitcode::OK
            }), // Graceful shutdown finished
            _ = signal_rx.recv() => Self::quit(),
        }
    }

    fn quit() -> ExitStatus {
        // It is highly unlikely that this event will exit from topology.
        emit!(VectorQuit);
        ExitStatus::from_raw({
            #[cfg(windows)]
            {
                exitcode::UNAVAILABLE as u32
            }
            #[cfg(unix)]
            exitcode::OK
        })
    }
}

pub fn init_global() {
    openssl_probe::init_ssl_cert_env_vars();

    #[cfg(not(feature = "enterprise-tests"))]
    metrics::init_global().expect("metrics initialization failed");
}

fn get_log_levels(default: &str) -> String {
    std::env::var("VECTOR_LOG")
        .or_else(|_| {
            std::env::var("LOG").map(|log| {
                warn!(
                    message =
                        "DEPRECATED: Use of $LOG is deprecated. Please use $VECTOR_LOG instead."
                );
                log
            })
        })
        .unwrap_or_else(|_| match default {
            "off" => "off".to_owned(),
            level => [
                format!("vector={}", level),
                format!("codec={}", level),
                format!("vrl={}", level),
                format!("file_source={}", level),
                format!("tower_limit={}", level),
                format!("rdkafka={}", level),
                format!("buffers={}", level),
                format!("lapin={}", level),
                format!("kube={}", level),
            ]
            .join(","),
        })
}

pub fn build_runtime(threads: Option<usize>, thread_name: &str) -> Result<Runtime, ExitCode> {
    let mut rt_builder = runtime::Builder::new_multi_thread();
    rt_builder.max_blocking_threads(20_000);
    rt_builder.enable_all().thread_name(thread_name);

    if let Some(threads) = threads {
        if threads < 1 {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("The `threads` argument must be greater or equal to 1.");
            }
            return Err(exitcode::CONFIG);
        } else {
            WORKER_THREADS
                .set(NonZeroUsize::new(threads).expect("already checked"))
                .expect("double thread initialization");
            rt_builder.worker_threads(threads);
        }
    }

    Ok(rt_builder.build().expect("Unable to create async runtime"))
}

pub async fn load_configs(
    config_paths: &[ConfigPath],
    watch_config: bool,
    require_healthy: Option<bool>,
    graceful_shutdown_duration: Option<Duration>,
    signal_handler: &mut SignalHandler,
) -> Result<Config, ExitCode> {
    let config_paths = config::process_paths(config_paths).ok_or(exitcode::CONFIG)?;

    if watch_config {
        // Start listening for config changes immediately.
        config::watcher::spawn_thread(config_paths.iter().map(Into::into), None).map_err(
            |error| {
                error!(message = "Unable to start config watcher.", %error);
                exitcode::CONFIG
            },
        )?;
    }

    info!(
        message = "Loading configs.",
        paths = ?config_paths.iter().map(<&PathBuf>::from).collect::<Vec<_>>()
    );

    // config::init_log_schema should be called before initializing sources.
    #[cfg(not(feature = "enterprise-tests"))]
    config::init_log_schema(&config_paths, true).map_err(handle_config_errors)?;

    let mut config =
        config::load_from_paths_with_provider_and_secrets(&config_paths, signal_handler)
            .await
            .map_err(handle_config_errors)?;

    config::init_telemetry(config.global.telemetry.clone(), true);

    if !config.healthchecks.enabled {
        info!("Health checks are disabled.");
    }
    config.healthchecks.set_require_healthy(require_healthy);
    config.graceful_shutdown_duration = graceful_shutdown_duration;

    Ok(config)
}

#[cfg(feature = "enterprise")]
// Enable enterprise features, if applicable.
fn build_enterprise(
    config: &mut Config,
    config_paths: Vec<ConfigPath>,
) -> Result<Option<EnterpriseReporter<BoxFuture<'static, ()>>>, ExitCode> {
    match EnterpriseMetadata::try_from(&*config) {
        Ok(metadata) => {
            let enterprise = EnterpriseReporter::new();

            attach_enterprise_components(config, &metadata);
            enterprise.send(report_configuration(config_paths, metadata));

            Ok(Some(enterprise))
        }
        Err(EnterpriseError::MissingApiKey) => {
            error!("Enterprise configuration incomplete: missing API key.");
            Err(exitcode::CONFIG)
        }
        Err(_) => Ok(None),
    }
}

pub fn init_logging(color: bool, format: LogFormat, log_level: &str, rate: u64) {
    let level = get_log_levels(log_level);
    let json = match format {
        LogFormat::Text => false,
        LogFormat::Json => true,
    };

    trace::init(color, json, &level, rate);
    debug!(
        message = "Internal log rate limit configured.",
        internal_log_rate_secs = rate,
    );
    info!(message = "Log level is enabled.", level = ?level);
}

/// Load the legacy OpenSSL provider.
///
/// The returned [Provider] must stay in scope for the entire lifetime of the application, as it
/// will be unloaded when it is dropped.
pub fn load_openssl_legacy_providers() -> Result<Vec<Provider>, ExitCode> {
    warn!(message = "DEPRECATED The openssl legacy provider provides algorithms and key sizes no longer recommended for use. Set `--openssl-legacy-provider=false` or `VECTOR_OPENSSL_LEGACY_PROVIDER=false` to disable. See https://vector.dev/highlights/2023-08-15-0-32-0-upgrade-guide/#legacy-openssl for details.");
    ["legacy", "default"].into_iter().map(|provider_name| {
        Provider::try_load(None, provider_name, true)
            .map(|provider| {
                info!(message = "Loaded openssl provider.", provider = provider_name);
                provider
            })
            .map_err(|error| {
                error!(message = "Failed to load openssl provider.", provider = provider_name, %error);
                exitcode::UNAVAILABLE
            })
    }).collect()
}
