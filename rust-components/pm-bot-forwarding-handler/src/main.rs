use opentelemetry::global;

use pegasus_common::bot::channel::MqUpdateListener;
use pegasus_common::bot::new_bot;
use pegasus_common::bot::state::new_state_storage;
use pegasus_common::mq::connection::new_amqp_connection;
use pegasus_common::{observability, settings};

use crate::run::run;

mod handlers;
mod run;

const SERVICE_NAME: &str = "pm-bot-forwarding-handler";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let ref settings = settings::Settings::read_from_default_file().unwrap();
    observability::tracing::init_tracer(settings, SERVICE_NAME);

    let amqp_conn = new_amqp_connection(settings).await;

    let bot = new_bot(settings.telegram_bot.as_ref().unwrap());
    let listener = MqUpdateListener::new(SERVICE_NAME, amqp_conn, settings).await?;
    let redis_storage = new_state_storage(settings).await;

    log::info!("Application started");

    run(bot, listener, redis_storage).await;

    log::info!("Shutting down tracer provider");
    global::shutdown_tracer_provider();

    Ok(())
}
