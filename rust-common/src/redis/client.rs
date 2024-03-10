use crate::redis::utils::parse_redis_settings;
use crate::settings::Settings;

pub fn new_client(settings: &Settings) -> redis::Client {
    let url = parse_redis_settings(settings);

    redis::Client::open(url).unwrap()
}
