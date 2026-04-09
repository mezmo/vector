use std::io;

use serde_json::json;

use crate::sinks::{
    prelude::*,
    util::encoding::{Encoder as SinkEncoder, write_all},
};

pub(super) struct NetworkMockEncoder {
    pub(super) transformer: Transformer,
}

impl SinkEncoder<Vec<Event>> for NetworkMockEncoder {
    fn encode_input(
        &self,
        events: Vec<Event>,
        writer: &mut dyn io::Write,
    ) -> io::Result<(usize, GroupedCountByteSize)> {
        let mut byte_size = telemetry().create_request_count_byte_size();
        let n_events = events.len();
        let mut json_events: Vec<serde_json::Value> = Vec::with_capacity(n_events);

        for mut event in events {
            self.transformer.transform(&mut event);
            byte_size.add_event(&event, event.estimated_json_encoded_size_of());
            json_events.push(json!(event.as_log()));
        }

        let body = serde_json::to_vec(&json_events)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        write_all(writer, n_events, &body).map(|()| (body.len(), byte_size))
    }
}
