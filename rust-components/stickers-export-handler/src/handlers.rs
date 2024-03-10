use teloxide::prelude::*;

use crate::state::{AppDialogue, State};

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub(crate) async fn start(bot: Bot, dialogue: AppDialogue, msg: Message) -> HandlerResult {
    if let None = msg.text() {
        log::debug!("Skipping non-text message");
        return Ok(());
    }

    if let Some(text) = msg.text() {
        if text != "/export_sticker" {
            log::debug!("Skipping non-start message");
            return Ok(());
        }
    }

    bot.send_message(msg.chat.id, "Please send me a sticker")
        .send()
        .await?;

    dialogue.update(State::ReceiveSticker).await?;

    Ok(())
}
