use std::collections::HashSet;

use super::config::{BloomFilterConfig, Mode, default_cache_size, default_max_tag_size};
use super::*;
use crate::event::Event;
use crate::test_util::components::assert_transform_compliance;
use crate::transforms::test::create_topology;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[test]
fn generate_config() {
    crate::test_util::test_generate_config::<TagCardinalityLimitConfig>();
}

fn make_event(tags: BTreeMap<KeyString, Value>) -> Event {
    Event::Log(
        BTreeMap::from([(
            "message".into(),
            BTreeMap::from([("tags".into(), tags.into())]).into(),
        )])
        .into(),
    )
}

#[macro_export]
macro_rules! tags {
    () => { $crate::event::MetricTags::default() };

    ($($key:expr => $value:expr,)+) => { $crate::tags!($($key => $value),+) };

    ($($key:expr => $value:expr),*) => {
        [
            $( ($key.into(), $value.into()), )*
        ].into_iter().collect::<BTreeMap<KeyString, Value>>()
    };
}

const fn make_transform_hashset(
    value_limit: usize,
    limit_exceeded_action: LimitExceededAction,
) -> TagCardinalityLimitConfig {
    TagCardinalityLimitConfig {
        value_limit,
        limit_exceeded_action,
        mode: Mode::Exact,
        max_tag_size: default_max_tag_size(),
        tags: None,
        exclude_tags: None,
    }
}

const fn make_transform_bloom(
    value_limit: usize,
    limit_exceeded_action: LimitExceededAction,
) -> TagCardinalityLimitConfig {
    TagCardinalityLimitConfig {
        value_limit,
        limit_exceeded_action,
        mode: Mode::Probabilistic(BloomFilterConfig {
            cache_size_per_key: default_cache_size(),
        }),
        max_tag_size: default_max_tag_size(),
        tags: None,
        exclude_tags: None,
    }
}

#[tokio::test]
async fn tag_cardinality_limit_drop_event_hashset() {
    drop_event(make_transform_hashset(2, LimitExceededAction::DropEvent)).await;
}

#[tokio::test]
async fn tag_cardinality_limit_drop_event_bloom() {
    drop_event(make_transform_bloom(2, LimitExceededAction::DropEvent)).await;
}

async fn drop_event(config: TagCardinalityLimitConfig) {
    assert_transform_compliance(async move {
        let event1 = make_event(tags!("tag1" => "val1"));
        let event2 = make_event(tags!("tag1" => "val2"));
        let event3 = make_event(tags!("tag1" => "val3"));

        let (tx, rx) = mpsc::channel(1);
        let (topology, mut out) = create_topology(ReceiverStream::new(rx), config).await;

        tx.send(event1.clone()).await.unwrap();
        tx.send(event2.clone()).await.unwrap();
        tx.send(event3.clone()).await.unwrap();

        let new_event1 = out.recv().await;
        let new_event2 = out.recv().await;

        drop(tx);
        topology.stop().await;

        let new_event3 = out.recv().await;

        assert_eq!(new_event1, Some(event1));
        assert_eq!(new_event2, Some(event2));
        // Third value rejected since value_limit is 2.
        assert_eq!(None, new_event3);
    })
    .await;
}

#[tokio::test]
async fn tag_cardinality_limit_drop_tag_hashset() {
    drop_tag(make_transform_hashset(2, LimitExceededAction::DropTag)).await;
}

#[tokio::test]
async fn tag_cardinality_limit_drop_tag_bloom() {
    drop_tag(make_transform_bloom(2, LimitExceededAction::DropTag)).await;
}

async fn drop_tag(config: TagCardinalityLimitConfig) {
    assert_transform_compliance(async move {
        let tags1 = tags!("tag1" => "val1", "tag2" => "val1");
        let event1 = make_event(tags1);

        let tags2 = tags!("tag1" => "val2", "tag2" => "val1");
        let event2 = make_event(tags2);

        let tags3 = tags!("tag1" => "val3", "tag2" => "val1");
        let event3 = make_event(tags3);

        let (tx, rx) = mpsc::channel(1);
        let (topology, mut out) = create_topology(ReceiverStream::new(rx), config).await;

        tx.send(event1.clone()).await.unwrap();
        tx.send(event2.clone()).await.unwrap();
        tx.send(event3.clone()).await.unwrap();

        let new_event1 = out.recv().await;
        let new_event2 = out.recv().await;
        let new_event3 = out.recv().await;

        drop(tx);
        topology.stop().await;

        assert_eq!(new_event1, Some(event1));
        assert_eq!(new_event2, Some(event2));
        // The third event should have been modified to remove "tag1"
        assert_ne!(new_event3, Some(event3));

        let new_event3 = new_event3.unwrap();
        assert!(
            !new_event3
                .as_log()
                .get("message")
                .unwrap()
                .get("tags")
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key("tag1")
        );
        assert_eq!(
            "val1",
            new_event3
                .as_log()
                .get("message")
                .unwrap()
                .get("tags")
                .unwrap()
                .as_object()
                .unwrap()
                .get("tag2")
                .unwrap()
                .as_str()
                .unwrap()
        );
    })
    .await;
}

