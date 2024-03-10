use std::sync::Arc;

use teloxide::dispatching::dialogue::{RedisStorage, serializer};

use crate::redis::utils::parse_redis_settings;
use crate::settings::Settings;

async fn new_state_storage(settings: &Settings) -> Arc<RedisStorage<serializer::Json>> {
    let url = parse_redis_settings(settings);

    log::debug!("Connecting to Redis server: {}", url);

    RedisStorage::open(url, serializer::Json).await.unwrap()
}
