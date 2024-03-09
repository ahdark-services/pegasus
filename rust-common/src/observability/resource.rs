use std::time::Duration;

use opentelemetry::KeyValue;
use opentelemetry_sdk::resource::{OsResourceDetector, TelemetryResourceDetector};
use opentelemetry_sdk::Resource;

use crate::settings::Settings;

pub fn init_resource(settings: &Settings, service_name: &str) -> Resource {
    let detector_resources = Box::new(Resource::from_detectors(
        Duration::from_secs(10),
        vec![
            Box::new(OsResourceDetector),
            Box::new(TelemetryResourceDetector),
        ],
    ));

    Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            format!("{}-{}", settings.namespace, service_name),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
            env!("CARGO_PKG_VERSION"),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAMESPACE,
            settings.namespace.clone(),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
            settings
                .instance_id
                .clone()
                .unwrap_or(uuid::Uuid::new_v4().to_string()),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::DEPLOYMENT_ENVIRONMENT,
            if settings.debug {
                "development"
            } else {
                "production"
            },
        ),
    ])
    .merge(detector_resources)
}
