use std::time::Duration;

use fast_qr::convert::Builder;
use fastping_rs::PingResult::{Idle, Receive};
use fastping_rs::Pinger;
use moka::future::Cache;
use opentelemetry::trace::{Span, SpanKind, Tracer};
use opentelemetry::{global, KeyValue};
use teloxide::prelude::*;
use teloxide::types::InputFile;
use teloxide::utils::command::BotCommands;

use crate::utils::parse_target;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub(crate) enum BotCommand {
    #[command(description = "Generate QRCode", rename = "qrcode")]
    QRCode(String),
    #[command(description = "Ping the target", rename = "ping")]
    Ping(String),
}

macro_rules! send_error_message {
    ($bot:expr, $message:expr, $error_msg:expr) => {
        $bot.send_message($message.chat.id, $error_msg)
            .reply_to_message_id($message.id)
            .send()
            .await?;
    };
}

macro_rules! match_error {
    ($data:expr, $bot:expr, $message:expr, $error_msg:expr) => {
        match $data {
            Ok(data) => data,
            Err(err) => {
                send_error_message!($bot, $message, format!($error_msg, err));
                return Err(anyhow::anyhow!($error_msg, err));
            }
        }
    };
}

pub(crate) async fn qrcode_handler(
    bot: Bot,
    update: Update,
    message: Message,
    text: String,
    cache: Cache<String, Vec<u8>>,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/network-functions-handler/handlers");
    let mut span = tracer
        .span_builder("qrcode_handler")
        .with_kind(SpanKind::Internal)
        .start_with_context(&tracer, &update.cx.unwrap_or_default());

    if text.is_empty() {
        let err = anyhow::anyhow!("Text is empty");
        send_error_message!(bot, message, "Text is empty");
        span.record_error(err.as_ref());
        return Err(err);
    }

    // Check if QRCode already exists in cache
    if let Some(data) = cache.get(&format!("qrcode:{}", &text).to_string()).await {
        log::debug!("QRCode cache hit");

        bot.send_photo(
            message.chat.id,
            InputFile::memory(data.to_vec()).file_name("qrcode.png"),
        )
        .reply_to_message_id(message.id)
        .send()
        .await?;

        span.set_attribute(KeyValue::new("cache_hit", true));

        return Ok(());
    }

    let qr_code = match_error!(
        fast_qr::QRBuilder::new(text.clone()).build(),
        bot,
        message,
        "Failed to build QRCode: {}"
    );

    let image = match_error!(
        fast_qr::convert::image::ImageBuilder::default()
            .shape(fast_qr::convert::Shape::Square)
            .background_color([255, 255, 255, 0])
            .fit_width(512)
            .to_bytes(&qr_code),
        bot,
        message,
        "Failed to convert QRCode to image: {}"
    );

    bot.send_photo(
        message.chat.id,
        InputFile::memory(image.clone()).file_name("qrcode.png"),
    )
    .reply_to_message_id(message.id)
    .send()
    .await?;

    cache
        .insert(format!("qrcode:{}", text).to_string(), image.to_vec())
        .await;

    Ok(())
}

pub(crate) async fn ping_handler(
    bot: Bot,
    update: Update,
    message: Message,
    target: String,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/network-functions-handler/handlers");
    let mut span = tracer
        .span_builder("ping_handler")
        .with_kind(SpanKind::Internal)
        .start_with_context(&tracer, &update.cx.unwrap_or_default());

    if target.is_empty() {
        let err = anyhow::anyhow!("Target is empty");
        send_error_message!(bot, message, "Usage: /ping <target>");
        span.record_error(err.as_ref());
        return Err(err);
    }

    let target_ip = match_error!(
        parse_target(&target).await,
        bot,
        message,
        "Failed to parse target: {}"
    );

    let (pinger, results) = match_error!(
        Pinger::new(None, Some(56)),
        bot,
        message,
        "Failed to create pinger: {}"
    );

    pinger.add_ipaddr(target_ip.to_string().as_str());
    pinger.ping_once();

    match results.recv_timeout(Duration::from_secs(10)) {
        Ok(result) => match result {
            Idle { addr } => {
                let err = format!("Failed to ping target: {}", addr);
                send_error_message!(bot, message, &err);
                span.record_error(anyhow::anyhow!("Failed to ping target: {}", addr).as_ref());
                return Err(anyhow::anyhow!("Failed to ping target: {}", err));
            }
            Receive { addr, rtt } => {
                bot.send_message(
                    message.chat.id,
                    format!("Sended 56 bytes to {} in {:.2}ms", addr, rtt.as_millis()),
                )
                .reply_to_message_id(message.id)
                .send()
                .await?;
            }
        },
        Err(e) => {
            let err = format!("Failed to receive result: {}", e);
            send_error_message!(bot, message, err.clone());
            span.record_error(anyhow::anyhow!(err.clone()).as_ref());
            return Err(anyhow::anyhow!(err));
        }
    }

    Ok(())
}