#[tokio::test]
async fn tag_cardinality_limit_separate_value_limit_per_tag_hashset() {
    separate_value_limit_per_tag(make_transform_hashset(2, LimitExceededAction::DropEvent)).await;
}

#[tokio::test]
async fn tag_cardinality_limit_separate_value_limit_per_tag_bloom() {
    separate_value_limit_per_tag(make_transform_bloom(2, LimitExceededAction::DropEvent)).await;
}

/// Test that hitting the value limit on one tag does not affect the ability to take new
/// values for other tags.
async fn separate_value_limit_per_tag(config: TagCardinalityLimitConfig) {
    assert_transform_compliance(async move {
        let event1 = make_event(tags!("tag1" => "val1", "tag2" => "val1"));

        let event2 = make_event(tags!("tag1" => "val2", "tag2" => "val1"));

        // Now value limit is reached for "tag1", but "tag2" still has values available.
        let event3 = make_event(tags!("tag1" => "val1", "tag2" => "val2"));

        let (tx, rx) = mpsc::channel(1);
        let (topology, mut out) = create_topology(ReceiverStream::new(rx), config).await;

        tx.send(event1.clone()).await.unwrap();
        tx.send(event2.clone()).await.unwrap();
        tx.send(event3.clone()).await.unwrap();

        let new_event1 = out.recv().await;
        let new_event2 = out.recv().await;
        let new_event3 = out.recv().await;

        drop(tx);
        topology.stop().await;

        assert_eq!(new_event1, Some(event1));
        assert_eq!(new_event2, Some(event2));
        assert_eq!(new_event3, Some(event3));
    })
    .await;
}

/// Test that hitting the value limit on one tag does not affect checking the limit on other
/// tags that happen to be ordered later
#[test]
fn drop_event_checks_all_tags1() {
    drop_event_checks_all_tags(|val1, val2| tags!("tag1" => val1, "tag2" => val2));
}

#[test]
fn drop_event_checks_all_tags2() {
    drop_event_checks_all_tags(|val1, val2| tags!("tag1" => val2, "tag2" => val1));
}

fn drop_event_checks_all_tags(make_tags: impl Fn(&str, &str) -> BTreeMap<KeyString, Value>) {
    let config = make_transform_hashset(2, LimitExceededAction::DropEvent);
    let mut transform = TagCardinalityLimit::new(config, None);

    let event1 = make_event(make_tags("val1", "val1"));
    let event2 = make_event(make_tags("val2", "val1"));
    // Next the limit is exceeded for the first tag.
    let event3 = make_event(make_tags("val3", "val2"));
    // And then check if the new value for the second tag was not recorded by the above event.
    let event4 = make_event(make_tags("val1", "val3"));

    let new_event1 = transform.transform_one(event1.clone());
    let new_event2 = transform.transform_one(event2.clone());
    let new_event3 = transform.transform_one(event3);
    let new_event4 = transform.transform_one(event4.clone());

    assert_eq!(new_event1, Some(event1));
    assert_eq!(new_event2, Some(event2));
    assert_eq!(new_event3, None);
    assert_eq!(new_event4, Some(event4));
}

/// Tests exclude_tags with all tags and with specific tags
#[test]
fn exclude_from_all_tags() {
    let config: TagCardinalityLimitConfig = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropEvent,
        mode: Mode::Exact,
        max_tag_size: default_max_tag_size(),
        tags: None,
        exclude_tags: Some(HashSet::from(["tag3".into(), "tag4".into()])),
    };
    exclude_tags_not_considered(config);
}

#[test]
fn exclude_from_specific_tags() {
    let config: TagCardinalityLimitConfig = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropEvent,
        mode: Mode::Exact,
        max_tag_size: default_max_tag_size(),
        tags: Some(HashSet::from([
            "tag1".into(),
            "tag2".into(),
            "tag3".into(),
            "tag4".into(),
        ])),
        exclude_tags: Some(HashSet::from(["tag3".into(), "tag4".into()])),
    };
    exclude_tags_not_considered(config);
}

