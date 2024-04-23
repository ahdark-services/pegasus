use opentelemetry::global;
use opentelemetry_otlp::{
    ExportConfig, HttpExporterBuilder, SpanExporterBuilder, TonicExporterBuilder, WithExportConfig,
};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::Sampler;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

use crate::duration::parse_go_duration;
use crate::observability::resource::init_resource;
use crate::settings::ExporterType::{OtlpGrpc, OtlpHttp};
use crate::settings::Settings;

pub fn init_tracer(service_name: &str, settings: &Settings) {
    let observability = settings.observability.as_ref().unwrap();
    let tracing_config = observability.trace.as_ref().unwrap();

    let export_config = ExportConfig {
        timeout: parse_go_duration(tracing_config.exporter.timeout.as_ref().unwrap())
            .unwrap_or_default(),
        endpoint: format!(
            "{}{}",
            if tracing_config.exporter.insecure.unwrap_or(true) {
                "http://"
            } else {
                "https://"
            },
            tracing_config.exporter.endpoint.as_ref().unwrap()
        ),
        ..Default::default()
    };

    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            match tracing_config.exporter.exporter_type.as_ref().unwrap() {
                OtlpHttp => SpanExporterBuilder::Http(
                    HttpExporterBuilder::default().with_export_config(export_config),
                ),
                OtlpGrpc => SpanExporterBuilder::Tonic(
                    TonicExporterBuilder::default().with_export_config(export_config),
                ),
            },
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(
                    tracing_config.sampling_ratio.clone().unwrap_or(1.0),
                ))
                .with_resource(init_resource(settings, service_name)),
        )
        .install_batch(Tokio)
        .expect("Failed to install `opentelemetry` tracer.");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO"));
    let subscriber = Registry::default().with(telemetry).with(env_filter);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");
}
