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
            settings.instance_id.as_ref().unwrap().clone(),
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

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_init_resource() {
        let settings = Settings {
            namespace: "namespace".to_string(),
            instance_id: Some("instance_id".to_string()),
            debug: true,
            ..Default::default()
        };

        let resource = init_resource(&settings, "service_name");

        assert_eq!(
            resource.get(opentelemetry_semantic_conventions::resource::SERVICE_NAME.into()),
            Some("namespace-service_name".into())
        );
        assert_eq!(
            resource.get(opentelemetry_semantic_conventions::resource::SERVICE_VERSION.into()),
            Some(env!("CARGO_PKG_VERSION").into())
        );
        assert_eq!(
            resource.get(opentelemetry_semantic_conventions::resource::SERVICE_NAMESPACE.into()),
            Some("namespace".into())
        );
        assert_eq!(
            resource.get(opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID.into()),
            Some("instance_id".into())
        );
        assert_eq!(
            resource
                .get(opentelemetry_semantic_conventions::resource::DEPLOYMENT_ENVIRONMENT.into()),
            Some("development".into())
        );
    }
}
