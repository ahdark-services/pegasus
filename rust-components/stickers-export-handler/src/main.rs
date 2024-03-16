use std::env;

use opentelemetry::global;
use teloxide::Bot;

use pegasus_common::bot::channel::MqUpdateListener;
use pegasus_common::mq::connection::new_amqp_connection;
use pegasus_common::{observability, settings};

use crate::run::run;

mod handlers;
mod run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let settings_path = match env::args().nth(1) {
        Some(path) => {
            log::info!("Using settings file: {}", path);
            path.into()
        }
        None => env::current_dir().unwrap().join("config.yaml"),
    };
    let ref settings = settings::Settings::read_from_file(settings_path).unwrap();

    observability::tracing::init_tracer(
        settings,
        env::var("CARGO_PKG_NAME")
            .unwrap_or("unknown".parse().unwrap())
            .as_str(),
    );

    let amqp_conn = new_amqp_connection(settings).await;

    let bot = Bot::new(settings.telegram_bot.clone().unwrap().token);
    let listener = MqUpdateListener::new("stickers-export-handler", amqp_conn, settings).await?;

    log::info!("Application started");

    run(bot, listener).await;

    log::info!("Shutting down tracer provider");
    global::shutdown_tracer_provider();

    Ok(())
}
