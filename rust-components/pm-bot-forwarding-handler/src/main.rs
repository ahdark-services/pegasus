use std::env;

use actix_web::{App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use opentelemetry::global;

use pegasus_common::bot::channel::MqUpdateListener;
use pegasus_common::bot::new_bot;
use pegasus_common::bot::state::new_state_storage;
use pegasus_common::mq::connection::new_amqp_connection;
use pegasus_common::{database, observability, redis, settings};

use crate::run::run;

mod handlers;
mod run;
mod services;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    let service_name = env!("CARGO_BIN_NAME");
    let ref settings =
        settings::Settings::read_from_default_file().expect("Failed to read settings");
    observability::tracing::init_tracer(service_name, settings);

    let amqp_conn = new_amqp_connection(settings).await;
    let db = database::init_conn(settings.database.as_ref().unwrap()).await?;
    let redis_client = redis::client::new_client(settings);

    let bot = new_bot(settings.telegram_bot.as_ref().unwrap());
    let listener = MqUpdateListener::new(service_name, amqp_conn, settings).await?;
    let redis_storage = new_state_storage(
        service_name,
        redis_client
            .get_multiplexed_tokio_connection()
            .await
            .unwrap(),
    )
    .await;

    let forwarding_bot_service =
        services::forwarding_bot::ForwardingBotService::new(db.clone(), settings.clone());
    let forwarding_message_service =
        services::forwarding_message::ForwardingMessageService::new(db.clone(), settings.clone());

    log::info!("Application started");

    let run_bot = run(bot, listener, redis_storage, db, settings.clone());
    let run_web_server = HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(RequestTracing::default())
            .app_data(actix_web::web::Data::new(forwarding_bot_service.clone()))
            .app_data(actix_web::web::Data::new(
                forwarding_message_service.clone(),
            ))
            .service(web::forwarding_bot_update_handler)
    })
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .run();

    let (r1, r2) = tokio::join!(run_bot, run_web_server);
    r1?;
    r2?;

    log::info!("Shutting down tracer provider");
    global::shutdown_tracer_provider();

    Ok(())
}
