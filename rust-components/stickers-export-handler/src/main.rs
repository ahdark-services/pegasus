use std::env;

use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::{global, Context};

use pegasus_common::{observability, settings};

#[tokio::main]
async fn main() {
    let settings_path = match env::args().nth(1) {
        Some(path) => {
            log::info!("Using settings file: {}", path);
            path.into()
        }
        None => env::current_dir().unwrap().join("config.yaml"),
    };
    let settings = settings::Settings::read_from_file(settings_path).unwrap();

    observability::tracing::init_tracer(
        &settings,
        env::var("CARGO_PKG_NAME")
            .unwrap_or("unknown".parse().unwrap())
            .as_str(),
    );

    print_hello();

    global::shutdown_tracer_provider();
}

fn print_hello() {
    let _guard =
        Context::current_with_span(global::tracer("example-tracer").start("print_hello")).attach();

    println!("Hello, world!");
}
