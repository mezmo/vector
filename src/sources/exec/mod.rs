use std::{
    borrow::BorrowMut,
    collections::HashMap,
    io::{Error, ErrorKind},
    path::PathBuf,
    process::ExitStatus,
};

use chrono::Utc;
use futures::StreamExt;
use smallvec::SmallVec;
use snafu::Snafu;
use tokio::{
    io::{AsyncRead, BufReader},
    process::Command,
    sync::mpsc::{channel, Sender},
    time::{self, sleep, Duration, Instant},
};
use tokio_stream::wrappers::IntervalStream;
use tokio_util::codec::FramedRead;
use vector_lib::configurable::configurable_component;
use vector_lib::internal_event::{ByteSize, BytesReceived, InternalEventHandle as _, Protocol};
use vector_lib::{
    codecs::{
        decoding::{DeserializerConfig, FramingConfig},
        StreamDecodingError,
    },
    config::LegacyKey,
    event::{LogEvent, TargetEvents},
    schema, ByteSizeOf, EstimatedJsonEncodedSizeOf, TimeZone,
};
use vrl::value::Kind;
use vrl::{
    compiler::{runtime::Runtime, Program},
    path::OwnedValuePath,
};

use crate::{
    codecs::{Decoder, DecodingConfig},
    config::{SourceConfig, SourceContext, SourceOutput},
    event::{Event, VrlTarget},
    internal_events::{
        ExecChannelClosedError, ExecCommandExecuted, ExecEventsReceived, ExecFailedError,
        ExecFailedToSignalChild, ExecFailedToSignalChildError, ExecTimeoutError,
        ExecVrlEventsReceived, StreamClosedError,
    },
    serde::default_decoding,
    shutdown::ShutdownSignal,
    transforms::remap::RemapConfig,
    SourceSender,
};
use async_trait::async_trait;
use futures::FutureExt;
use vector_lib::config::{log_schema, LogNamespace};
use vector_lib::lookup::{owned_value_path, path};

#[cfg(test)]
mod tests;

/// Configuration for the `exec` source.
#[configurable_component(source("exec", "Collect output from a process running on the host."))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExecConfig {
    #[configurable(derived)]
    pub mode: Mode,

    #[configurable(derived)]
    pub scheduled: Option<ScheduledConfig>,

    #[configurable(derived)]
    pub streaming: Option<StreamingConfig>,

    /// The command to run, plus any arguments required.
    #[configurable(metadata(docs::examples = "echo", docs::examples = "Hello World!"))]
    pub command: Option<Vec<String>>,

    /// The VRL program to execute
    #[configurable(derived)]
    source: Option<String>,

    /// Custom environment variables to set or update when running the command.
    /// If a variable name already exists in the environment, its value is replaced.
    #[serde(default)]
    #[configurable(metadata(docs::additional_props_description = "An environment variable."))]
    #[configurable(metadata(docs::examples = "environment_examples()"))]
    pub environment: Option<HashMap<String, String>>,

    /// Whether or not to clear the environment before setting custom environment variables.
    #[serde(default = "default_clear_environment")]
    pub clear_environment: bool,

    /// The directory in which to run the command.
    pub working_directory: Option<PathBuf>,

    /// Whether or not the output from stderr should be included when generating events.
    #[serde(default = "default_include_stderr")]
    pub include_stderr: bool,

    /// The maximum buffer size allowed before a log event is generated.
    #[serde(default = "default_maximum_buffer_size")]
    pub maximum_buffer_size_bytes: usize,

    #[configurable(derived)]
    framing: Option<FramingConfig>,

    #[configurable(derived)]
    #[serde(default = "default_decoding")]
    decoding: DeserializerConfig,

    /// The namespace to use for logs. This overrides the global setting.
    #[configurable(metadata(docs::hidden))]
    #[serde(default)]
    log_namespace: Option<bool>,
}