fn exclude_tags_not_considered(config: TagCardinalityLimitConfig) {
    let mut transform: TagCardinalityLimit = TagCardinalityLimit::new(config, None);

    let event1 = make_event(tags!("tag1" => "val1", "tag2" => "val1"));
    let event2 = make_event(tags!("tag1" => "val2", "tag2" => "val1"));
    // Next the limit is exceeded for the first tag.
    let event3 = make_event(tags!("tag1" => "val3", "tag2" => "val2"));
    // And then check if the new value for the second tag was not recorded by the above event.
    let event4 = make_event(tags!("tag1" => "val1", "tag2" => "val3"));

    // These events are ignored because the config excludes them
    let event5 = make_event(tags!("tag3" => "val1", "tag4" => "val1"));
    let event6 = make_event(tags!("tag3" => "val2", "tag4" => "val2"));
    let event7 = make_event(tags!("tag3" => "val3", "tag4" => "val3"));

    let new_event1 = transform.transform_one(event1.clone());
    let new_event2 = transform.transform_one(event2.clone());
    let new_event3 = transform.transform_one(event3);
    let new_event4 = transform.transform_one(event4.clone());
    let new_event5 = transform.transform_one(event5.clone());
    let new_event6 = transform.transform_one(event6.clone());
    let new_event7 = transform.transform_one(event7.clone());

    assert_eq!(new_event1, Some(event1));
    assert_eq!(new_event2, Some(event2));
    assert_eq!(new_event3, None);
    assert_eq!(new_event4, Some(event4));
    assert_eq!(new_event5, Some(event5));
    assert_eq!(new_event6, Some(event6));
    assert_eq!(new_event7, Some(event7));
}

#[test]
fn drop_event_specific_tags_exact() {
    let config = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropEvent,
        mode: Mode::Exact,
        max_tag_size: default_max_tag_size(),
        tags: Some(HashSet::from(["tag1".into(), "tag2".into()])),
        exclude_tags: None,
    };
    drop_event_specific_tags(config);
}

#[test]
fn drop_event_specific_tags_prob() {
    let config = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropEvent,
        mode: Mode::Probabilistic(BloomFilterConfig {
            cache_size_per_key: default_cache_size(),
        }),
        max_tag_size: default_max_tag_size(),
        tags: Some(HashSet::from(["tag1".into(), "tag2".into()])),
        exclude_tags: None,
    };
    drop_event_specific_tags(config);
}

fn drop_event_specific_tags(config: TagCardinalityLimitConfig) {
    let mut transform = TagCardinalityLimit::new(config, None);

    let event1 = make_event(tags!("tag1" => "val1", "tag2" => "val1"));
    let event2 = make_event(tags!("tag1" => "val2", "tag2" => "val1"));
    // Next the limit is exceeded for the first tag.
    let event3 = make_event(tags!("tag1" => "val3", "tag2" => "val2"));
    // And then check if the new value for the second tag was not recorded by the above event.
    let event4 = make_event(tags!("tag1" => "val1", "tag2" => "val3"));

    // These events are ignored because the tags don't match
    let event5 = make_event(tags!("tag3" => "val1", "tag4" => "val1"));
    let event6 = make_event(tags!("tag3" => "val2", "tag4" => "val2"));
    let event7 = make_event(tags!("tag3" => "val3", "tag4" => "val3"));

    let new_event1 = transform.transform_one(event1.clone());
    let new_event2 = transform.transform_one(event2.clone());
    let new_event3 = transform.transform_one(event3);
    let new_event4 = transform.transform_one(event4.clone());
    let new_event5 = transform.transform_one(event5.clone());
    let new_event6 = transform.transform_one(event6.clone());
    let new_event7 = transform.transform_one(event7.clone());

    assert_eq!(new_event1, Some(event1));
    assert_eq!(new_event2, Some(event2));
    assert_eq!(new_event3, None);
    assert_eq!(new_event4, Some(event4));
    assert_eq!(new_event5, Some(event5));
    assert_eq!(new_event6, Some(event6));
    assert_eq!(new_event7, Some(event7));
}

#[test]
fn drop_specific_tags_exact() {
    let config = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropTag,
        mode: Mode::Exact,
        max_tag_size: default_max_tag_size(),
        tags: Some(HashSet::from(["tag1".into(), "tag2".into()])),
        exclude_tags: None,
    };
    drop_specific_tags(config);
}

#[test]
fn drop_specific_tags_prob() {
    let config = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropTag,
        mode: Mode::Probabilistic(BloomFilterConfig {
            cache_size_per_key: default_cache_size(),
        }),
        max_tag_size: default_max_tag_size(),
        tags: Some(HashSet::from(["tag1".into(), "tag2".into()])),
        exclude_tags: None,
    };
    drop_specific_tags(config);
}

