use actix_web::http::header::HeaderName;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use opentelemetry::Context;

use crate::services::forwarding_bot::{ForwardingBotService, IForwardingBotService};
use crate::services::forwarding_message::{ForwardingMessageService, IForwardingMessageService};

static TELEGRAM_BOT_API_SECRET_TOKEN: &[u8] = b"X-Telegram-Bot-Api-Secret-Token";

#[post("/webhook/{token}")]
#[tracing::instrument]
pub async fn forwarding_bot_update_handler(
    req: HttpRequest,
    update: web::Json<teloxide::types::Update>,
    token: web::Path<String>,
    forwarding_bot_service: web::Data<ForwardingBotService>,
    forwarding_message_service: web::Data<ForwardingMessageService>,
) -> impl Responder {
    log::debug!("Received update: {:?}", update);

    let secret = match req
        .headers()
        .get(HeaderName::from_bytes(TELEGRAM_BOT_API_SECRET_TOKEN).expect("Invalid header name"))
    {
        Some(secret) => secret.to_str().unwrap_or_default(),
        None => {
            log::warn!("Missing secret token");
            return HttpResponse::Unauthorized().body("Missing secret token");
        }
    };

    let bot_info = match forwarding_bot_service
        .get_bot_record_by_token(token.into_inner())
        .await
    {
        Ok(bot_info) => bot_info,
        Err(err) => {
            log::error!("Error getting bot info: {}", err);
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    if bot_info.bot_webhook_secret != secret {
        return HttpResponse::Unauthorized().body("Invalid secret token");
    }

    match forwarding_message_service
        .handle_update_income(bot_info.id, update.into_inner())
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => {
            log::error!("Error handling update: {}", err);
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}