/// Mode of operation for running the command.
#[configurable_component]
#[derive(Clone, Copy, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Mode {
    /// The command is run on a schedule.
    Scheduled,

    /// The command is run until it exits, potentially being restarted.
    Streaming,
}

/// Configuration options for scheduled commands.
#[configurable_component]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ScheduledConfig {
    /// The interval, in seconds, between scheduled command runs.
    ///
    /// If the command takes longer than `exec_interval_secs` to run, it is killed.
    #[serde(default = "default_exec_interval_secs")]
    exec_interval_secs: u64,
}

/// Configuration options for streaming commands.
#[configurable_component]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct StreamingConfig {
    /// Whether or not the command should be rerun if the command exits.
    #[serde(default = "default_respawn_on_exit")]
    respawn_on_exit: bool,

    /// The amount of time, in seconds, before rerunning a streaming command that exited.
    #[serde(default = "default_respawn_interval_secs")]
    #[configurable(metadata(docs::human_name = "Respawn Interval"))]
    respawn_interval_secs: u64,
}

#[derive(Debug, PartialEq, Eq, Snafu)]
pub enum ExecConfigError {
    #[snafu(display("A non-empty list for command must be provided"))]
    CommandEmpty,
    #[snafu(display("The maximum buffer size must be greater than zero"))]
    ZeroBuffer,
    #[snafu(display("A non-empty VRL source must be provided"))]
    VrlSourceEmpty,
    #[snafu(display("A command or VRL source to execute must be provided"))]
    CommandAndSourceEmpty,
    #[snafu(display("One of command or VRL source must be provided"))]
    CommandAndVrlProvided,
}

impl Default for ExecConfig {
    fn default() -> Self {
        ExecConfig {
            mode: Mode::Scheduled,
            scheduled: Some(ScheduledConfig {
                exec_interval_secs: default_exec_interval_secs(),
            }),
            streaming: None,
            command: Some(vec!["echo".to_owned(), "Hello World!".to_owned()]),
            source: None,
            environment: None,
            clear_environment: default_clear_environment(),
            working_directory: None,
            include_stderr: default_include_stderr(),
            maximum_buffer_size_bytes: default_maximum_buffer_size(),
            framing: None,
            decoding: default_decoding(),
            log_namespace: None,
        }
    }
}

const fn default_maximum_buffer_size() -> usize {
    // 1MB
    1000000
}

const fn default_exec_interval_secs() -> u64 {
    60
}

const fn default_respawn_interval_secs() -> u64 {
    5
}

const fn default_respawn_on_exit() -> bool {
    true
}

const fn default_clear_environment() -> bool {
    false
}

const fn default_include_stderr() -> bool {
    true
}

fn environment_examples() -> HashMap<String, String> {
    HashMap::<_, _>::from_iter([
        ("LANG".to_owned(), "es_ES.UTF-8".to_owned()),
        ("TZ".to_owned(), "Etc/UTC".to_owned()),
        ("PATH".to_owned(), "/bin:/usr/bin:/usr/local/bin".to_owned()),
    ])
}

fn get_hostname() -> Option<String> {
    crate::get_hostname().ok()
}

const STDOUT: &str = "stdout";
const STDERR: &str = "stderr";
const STREAM_KEY: &str = "stream";
const PID_KEY: &str = "pid";
const COMMAND_KEY: &str = "command";
const EXEC_TYPE_KEY: &str = "exec_type";

impl_generate_config_from_default!(ExecConfig);

impl ExecConfig {
    fn validate(&self) -> Result<(), ExecConfigError> {
        let script_mode = self.is_script_mode();

        if script_mode && self.command.is_some() {
            return Err(ExecConfigError::CommandAndVrlProvided);
        }
        if script_mode && self.source.as_ref().unwrap().is_empty() {
            return Err(ExecConfigError::VrlSourceEmpty);
        }
        if !script_mode && (self.command.is_none() || self.command.as_ref().unwrap().is_empty()) {
            return Err(ExecConfigError::CommandEmpty);
        }
        if self.maximum_buffer_size_bytes == 0 {
            return Err(ExecConfigError::ZeroBuffer);
        }
        Ok(())
    }

