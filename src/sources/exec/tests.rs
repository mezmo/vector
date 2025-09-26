use crate::sources::exec::*;
use crate::{event::LogEvent, test_util::trace_init};
use bytes::Bytes;
use std::ffi::OsStr;
use std::io::Cursor;
use tokio_test::assert_ok;
use vector_lib::event::EventMetadata;
use vrl::value;

#[cfg(unix)]
use futures::task::Poll;

const SINGLE_EVENT_VRL_SOURCE: &str = r#"
    . = {
        "metadata": {
            "mezmo": {
                "type": "clock_tick"
            }
        },
        "message": {
            "timestamp": now(),
            "origin": "exec",
            "mode": "scheduled"
        }
    }
"#;

const MULTI_EVENT_VRL_SOURCE: &str = r#"
    .message.data = {
    "users": [
      {
        "name": "jon",
        "uid": 1
      },
      {
        "name": "jane",
        "uid": 2
      }
    ]
  }
  result = unnest(.message.data.users)
  . = map_values(result) -> |item| {
    item.message.user = item.message.data.users;
    item
  }
"#;

#[test]
fn test_generate_config() {
    crate::test_util::test_generate_config::<ExecConfig>();
}

#[test]
fn test_scheduled_handle_event() {
    let config = standard_scheduled_test_config();
    let hostname = Some("Some.Machine".to_string());
    let data_stream = Some(STDOUT.to_string());
    let pid = Some(8888_u32);

    let mut event = LogEvent::from("hello world").into();
    handle_event(
        &config,
        &hostname,
        &data_stream,
        pid,
        &mut event,
        LogNamespace::Legacy,
    );
    let log = event.as_log();

    assert_eq!(*log.get_host().unwrap(), "Some.Machine".into());
    assert_eq!(log[STREAM_KEY], STDOUT.into());
    assert_eq!(log[PID_KEY], (8888_i64).into());
    assert_eq!(log[COMMAND_KEY], config.command.into());
    assert_eq!(*log.get_message().unwrap(), "hello world".into());
    assert_eq!(*log.get_source_type().unwrap(), "exec".into());
    assert!(log.get_timestamp().is_some());
    assert_eq!(log[EXEC_TYPE_KEY], "shell".into());

    let config = config_with_vrl(SINGLE_EVENT_VRL_SOURCE);
    event = LogEvent::from("hello world").into();
    handle_event(
        &config,
        &hostname,
        &data_stream,
        pid,
        &mut event,
        LogNamespace::Legacy,
    );
    let log = event.as_log();
    assert_eq!(log[EXEC_TYPE_KEY], "vrl".into());
}

#[test]
fn test_scheduled_handle_event_vector_namespace() {
    let config = standard_scheduled_test_config();
    let hostname = Some("Some.Machine".to_string());
    let data_stream = Some(STDOUT.to_string());
    let pid = Some(8888_u32);

    let mut event: Event =
        LogEvent::from_parts(value!("hello world"), EventMetadata::default()).into();

    handle_event(
        &config,
        &hostname,
        &data_stream,
        pid,
        &mut event,
        LogNamespace::Vector,
    );

    let log = event.as_log();
    let meta = log.metadata().value();

    assert_eq!(
        meta.get(path!(ExecConfig::NAME, "host")).unwrap(),
        &value!("Some.Machine")
    );
    assert_eq!(
        meta.get(path!(ExecConfig::NAME, STREAM_KEY)).unwrap(),
        &value!(STDOUT)
    );
    assert_eq!(
        meta.get(path!(ExecConfig::NAME, PID_KEY)).unwrap(),
        &value!(8888_i64)
    );
    assert_eq!(
        meta.get(path!(ExecConfig::NAME, COMMAND_KEY)).unwrap(),
        &value!(config.command)
    );
    assert_eq!(log.value(), &value!("hello world"));
    assert_eq!(
        meta.get(path!("vector", "source_type")).unwrap(),
        &value!("exec")
    );
    assert!(meta
        .get(path!("vector", "ingest_timestamp"))
        .unwrap()
        .is_timestamp());
    assert_eq!(
        meta.get(path!(ExecConfig::NAME, EXEC_TYPE_KEY)).unwrap(),
        &value!("shell")
    );
}

