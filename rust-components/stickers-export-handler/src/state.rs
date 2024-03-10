use serde::{Deserialize, Serialize};
use teloxide::dispatching::dialogue::{serializer, RedisStorage};
use teloxide::prelude::Dialogue;

pub(crate) type AppDialogue = Dialogue<State, RedisStorage<serializer::Json>>;

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) enum State {
    #[default]
    Start,
    ReceiveSticker,
}