    fn command_line(&self) -> String {
        self.command.as_ref().unwrap().join(" ")
    }

    const fn exec_interval_secs_or_default(&self) -> u64 {
        match &self.scheduled {
            None => default_exec_interval_secs(),
            Some(config) => config.exec_interval_secs,
        }
    }

    const fn respawn_on_exit_or_default(&self) -> bool {
        match &self.streaming {
            None => default_respawn_on_exit(),
            Some(config) => config.respawn_on_exit,
        }
    }

    const fn respawn_interval_secs_or_default(&self) -> u64 {
        match &self.streaming {
            None => default_respawn_interval_secs(),
            Some(config) => config.respawn_interval_secs,
        }
    }

    const fn is_script_mode(&self) -> bool {
        self.source.is_some()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "exec")]
impl SourceConfig for ExecConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<super::Source> {
        self.validate()?;
        let hostname = get_hostname();
        let log_namespace = cx.log_namespace(self.log_namespace);

        let framing = self
            .framing
            .clone()
            .unwrap_or_else(|| self.decoding.default_stream_framing());
        let decoder =
            DecodingConfig::new(framing, self.decoding.clone(), LogNamespace::Legacy).build()?;
        let program = maybe_compile_vrl_script(self, &cx)?;

        match &self.mode {
            Mode::Scheduled => Ok(Box::pin(run_scheduled(
                self.clone(),
                hostname,
                decoder,
                cx.shutdown,
                cx.out,
                log_namespace,
                program.clone(),
            ))),
            Mode::Streaming => {
                let respawn_on_exit = self.respawn_on_exit_or_default();
                let respawn_interval_secs = self.respawn_interval_secs_or_default();

                Ok(Box::pin(run_streaming(
                    self.clone(),
                    hostname,
                    respawn_on_exit,
                    respawn_interval_secs,
                    decoder,
                    cx.shutdown,
                    cx.out,
                    log_namespace,
                    program.clone(),
                )))
            }
        }
    }

    fn outputs(&self, global_log_namespace: LogNamespace) -> Vec<SourceOutput> {
        let log_namespace = global_log_namespace.merge(Some(self.log_namespace.unwrap_or(false)));

        let schema_definition = self
            .decoding
            .schema_definition(log_namespace)
            .with_standard_vector_source_metadata()
            .with_source_metadata(
                Self::NAME,
                Some(LegacyKey::InsertIfEmpty(
                    log_schema()
                        .host_key()
                        .map_or(OwnedValuePath::root(), |key| key.clone()),
                )),
                &owned_value_path!("host"),
                Kind::bytes().or_undefined(),
                Some("host"),
            )
            .with_source_metadata(
                Self::NAME,
                Some(LegacyKey::InsertIfEmpty(owned_value_path!(STREAM_KEY))),
                &owned_value_path!(STREAM_KEY),
                Kind::bytes().or_undefined(),
                None,
            )
            .with_source_metadata(
                Self::NAME,
                Some(LegacyKey::InsertIfEmpty(owned_value_path!(PID_KEY))),
                &owned_value_path!(PID_KEY),
                Kind::integer().or_undefined(),
                None,
            )
            .with_source_metadata(
                Self::NAME,
                Some(LegacyKey::InsertIfEmpty(owned_value_path!(COMMAND_KEY))),
                &owned_value_path!(COMMAND_KEY),
                Kind::bytes(),
                None,
            )
            .with_source_metadata(
                Self::NAME,
                Some(LegacyKey::InsertIfEmpty(owned_value_path!(EXEC_TYPE_KEY))),
                &owned_value_path!(EXEC_TYPE_KEY),
                Kind::bytes(),
                None,
            );

        vec![SourceOutput::new_logs(
            self.decoding.output_type(),
            schema_definition,
        )]
    }

    fn can_acknowledge(&self) -> bool {
        false
    }
}