#[test]
fn test_streaming_create_event() {
    let config = standard_streaming_test_config();
    let hostname = Some("Some.Machine".to_string());
    let data_stream = Some(STDOUT.to_string());
    let pid = Some(8888_u32);

    let mut event = LogEvent::from("hello world").into();
    handle_event(
        &config,
        &hostname,
        &data_stream,
        pid,
        &mut event,
        LogNamespace::Legacy,
    );
    let log = event.as_log();

    assert_eq!(*log.get_host().unwrap(), "Some.Machine".into());
    assert_eq!(log[STREAM_KEY], STDOUT.into());
    assert_eq!(log[PID_KEY], (8888_i64).into());
    assert_eq!(log[COMMAND_KEY], config.command.into());
    assert_eq!(*log.get_message().unwrap(), "hello world".into());
    assert_eq!(*log.get_source_type().unwrap(), "exec".into());
    assert!(log.get_timestamp().is_some());
    assert_eq!(log[EXEC_TYPE_KEY], "shell".into());

    let config = config_with_vrl(SINGLE_EVENT_VRL_SOURCE);
    event = LogEvent::from("hello world").into();
    handle_event(
        &config,
        &hostname,
        &data_stream,
        pid,
        &mut event,
        LogNamespace::Legacy,
    );
    let log = event.as_log();
    assert_eq!(log[EXEC_TYPE_KEY], "vrl".into());
}

#[test]
fn test_streaming_create_event_vector_namespace() {
    let config = standard_streaming_test_config();
    let hostname = Some("Some.Machine".to_string());
    let data_stream = Some(STDOUT.to_string());
    let pid = Some(8888_u32);

    let mut event: Event =
        LogEvent::from_parts(value!("hello world"), EventMetadata::default()).into();

    handle_event(
        &config,
        &hostname,
        &data_stream,
        pid,
        &mut event,
        LogNamespace::Vector,
    );

    let log = event.as_log();
    let meta = event.metadata().value();

    assert_eq!(
        meta.get(path!(ExecConfig::NAME, "host")).unwrap(),
        &value!("Some.Machine")
    );
    assert_eq!(
        meta.get(path!(ExecConfig::NAME, STREAM_KEY)).unwrap(),
        &value!(STDOUT)
    );
    assert_eq!(
        meta.get(path!(ExecConfig::NAME, PID_KEY)).unwrap(),
        &value!(8888_i64)
    );
    assert_eq!(
        meta.get(path!(ExecConfig::NAME, COMMAND_KEY)).unwrap(),
        &value!(config.command)
    );
    assert_eq!(log.value(), &value!("hello world"));
    assert_eq!(
        meta.get(path!("vector", "source_type")).unwrap(),
        &value!("exec")
    );
    assert!(meta
        .get(path!("vector", "ingest_timestamp"))
        .unwrap()
        .is_timestamp());
    assert_eq!(
        meta.get(path!(ExecConfig::NAME, EXEC_TYPE_KEY)).unwrap(),
        &value!("shell")
    );

    let config = config_with_vrl(SINGLE_EVENT_VRL_SOURCE);
    event = LogEvent::from_parts(value!("hello world"), EventMetadata::default()).into();
    handle_event(
        &config,
        &hostname,
        &data_stream,
        pid,
        &mut event,
        LogNamespace::Vector,
    );
    assert_eq!(
        event
            .metadata()
            .value()
            .get(path!(ExecConfig::NAME, EXEC_TYPE_KEY))
            .unwrap(),
        &value!("vrl")
    );
}

#[test]
fn test_build_command() {
    let config = ExecConfig {
        mode: Mode::Streaming,
        scheduled: None,
        streaming: Some(StreamingConfig {
            respawn_on_exit: default_respawn_on_exit(),
            respawn_interval_secs: default_respawn_interval_secs(),
        }),
        command: Some(vec![
            "./runner".to_owned(),
            "arg1".to_owned(),
            "arg2".to_owned(),
        ]),
        source: None,
        environment: None,
        clear_environment: default_clear_environment(),
        working_directory: Some(PathBuf::from("/tmp")),
        include_stderr: default_include_stderr(),
        maximum_buffer_size_bytes: default_maximum_buffer_size(),
        framing: None,
        decoding: default_decoding(),
        log_namespace: None,
    };

    let command = build_command(&config);

    let mut expected_command = Command::new("./runner");
    expected_command.kill_on_drop(true);
    expected_command.current_dir("/tmp");
    expected_command.args(vec!["arg1".to_owned(), "arg2".to_owned()]);

    // Unfortunately the current_dir is not included in the formatted string
    let expected_command_string = format!("{expected_command:?}");
    let command_string = format!("{command:?}");

    assert_eq!(expected_command_string, command_string);
}

