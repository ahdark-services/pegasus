use actix_web::{App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use opentelemetry::global;

use pegasus_common::bot::channel::MqUpdateListener;
use pegasus_common::bot::new_bot;
use pegasus_common::bot::state::new_state_storage;
use pegasus_common::mq::connection::new_amqp_connection;
use pegasus_common::{database, observability, settings};

use crate::run::run;

mod handlers;
mod run;
mod services;
mod web;

const SERVICE_NAME: &str = "pm-bot-forwarding-handler";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let ref settings = settings::Settings::read_from_default_file().unwrap();
    observability::tracing::init_tracer(settings, SERVICE_NAME);

    let amqp_conn = new_amqp_connection(settings).await;
    let db = database::init_conn(settings.database.as_ref().unwrap()).await?;

    let bot = new_bot(settings.telegram_bot.as_ref().unwrap());
    let listener = MqUpdateListener::new(SERVICE_NAME, amqp_conn, settings).await?;
    let redis_storage = new_state_storage(settings).await;

    let forwarding_bot_service =
        services::forwarding_bot::ForwardingBotService::new(db.clone(), settings.clone());
    let forwarding_message_service =
        services::forwarding_message::ForwardingMessageService::new(db.clone(), settings.clone());

    log::info!("Application started");

    let run_bot = run(bot, listener, redis_storage, db, settings.clone());
    let run_web_server = HttpServer::new(move || {
        App::new()
            .wrap(RequestTracing::new())
            .app_data(forwarding_bot_service.clone())
            .app_data(forwarding_message_service.clone())
            .service(actix_web::web::scope("/webhook").route(
                "{token}",
                actix_web::web::post().to(web::forwarding_bot_update_handler),
            ))
    })
    .bind_auto_h2c(("0.0.0.0", 8080))
    .unwrap()
    .run();

    let (r1, r2) = tokio::join!(run_bot, run_web_server);
    r1?;
    r2?;

    log::info!("Shutting down tracer provider");
    global::shutdown_tracer_provider();

    Ok(())
}