async fn run_scheduled(
    config: ExecConfig,
    hostname: Option<String>,
    decoder: Decoder,
    shutdown: ShutdownSignal,
    out: SourceSender,
    log_namespace: LogNamespace,
    program: Option<Program>,
) -> Result<(), ()> {
    debug!("Starting scheduled exec runs.");
    let exec_interval_secs = config.exec_interval_secs_or_default();
    let schedule = Duration::from_secs(exec_interval_secs);

    let mut interval = IntervalStream::new(time::interval(schedule)).take_until(shutdown.clone());

    while interval.next().await.is_some() {
        let mut executor = create_executor(
            config.clone(),
            hostname.clone(),
            decoder.clone(),
            shutdown.clone(),
            out.clone(),
            log_namespace,
            program.clone(),
        );

        // Wait for our task to finish, wrapping it in a timeout
        let timeout: Result<Result<Option<ExitStatus>, Error>, time::error::Elapsed> =
            tokio::time::timeout(schedule, executor.run()).await;

        match timeout {
            Ok(output) => {
                if let Err(command_error) = output {
                    emit!(ExecFailedError {
                        command: config.command_line().as_str(),
                        error: command_error,
                    });
                }
            }
            Err(error) => {
                emit!(ExecTimeoutError {
                    command: config.command_line().as_str(),
                    elapsed_seconds: schedule.as_secs(),
                    error,
                });
            }
        }
    }

    debug!("Finished scheduled exec runs.");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_streaming(
    config: ExecConfig,
    hostname: Option<String>,
    respawn_on_exit: bool,
    respawn_interval_secs: u64,
    decoder: Decoder,
    mut shutdown: ShutdownSignal,
    out: SourceSender,
    log_namespace: LogNamespace,
    program: Option<Program>,
) -> Result<(), ()> {
    // respawn only applies to command execution
    let mut executor = create_executor(
        config.clone(),
        hostname.clone(),
        decoder.clone(),
        shutdown.clone(),
        out.clone(),
        log_namespace,
        program.clone(),
    );
    if !config.is_script_mode() && respawn_on_exit {
        let duration = Duration::from_secs(respawn_interval_secs);

        // Continue to loop while not shutdown
        loop {
            let output = executor.run().await;

            // handle command finished
            if let Err(command_error) = output {
                emit!(ExecFailedError {
                    command: config.command_line().as_str(),
                    error: command_error,
                });
            }

            tokio::select! {
                _ = &mut shutdown => break, // will break early if a shutdown is started
                _ = sleep(duration) => debug!("Restarting streaming process."),
            }
        }
    } else {
        let output = executor.run().await;

        if let Err(command_error) = output {
            emit!(ExecFailedError {
                command: config.command_line().as_str(),
                error: command_error,
            });
        }
    }

    Ok(())
}

fn create_executor(
    config: ExecConfig,
    hostname: Option<String>,
    decoder: Decoder,
    shutdown: ShutdownSignal,
    out: SourceSender,
    log_namespace: LogNamespace,
    program: Option<Program>,
) -> Box<dyn ExecInner + Send> {
    if config.is_script_mode() {
        Box::new(VrlExecInner {
            config,
            log_namespace,
            out,
            program: program.unwrap(),
        })
    } else {
        Box::new(CommandExecInner {
            config,
            decoder,
            hostname,
            log_namespace,
            out,
            shutdown,
        })
    }
}

#[async_trait]
trait ExecInner {
    async fn run(&mut self) -> Result<Option<ExitStatus>, Error>;
}

#[derive(Clone)]
struct CommandExecInner {
    config: ExecConfig,
    hostname: Option<String>,
    decoder: Decoder,
    shutdown: ShutdownSignal,
    out: SourceSender,
    log_namespace: LogNamespace,
}
#[async_trait]
impl ExecInner for CommandExecInner {
    async fn run(&mut self) -> Result<Option<ExitStatus>, Error> {
        debug!("Starting command run.");
        let mut command = build_command(&self.config);

        // Mark the start time just before spawning the process as
        // this seems to be the best approximation of exec duration
        let start = Instant::now();

        let mut child = command.spawn()?;

        // Set up communication channels
        let (sender, mut receiver) = channel(1024);

        // Optionally include stderr
        if self.config.include_stderr {
            let stderr = child.stderr.take().ok_or_else(|| {
                Error::new(ErrorKind::Other, "Unable to take stderr of spawned process")
            })?;

            // Create stderr async reader
            let stderr_reader = BufReader::new(stderr);

            spawn_reader_thread(stderr_reader, self.decoder.clone(), STDERR, sender.clone());
        }

        let stdout = child.stdout.take().ok_or_else(|| {
            Error::new(ErrorKind::Other, "Unable to take stdout of spawned process")
        })?;

        // Create stdout async reader
        let stdout_reader = BufReader::new(stdout);

        let pid = child.id();

        spawn_reader_thread(stdout_reader, self.decoder.clone(), STDOUT, sender);

        let bytes_received = register!(BytesReceived::from(Protocol::NONE));

        // LOG-17649: Due to the `shutdown_child` call in the shutdown arm in the select! macro,
        // we need to fuse shutdown so shutdown.poll() won't be called after it produces a ready
        // response. Without the fuse call, the test sources::exec::tests::test_graceful_shutdown
        // will fail.
        let mut shutdown = self.shutdown.borrow_mut().fuse();
        'outer: loop {
            tokio::select! {
                _ = &mut shutdown => {
                    if !shutdown_child(&mut child, &command).await {
                            break 'outer; // couldn't signal, exit early
                    }
                }
                v = receiver.recv() => {
                    match v {
                        None => break 'outer,
                        Some(((mut events, byte_size), stream)) => {
                            bytes_received.emit(ByteSize(byte_size));

                            let count = events.len();
                            emit!(ExecEventsReceived {
                                count,
                                command: self.config.command_line().as_str(),
                                byte_size: events.estimated_json_encoded_size_of(),
                            });

                            for event in &mut events {
                                handle_event(&self.config, &self.hostname, &Some(stream.to_string()), pid, event, self.log_namespace);
                            }
                            if (self.out.send_batch(events).await).is_err() {
                                emit!(StreamClosedError { count });
                                break;
                            }
                        },
                    }
                }
            }
        }

        let elapsed = start.elapsed();

        let result = match child.try_wait() {
            Ok(Some(exit_status)) => {
                handle_exit_status(&self.config, exit_status.code(), elapsed);
                Ok(Some(exit_status))
            }
            Ok(None) => {
                handle_exit_status(&self.config, None, elapsed);
                Ok(None)
            }
            Err(error) => {
                error!(message = "Unable to obtain exit status.", %error);

                handle_exit_status(&self.config, None, elapsed);
                Ok(None)
            }
        };

        debug!("Finished command run.");

        result
    }
}

