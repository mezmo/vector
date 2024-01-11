use indoc::indoc;
use vector_lib::finalization::{BatchNotifier, BatchStatus};

use crate::{
    config::SinkConfig,
    sinks::{sumo_logic::config::SumoLogicSinkConfig, util::test::load_sink},
    test_util::{
        components::{run_and_assert_sink_compliance, SINK_TAGS},
        generate_lines_with_stream,
    },
};

#[cfg(feature = "sumo-logic-integration-tests")]
#[tokio::test]
async fn test_sumo_sink_endpoint() {
    let config = indoc! {r#"
        endpoint = "sumo-logic-endpoint"
        compression = "gzip"
        category = "integration-test-pipeline"
    "#};

    let endpoint = std::env::var("TEST_SUMO_LOGIC_ENDPOINT")
        .expect("test endpoint environment variable not set");
    assert!(!endpoint.is_empty(), "$TEST_SUMO_LOGIC_ENDPOINT required");

    let config = config.replace("sumo-logic-endpoint", &endpoint);
    let (config, cx) = load_sink::<SumoLogicSinkConfig>(config.as_str()).unwrap();

    let (sink, _) = config.build(cx).await.unwrap();
    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let generator = |index| format!("sumo logic test log index {}", index);
    let (messages, events) = generate_lines_with_stream(generator, 3, Some(batch));

    for (index, message) in messages.iter().enumerate() {
        assert_eq!(&format!("sumo logic test log index {}", index), message);
    }

    run_and_assert_sink_compliance(sink, events, &SINK_TAGS).await;
    assert_eq!(receiver.await, BatchStatus::Delivered);
}