#[test]
fn test_build_command_custom_environment() {
    let config = ExecConfig {
        mode: Mode::Streaming,
        scheduled: None,
        streaming: Some(StreamingConfig {
            respawn_on_exit: default_respawn_on_exit(),
            respawn_interval_secs: default_respawn_interval_secs(),
        }),
        command: Some(vec![
            "./runner".to_owned(),
            "arg1".to_owned(),
            "arg2".to_owned(),
        ]),
        source: None,
        environment: Some(HashMap::from([("FOO".to_owned(), "foo".to_owned())])),
        clear_environment: default_clear_environment(),
        working_directory: Some(PathBuf::from("/tmp")),
        include_stderr: default_include_stderr(),
        maximum_buffer_size_bytes: default_maximum_buffer_size(),
        framing: None,
        decoding: default_decoding(),
        log_namespace: None,
    };

    let command = build_command(&config);
    let cmd = command.as_std();

    let idx = cmd
        .get_envs()
        .position(|v| v == (OsStr::new("FOO"), Some(OsStr::new("foo"))));

    assert_ne!(idx, None);
}

#[test]
fn test_build_command_clear_environment() {
    let config = ExecConfig {
        mode: Mode::Streaming,
        scheduled: None,
        streaming: Some(StreamingConfig {
            respawn_on_exit: default_respawn_on_exit(),
            respawn_interval_secs: default_respawn_interval_secs(),
        }),
        command: Some(vec![
            "./runner".to_owned(),
            "arg1".to_owned(),
            "arg2".to_owned(),
        ]),
        source: None,
        environment: Some(HashMap::from([("FOO".to_owned(), "foo".to_owned())])),
        clear_environment: true,
        working_directory: Some(PathBuf::from("/tmp")),
        include_stderr: default_include_stderr(),
        maximum_buffer_size_bytes: default_maximum_buffer_size(),
        framing: None,
        decoding: default_decoding(),
        log_namespace: None,
    };

    let command = build_command(&config);
    let cmd = command.as_std();

    let envs: Vec<_> = cmd.get_envs().collect();

    assert_eq!(envs.len(), 1);
}

#[tokio::test]
async fn test_spawn_reader_thread() {
    trace_init();

    let buf = Cursor::new("hello world\nhello rocket 🚀");
    let reader = BufReader::new(buf);
    let decoder = crate::codecs::Decoder::default();
    let (sender, mut receiver) = channel(1024);

    spawn_reader_thread(reader, decoder, STDOUT, sender);

    let mut counter = 0;
    if let Some(((events, byte_size), origin)) = receiver.recv().await {
        assert_eq!(byte_size, 11);
        assert_eq!(events.len(), 1);
        let log = events[0].as_log();
        assert_eq!(
            *log.get_message().unwrap(),
            Bytes::from("hello world").into()
        );
        assert_eq!(origin, STDOUT);
        counter += 1;
    }

    if let Some(((events, byte_size), origin)) = receiver.recv().await {
        assert_eq!(byte_size, 17);
        assert_eq!(events.len(), 1);
        let log = events[0].as_log();
        assert_eq!(
            *log.get_message().unwrap(),
            Bytes::from("hello rocket 🚀").into()
        );
        assert_eq!(origin, STDOUT);
        counter += 1;
    }

    assert_eq!(counter, 2);
}