struct VrlExecInner {
    config: ExecConfig,
    out: SourceSender,
    log_namespace: LogNamespace,
    program: Program,
}

#[async_trait]
impl ExecInner for VrlExecInner {
    async fn run(&mut self) -> Result<Option<ExitStatus>, Error> {
        debug!("Starting vrl execution.");

        let event = LogEvent::default();
        let timezone = TimeZone::parse("UTC").unwrap();
        let mut target = VrlTarget::new(Event::Log(event), self.program.info(), false);
        let result = Runtime::default().resolve(&mut target, &self.program, &timezone);

        let bytes_received = register!(BytesReceived::from(Protocol::NONE));
        let source = self.config.source.as_ref().unwrap();

        // handle the result error
        if let Err(err) = result {
            error!("VRL execution terminated: {}", err)
        } else {
            match target.into_events(self.log_namespace) {
                TargetEvents::One(mut event) => {
                    bytes_received.emit(ByteSize(event.allocated_bytes()));
                    emit!(ExecVrlEventsReceived {
                        count: 1,
                        byte_size: event.estimated_json_encoded_size_of(),
                        source: source.as_str(),
                    });
                    handle_event(
                        &self.config,
                        &None,
                        &None,
                        None,
                        &mut event,
                        self.log_namespace,
                    );

                    if (self.out.send_event(event).await).is_err() {
                        emit!(StreamClosedError { count: 1 });
                    }
                }
                TargetEvents::Logs(mut events) => {
                    let mut count = 0;
                    for mut event in &mut events {
                        emit!(ExecVrlEventsReceived {
                            count: 1,
                            byte_size: event.estimated_json_encoded_size_of(),
                            source: source.as_str(),
                        });
                        handle_event(
                            &self.config,
                            &None,
                            &None,
                            None,
                            &mut event,
                            self.log_namespace,
                        );

                        if (self.out.send_event(event).await).is_err() {
                            count += 1;
                            emit!(StreamClosedError { count });
                        }
                    }
                }
                // for traces do nothing
                _ => panic!("VRL execution must return logs, not traces"),
            }
        }

        debug!("Finished vrl execution.");
        Ok(None)
    }
}

