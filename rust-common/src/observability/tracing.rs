use opentelemetry::global;
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace;
use opentelemetry_sdk::trace::{Sampler, TracerProvider};

use crate::duration::parse_go_duration;
use crate::observability::resource::init_resource;
use crate::settings::ExporterType::{OtlpGrpc, OtlpHttp};
use crate::settings::Settings;

pub fn init_tracer(settings: &Settings, service_name: &str) {
    let observability = settings.observability.as_ref().unwrap();
    let tracing_config = observability.trace.as_ref().unwrap();

    let export_config = ExportConfig {
        timeout: parse_go_duration(tracing_config.exporter.timeout.as_ref().unwrap())
            .unwrap_or_default(),
        endpoint: format!(
            "{}{}",
            if tracing_config.exporter.insecure.unwrap_or_default() {
                "http://"
            } else {
                "https://"
            },
            tracing_config.exporter.endpoint.as_ref().unwrap()
        ),
        ..Default::default()
    };

    let exporter = match tracing_config.exporter.exporter_type.as_ref().unwrap() {
        OtlpHttp => opentelemetry_otlp::new_exporter()
            .http()
            .with_export_config(export_config)
            .build_span_exporter()
            .unwrap(),
        OtlpGrpc => opentelemetry_otlp::new_exporter()
            .tonic()
            .with_export_config(export_config)
            .build_span_exporter()
            .unwrap(),
    };

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, Tokio)
        .with_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(
                    tracing_config.sampling_ratio.unwrap_or_default(),
                ))
                .with_resource(init_resource(settings, service_name)),
        )
        .build();

    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider);
}
