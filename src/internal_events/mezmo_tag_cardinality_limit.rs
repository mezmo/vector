use metrics::counter;
use vector_lib::{
    emit,
    internal_event::{ComponentEventsDropped, InternalEvent, INTENTIONAL},
};

pub struct MezmoTagCardinalityLimitRejectingEvent<'a> {
    pub tag_key: &'a str,
    pub tag_value: &'a str,
}

impl<'a> InternalEvent for MezmoTagCardinalityLimitRejectingEvent<'a> {
    fn emit(self) {
        debug!(
            message = "Event containing tag with new value after hitting configured 'value_limit'; discarding event.",
            tag_key = self.tag_key,
            tag_value = self.tag_value,
            internal_log_rate_limit = true,
        );
        counter!("mezmo_tag_value_limit_exceeded_total", 1);

        emit!(ComponentEventsDropped::<INTENTIONAL> {
            count: 1,
            reason: "Tag value limit exceeded."
        })
    }
}

pub struct MezmoTagCardinalityLimitRejectingTag<'a> {
    pub tag_key: &'a str,
    pub tag_value: &'a str,
}

impl<'a> InternalEvent for MezmoTagCardinalityLimitRejectingTag<'a> {
    fn emit(self) {
        debug!(
            message = "Rejecting tag after hitting configured 'value_limit'.",
            tag_key = self.tag_key,
            tag_value = self.tag_value,
            internal_log_rate_limit = true,
        );
        counter!("mezmo_tag_value_limit_exceeded_total", 1);
    }
}

pub struct MezmoTagCardinalityValueLimitReached<'a> {
    pub key: &'a str,
}

impl<'a> InternalEvent for MezmoTagCardinalityValueLimitReached<'a> {
    fn emit(self) {
        debug!(
            message = "Value_limit reached for key. New values for this key will be rejected.",
            key = %self.key,
        );
        counter!("mezmo_value_limit_reached_total", 1);
    }
}