fn maybe_compile_vrl_script(
    config: &ExecConfig,
    ctx: &SourceContext,
) -> Result<Option<Program>, Error> {
    if !config.is_script_mode() {
        return Ok(None);
    }
    let remap_config = RemapConfig {
        source: config.source.clone(),
        ..Default::default()
    };
    let result = remap_config.compile_vrl_program(
        Default::default(),
        schema::Definition::any(),
        ctx.mezmo_ctx.clone(),
    );
    match result {
        Ok((program, _, _, _)) => Ok(Some(program)),
        Err(err) => Err(Error::new(
            ErrorKind::Other,
            format!("Error compiling VRL program. Reason: {}", err),
        )),
    }
}

fn handle_exit_status(config: &ExecConfig, exit_status: Option<i32>, exec_duration: Duration) {
    emit!(ExecCommandExecuted {
        command: config.command_line().as_str(),
        exit_status,
        exec_duration,
    });
}

#[cfg(unix)]
async fn shutdown_child(
    child: &mut tokio::process::Child,
    command: &tokio::process::Command,
) -> bool {
    match child.id().map(i32::try_from) {
        Some(Ok(pid)) => {
            // shutting down, send a SIGTERM to the child
            if let Err(error) = nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(pid),
                nix::sys::signal::Signal::SIGTERM,
            ) {
                emit!(ExecFailedToSignalChildError {
                    command,
                    error: ExecFailedToSignalChild::SignalError(error)
                });
                false
            } else {
                true
            }
        }
        Some(Err(err)) => {
            emit!(ExecFailedToSignalChildError {
                command,
                error: ExecFailedToSignalChild::FailedToMarshalPid(err)
            });
            false
        }
        None => {
            emit!(ExecFailedToSignalChildError {
                command,
                error: ExecFailedToSignalChild::NoPid
            });
            false
        }
    }
}

#[cfg(windows)]
async fn shutdown_child(
    child: &mut tokio::process::Child,
    command: &tokio::process::Command,
) -> bool {
    // TODO Graceful shutdown of Windows processes
    match child.kill().await {
        Ok(()) => true,
        Err(err) => {
            emit!(ExecFailedToSignalChildError {
                command: &command,
                error: ExecFailedToSignalChild::IoError(err)
            });
            false
        }
    }
}