#[tokio::test]
async fn test_drop_receiver() {
    let config = standard_scheduled_test_config();
    let hostname = Some("Some.Machine".to_string());
    let decoder = Default::default();
    let shutdown = ShutdownSignal::noop();
    let (tx, rx) = SourceSender::new_test();

    let mut executor = CommandExecInner {
        config: config.clone(),
        decoder,
        hostname,
        log_namespace: LogNamespace::Legacy,
        out: tx,
        shutdown,
    };
    // Wait for our task to finish, wrapping it in a timeout
    let timeout = tokio::time::timeout(time::Duration::from_secs(5), executor.run());

    drop(rx);

    let _timeout_result = crate::test_util::components::assert_source_error(
        &crate::test_util::components::COMPONENT_ERROR_TAGS,
        timeout,
    )
    .await;
}

#[tokio::test]
#[cfg(unix)]
async fn test_run_command_linux() {
    let config = standard_scheduled_test_config();

    let (mut rx, timeout_result) = crate::test_util::components::assert_source_compliance(
        &crate::test_util::components::SOURCE_TAGS,
        async {
            let hostname = Some("Some.Machine".to_string());
            let decoder = Default::default();
            let shutdown = ShutdownSignal::noop();
            let (tx, rx) = SourceSender::new_test();

            let mut executor = CommandExecInner {
                config: config.clone(),
                decoder,
                hostname,
                log_namespace: LogNamespace::Legacy,
                out: tx,
                shutdown,
            };
            // Wait for our task to finish, wrapping it in a timeout
            let result = tokio::time::timeout(time::Duration::from_secs(5), executor.run()).await;
            (rx, result)
        },
    )
    .await;

    let exit_status = timeout_result
        .expect("command timed out")
        .expect("command error");
    assert_eq!(0_i32, exit_status.unwrap().code().unwrap());

    if let Poll::Ready(Some(event)) = futures::poll!(rx.next()) {
        let log = event.as_log();
        assert_eq!(log[COMMAND_KEY], config.command.clone().into());
        assert_eq!(log[EXEC_TYPE_KEY], "shell".into());
        assert_eq!(log[STREAM_KEY], STDOUT.into());
        assert_eq!(*log.get_source_type().unwrap(), "exec".into());
        assert_eq!(*log.get_message().unwrap(), "Hello World!".into());
        assert_eq!(*log.get_host().unwrap(), "Some.Machine".into());
        assert!(log.get(PID_KEY).is_some());
        assert!(log.get_timestamp().is_some());

        assert_eq!(9, log.all_event_fields().unwrap().count());
    } else {
        panic!("Expected to receive a linux event");
    }
}

#[tokio::test]
#[cfg(unix)]
async fn test_graceful_shutdown() {
    trace_init();
    let mut config = standard_streaming_test_config();
    config.command = Some(vec![
        String::from("bash"),
        String::from("-c"),
        String::from(
            r"trap 'echo signal received ; sleep 1; echo slept ; exit' SIGTERM; while true ; do sleep 10 ; done",
        ),
    ]);
    let hostname = Some("Some.Machine".to_string());
    let decoder = Default::default();
    let (trigger, shutdown, _) = ShutdownSignal::new_wired();
    let (tx, mut rx) = SourceSender::new_test();

    let mut executor = CommandExecInner {
        config: config.clone(),
        decoder,
        hostname,
        log_namespace: LogNamespace::Legacy,
        out: tx,
        shutdown,
    };
    let task = tokio::spawn(async move { executor.run().await });

    tokio::time::sleep(Duration::from_secs(1)).await; // let the source start the command

    drop(trigger); // start shutdown

    let exit_status = tokio::time::timeout(time::Duration::from_secs(30), task)
        .await
        .expect("join failed")
        .expect("command timed out")
        .expect("command error");

    assert_eq!(
        0_i32,
        exit_status.expect("missing exit status").code().unwrap()
    );

    if let Poll::Ready(Some(event)) = futures::poll!(rx.next()) {
        let log = event.as_log();
        assert_eq!(*log.get_message().unwrap(), "signal received".into());
    } else {
        panic!("Expected to receive event");
    }

    if let Poll::Ready(Some(event)) = futures::poll!(rx.next()) {
        let log = event.as_log();
        assert_eq!(*log.get_message().unwrap(), "slept".into());
    } else {
        panic!("Expected to receive event");
    }
}

