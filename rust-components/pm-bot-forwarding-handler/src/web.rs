use std::borrow::Cow;

use actix_web::http::header::HeaderName;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use opentelemetry::trace::{Status, TraceContextExt, Tracer};
use opentelemetry::{global, Context};

use crate::services::forwarding_bot::{ForwardingBotService, IForwardingBotService};
use crate::services::forwarding_message::{ForwardingMessageService, IForwardingMessageService};

static TELEGRAM_BOT_API_SECRET_TOKEN: &[u8] = b"X-Telegram-Bot-Api-Secret-Token";

pub async fn forwarding_bot_update_handler(
    req: HttpRequest,
    update: web::Json<teloxide::types::Update>,
    token: web::Path<String>,
    forwarding_bot_service: web::Data<ForwardingBotService>,
    forwarding_message_service: web::Data<ForwardingMessageService>,
) -> impl Responder {
    log::debug!("Received update: {:?}", update);

    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/web");
    let cx = Context::current_with_span(
        tracer
            .span_builder("forwarding_bot_update_handler")
            .with_kind(opentelemetry::trace::SpanKind::Server)
            .start(&tracer),
    );

    let secret = match req
        .headers()
        .get(HeaderName::from_bytes(TELEGRAM_BOT_API_SECRET_TOKEN).expect("Invalid header name"))
    {
        Some(secret) => secret.to_str().unwrap_or_default(),
        None => {
            cx.span().set_status(Status::Error {
                description: Cow::from("Missing secret token"),
            });
            return HttpResponse::Unauthorized().body("Missing secret token");
        }
    };

    let bot_info = match forwarding_bot_service
        .get_bot_record_by_token(&cx, token.into_inner())
        .await
    {
        Ok(bot_info) => bot_info,
        Err(err) => {
            cx.span().record_error(err.as_ref());
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    if bot_info.bot_webhook_secret != secret {
        cx.span().set_status(Status::Error {
            description: Cow::from("Invalid secret token"),
        });
        return HttpResponse::Unauthorized().body("Invalid secret token");
    }

    match forwarding_message_service
        .handle_update_income(&cx, bot_info.id, update.into_inner())
        .await
    {
        Ok(_) => {
            cx.span().set_status(Status::Ok);
            HttpResponse::Ok().finish()
        }
        Err(err) => {
            cx.span().record_error(err.as_ref());
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}
