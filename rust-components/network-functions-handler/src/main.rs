use std::env;

use opentelemetry::global;

use pegasus_common::bot::channel::MqUpdateListener;
use pegasus_common::bot::new_bot;
use pegasus_common::mq::connection::new_amqp_connection;
use pegasus_common::{observability, settings};

use crate::run::run;

mod handlers;
mod run;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    let service_name = env!("CARGO_BIN_NAME");
    let ref settings =
        settings::Settings::read_from_default_file().expect("Failed to read settings");
    observability::tracing::init_tracer(service_name, settings);

    let amqp_conn = new_amqp_connection(settings).await;

    let bot = new_bot(settings.telegram_bot.as_ref().unwrap());
    let listener = MqUpdateListener::new(service_name, amqp_conn, settings).await?;

    log::info!("Application started");

    run(bot, listener).await;

    log::info!("Shutting down tracer provider");
    global::shutdown_tracer_provider();

    Ok(())
}