#[test]
fn test_maybe_compile_vrl() {
    let config = config_with_vrl(SINGLE_EVENT_VRL_SOURCE);

    let (sender, _) = SourceSender::new_test();
    let ctx = SourceContext::new_test(sender, Some(HashMap::default()));
    let output = maybe_compile_vrl_script(&config, &ctx);

    assert_ok!(output);
}

#[test]
fn test_validate() {
    let mut config = config_with_vrl(SINGLE_EVENT_VRL_SOURCE);

    config.command = None;
    config.source = None;
    let err = config
        .validate()
        .expect_err("Expected command validation error");
    assert_eq!(err, ExecConfigError::CommandEmpty);

    config.command = Some(vec![]);
    let err = config
        .validate()
        .expect_err("Expected command validation error");
    assert_eq!(err, ExecConfigError::CommandEmpty);

    config.command = None;
    config.source = Some("".to_owned());
    let err = config
        .validate()
        .expect_err("Expected VRL source validation error");
    assert_eq!(err, ExecConfigError::VrlSourceEmpty);

    config.source = Some("some vrl".to_owned());
    config.command = Some(vec!["command".to_owned()]);
    let err = config.validate().expect_err("Expected validation error");
    assert_eq!(err, ExecConfigError::CommandAndVrlProvided);

    config.source = None;
    config.command = Some(vec!["command".to_owned()]);
    config.maximum_buffer_size_bytes = 0;
    let err = config.validate().expect_err("Expected validation error");
    assert_eq!(err, ExecConfigError::ZeroBuffer);
}

#[tokio::test]
#[cfg(unix)]
async fn test_run_vrl_script() {
    let config = config_with_vrl(SINGLE_EVENT_VRL_SOURCE);

    let log_events = run_script_test(config).await;

    assert_eq!(log_events.len(), 1);
    assert_eq!(
        *log_events[0]["metadata"].get("mezmo.type").unwrap(),
        "clock_tick".into()
    );
    assert_eq!(
        *log_events[0]["message"].get("origin").unwrap(),
        "exec".into()
    );
    assert_eq!(
        *log_events[0]["message"].get("mode").unwrap(),
        "scheduled".into()
    );

    // multiple messages
    let config = config_with_vrl(MULTI_EVENT_VRL_SOURCE);
    let log_events = run_script_test(config).await;
    assert_eq!(log_events.len(), 2);
    assert_eq!(
        *log_events[0]
            .get_message()
            .unwrap()
            .get("user.name")
            .unwrap(),
        "jon".into()
    );
    assert_eq!(
        *log_events[1]
            .get_message()
            .unwrap()
            .get("user.name")
            .unwrap(),
        "jane".into()
    );
}

async fn run_script_test(config: ExecConfig) -> Vec<LogEvent> {
    let (mut rx, timeout_result) = crate::test_util::components::assert_source_compliance(
        &crate::test_util::components::SOURCE_TAGS,
        async {
            let (sender, rx) = SourceSender::new_test();
            let ctx = SourceContext::new_test(sender.clone(), Some(HashMap::default()));
            let program = maybe_compile_vrl_script(&config, &ctx).unwrap();

            let mut executor = VrlExecInner {
                config: config.clone(),
                log_namespace: LogNamespace::Legacy,
                program: program.unwrap(),
                out: sender,
            };
            let result = tokio::time::timeout(time::Duration::from_secs(5), executor.run()).await;
            (rx, result)
        },
    )
    .await;

    let res = timeout_result
        .expect("script timed out")
        .expect("script error");
    assert_eq!(res, None);

    let mut log_events = Vec::new();
    while let Some(event) = rx.next().await {
        log_events.push(event.as_log().clone());
    }
    log_events
}

fn standard_scheduled_test_config() -> ExecConfig {
    Default::default()
}

fn standard_streaming_test_config() -> ExecConfig {
    ExecConfig {
        mode: Mode::Streaming,
        scheduled: None,
        streaming: Some(StreamingConfig {
            respawn_on_exit: default_respawn_on_exit(),
            respawn_interval_secs: default_respawn_interval_secs(),
        }),
        command: Some(vec!["yes".to_owned()]),
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

fn config_with_vrl(source: &str) -> ExecConfig {
    ExecConfig {
        source: Some(source.to_owned()),
        command: None,
        ..Default::default()
    }
}
