use teloxide::Bot;

use crate::settings::TelegramBot;

pub mod channel;
pub mod state;

pub fn new_bot(settings: &TelegramBot) -> Bot {
    let api_url = settings
        .api_url
        .clone()
        .unwrap_or("https://api.telegram.org".into());

    Bot::new(settings.token.clone()).set_api_url(reqwest::Url::parse(&api_url).unwrap())
}