fn build_command(config: &ExecConfig) -> Command {
    let command = config
        .command
        .clone()
        .expect("Expected command to be present");
    let command_args = if command.len() > 1 {
        Some(&command[1..])
    } else {
        None
    };

    let mut command = Command::new(&command[0]);

    if let Some(args) = command_args {
        command.args(args);
    };

    command.kill_on_drop(true);

    // Clear environment variables if needed
    if config.clear_environment {
        command.env_clear();
    }

    // Configure environment variables if needed
    if let Some(envs) = &config.environment {
        command.envs(envs);
    }

    // Explicitly set the current dir if needed
    if let Some(current_dir) = &config.working_directory {
        command.current_dir(current_dir);
    }

    // Pipe our stdout to the process
    command.stdout(std::process::Stdio::piped());

    // Pipe stderr to the process if needed
    if config.include_stderr {
        command.stderr(std::process::Stdio::piped());
    } else {
        command.stderr(std::process::Stdio::null());
    }

    // Stdin is not needed
    command.stdin(std::process::Stdio::null());

    command
}

fn handle_event(
    config: &ExecConfig,
    hostname: &Option<String>,
    data_stream: &Option<String>,
    pid: Option<u32>,
    event: &mut Event,
    log_namespace: LogNamespace,
) {
    if let Event::Log(log) = event {
        log_namespace.insert_standard_vector_source_metadata(log, ExecConfig::NAME, Utc::now());

        // Add data stream of stdin or stderr (if needed)
        if let Some(data_stream) = data_stream {
            log_namespace.insert_source_metadata(
                ExecConfig::NAME,
                log,
                Some(LegacyKey::InsertIfEmpty(path!(STREAM_KEY))),
                path!(STREAM_KEY),
                data_stream.clone(),
            );
        }

        // Add pid (if needed)
        if let Some(pid) = pid {
            log_namespace.insert_source_metadata(
                ExecConfig::NAME,
                log,
                Some(LegacyKey::InsertIfEmpty(path!(PID_KEY))),
                path!(PID_KEY),
                pid as i64,
            );
        }

        // Add hostname (if needed)
        if let Some(hostname) = hostname {
            log_namespace.insert_source_metadata(
                ExecConfig::NAME,
                log,
                log_schema().host_key().map(LegacyKey::InsertIfEmpty),
                path!("host"),
                hostname.clone(),
            );
        }

        // Add command
        if config.command.is_some() {
            log_namespace.insert_source_metadata(
                ExecConfig::NAME,
                log,
                Some(LegacyKey::InsertIfEmpty(path!(COMMAND_KEY))),
                path!(COMMAND_KEY),
                config.command.clone(),
            );
        }

        let exec_type = if config.is_script_mode() {
            "vrl".to_owned()
        } else {
            "shell".to_owned()
        };
        log_namespace.insert_source_metadata(
            ExecConfig::NAME,
            log,
            Some(LegacyKey::InsertIfEmpty(path!(EXEC_TYPE_KEY))),
            path!(EXEC_TYPE_KEY),
            exec_type,
        );
    }
}

fn spawn_reader_thread<R: 'static + AsyncRead + Unpin + std::marker::Send>(
    reader: BufReader<R>,
    decoder: Decoder,
    origin: &'static str,
    sender: Sender<((SmallVec<[Event; 1]>, usize), &'static str)>,
) {
    // Start the green background thread for collecting
    drop(tokio::spawn(async move {
        debug!("Start capturing {} command output.", origin);

        let mut stream = FramedRead::new(reader, decoder);
        while let Some(result) = stream.next().await {
            match result {
                Ok(next) => {
                    if sender.send((next, origin)).await.is_err() {
                        // If the receive half of the channel is closed, either due to close being
                        // called or the Receiver handle dropping, the function returns an error.
                        emit!(ExecChannelClosedError);
                        break;
                    }
                }
                Err(error) => {
                    // Error is logged by `crate::codecs::Decoder`, no further
                    // handling is needed here.
                    if !error.can_continue() {
                        break;
                    }
                }
            }
        }

        debug!("Finished capturing {} command output.", origin);
    }));
}