fn drop_specific_tags(config: TagCardinalityLimitConfig) {
    let mut transform = TagCardinalityLimit::new(config, None);

    let event1 = make_event(tags!("tag1" => "val1", "tag2" => "val1"));
    let event2 = make_event(tags!("tag1" => "val2", "tag2" => "val1"));

    // Next the limit is exceeded for the first tag so it is dropped.
    let event3 = make_event(tags!("tag1" => "val3", "tag2" => "val2"));

    // Then the limit is exceeded for the second tag so it is dropped.
    let event4 = make_event(tags!("tag1" => "val1", "tag2" => "val3"));

    // These events are ignored because the tags don't match
    let event5 = make_event(tags!("tag3" => "val1", "tag4" => "val1"));
    let event6 = make_event(tags!("tag3" => "val2", "tag4" => "val2"));
    let event7 = make_event(tags!("tag3" => "val3", "tag4" => "val3"));

    let new_event1 = transform.transform_one(event1.clone());
    let new_event2 = transform.transform_one(event2.clone());
    let new_event3 = transform.transform_one(event3);
    let new_event4 = transform.transform_one(event4);
    let new_event5 = transform.transform_one(event5.clone());
    let new_event6 = transform.transform_one(event6.clone());
    let new_event7 = transform.transform_one(event7.clone());

    assert_eq!(new_event1, Some(event1));
    assert_eq!(new_event2, Some(event2));
    assert_eq!(new_event3, Some(make_event(tags!("tag2" => "val2")))); // "tag1" should be dropped
    assert_eq!(new_event4, Some(make_event(tags!("tag1" => "val1")))); // "tag2" should be dropped
    assert_eq!(new_event5, Some(event5));
    assert_eq!(new_event6, Some(event6));
    assert_eq!(new_event7, Some(event7));
}

#[test]
fn drop_event_with_max_tag_size_exact() {
    let config = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropEvent,
        mode: Mode::Exact,
        max_tag_size: 4,
        tags: None,
        exclude_tags: None,
    };
    drop_event_with_max_tag_size(config);
}

#[test]
fn drop_event_with_max_tag_size_prob() {
    let config = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropEvent,
        mode: Mode::Probabilistic(BloomFilterConfig {
            cache_size_per_key: default_cache_size(),
        }),
        max_tag_size: 4,
        tags: None,
        exclude_tags: None,
    };
    drop_event_with_max_tag_size(config);
}

fn drop_event_with_max_tag_size(config: TagCardinalityLimitConfig) {
    let mut transform = TagCardinalityLimit::new(config, None);

    let event1 = make_event(tags!("tag1" => "val1", "tag2" => "val1"));
    let event2 = make_event(tags!("tag1" => "val2", "tag2" => "val1"));

    // Exceeds max tag length so it will be let through
    let event3 = make_event(tags!("tag1" => "val12", "tag2" => "val1"));

    // Next the limit is exceeded for the first tag so the event is dropped.
    let event4 = make_event(tags!("tag1" => "val3", "tag2" => "val1"));

    let new_event1 = transform.transform_one(event1.clone());
    let new_event2 = transform.transform_one(event2.clone());
    let new_event3 = transform.transform_one(event3.clone());
    let new_event4 = transform.transform_one(event4);

    assert_eq!(new_event1, Some(event1));
    assert_eq!(new_event2, Some(event2));
    assert_eq!(new_event3, Some(event3));
    assert_eq!(new_event4, None); // Event 4 should be dropped
}

#[test]
fn drop_tag_with_max_tag_size_exact() {
    let config = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropTag,
        mode: Mode::Exact,
        max_tag_size: 4,
        tags: None,
        exclude_tags: None,
    };
    drop_tag_with_max_tag_size(config);
}

#[test]
fn drop_tag_with_max_tag_size_prob() {
    let config = TagCardinalityLimitConfig {
        value_limit: 2,
        limit_exceeded_action: LimitExceededAction::DropTag,
        mode: Mode::Probabilistic(BloomFilterConfig {
            cache_size_per_key: default_cache_size(),
        }),
        max_tag_size: 4,
        tags: None,
        exclude_tags: None,
    };
    drop_tag_with_max_tag_size(config);
}

fn drop_tag_with_max_tag_size(config: TagCardinalityLimitConfig) {
    let mut transform = TagCardinalityLimit::new(config, None);

    let event1 = make_event(tags!("tag1" => "val1", "tag2" => "val1"));
    let event2 = make_event(tags!("tag1" => "val2", "tag2" => "val1"));

    // Exceeds max tag length so it will be let through
    let event3 = make_event(tags!("tag1" => "val12", "tag2" => "val1"));

    // Next the limit is exceeded for the first tag so it is dropped.
    let event4 = make_event(tags!("tag1" => "val3", "tag2" => "val1"));

    let new_event1 = transform.transform_one(event1.clone());
    let new_event2 = transform.transform_one(event2.clone());
    let new_event3 = transform.transform_one(event3.clone());
    let new_event4 = transform.transform_one(event4);

    assert_eq!(new_event1, Some(event1));
    assert_eq!(new_event2, Some(event2));
    assert_eq!(new_event3, Some(event3));
    assert_eq!(new_event4, Some(make_event(tags!("tag2" => "val1")))); // "tag1" should be dropped
}
