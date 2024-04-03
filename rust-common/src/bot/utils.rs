use std::collections::HashMap;

use lapin::types::AMQPValue;
use opentelemetry::trace::{SpanKind, TraceContextExt, Tracer};
use opentelemetry::{global, Context};

pub fn extract_span_from_delivery(delivery: &lapin::message::Delivery) -> Context {
    let headers = delivery
        .properties
        .headers()
        .as_ref()
        .unwrap_or(&Default::default())
        .inner()
        .clone();

    let parent_cx = global::get_text_map_propagator(|propagator| {
        let trace_data = match headers.get("x-trace") {
            Some(AMQPValue::FieldTable(t)) => t.clone(),
            _ => Default::default(),
        };
        let mut trace_data_map = HashMap::new();
        for x in &trace_data {
            let s = match x.1 {
                AMQPValue::ShortString(s) => s.to_string(),
                AMQPValue::LongString(s) => s.to_string(),
                _ => {
                    continue;
                }
            };

            trace_data_map.insert(x.0.to_string(), s);
        }

        propagator.extract(&trace_data_map)
    });

    let tracer = global::tracer("pegasus/rust-common/bot/channel");
    let span = tracer
        .span_builder("UpdateListener::as_stream")
        .with_kind(SpanKind::Consumer)
        .start_with_context(&tracer, &parent_cx);

    let cx = parent_cx.with_span(span);

    cx
}
