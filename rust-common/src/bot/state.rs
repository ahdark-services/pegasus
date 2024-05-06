use std::fmt::Debug;
use std::ops::DerefMut;
use std::sync::Arc;

use futures::future::BoxFuture;
use redis::AsyncCommands;
use serde::de::DeserializeOwned;
use serde::Serialize;
use teloxide::dispatching::dialogue::Storage;
use teloxide::prelude::ChatId;
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Dialogue not found")]
    DialogueNotFound,
}

#[derive(Debug)]
pub struct RedisStorage {
    conn: Mutex<redis::aio::MultiplexedConnection>,
    service_name: String,
}

impl RedisStorage {
    pub async fn open(conn: redis::aio::MultiplexedConnection, service_name: String) -> Arc<Self> {
        Arc::new(Self {
            conn: Mutex::new(conn),
            service_name,
        })
    }
}

impl<D> Storage<D> for RedisStorage
where
    D: Send + Serialize + DeserializeOwned + 'static,
{
    type Error = StorageError;

    fn remove_dialogue(
        self: Arc<Self>,
        ChatId(chat_id): ChatId,
    ) -> BoxFuture<'static, Result<(), Self::Error>> {
        Box::pin(async move {
            let deleted_rows_count = redis::pipe()
                .atomic()
                .del(format!("{}-{}", &self.service_name, chat_id))
                .query_async::<_, redis::Value>(self.conn.lock().await.deref_mut())
                .await?;

            if let redis::Value::Bulk(values) = deleted_rows_count {
                if let redis::Value::Int(deleted_rows_count) = values[0] {
                    return match deleted_rows_count {
                        0 => Err(StorageError::DialogueNotFound),
                        _ => Ok(()),
                    };
                }
            }

            unreachable!("Must return redis::Value::Bulk(redis::Value::Int(_))");
        })
    }

    fn update_dialogue(
        self: Arc<Self>,
        ChatId(chat_id): ChatId,
        dialogue: D,
    ) -> BoxFuture<'static, Result<(), Self::Error>> {
        Box::pin(async move {
            let dialogue = serde_json::to_vec(&dialogue)?;
            self.conn
                .lock()
                .await
                .set::<_, Vec<u8>, _>(format!("{}-{}", &self.service_name, chat_id), dialogue)
                .await?;
            Ok(())
        })
    }

    fn get_dialogue(
        self: Arc<Self>,
        ChatId(chat_id): ChatId,
    ) -> BoxFuture<'static, Result<Option<D>, Self::Error>> {
        Box::pin(async move {
            self.conn
                .lock()
                .await
                .get::<_, Option<Vec<u8>>>(format!("{}-{}", &self.service_name, chat_id))
                .await?
                .map(|d| Ok(serde_json::from_slice(&d)?))
                .transpose()
        })
    }
}

pub async fn new_state_storage(
    service_name: &str,
    conn: redis::aio::MultiplexedConnection,
) -> Arc<RedisStorage> {
    RedisStorage::open(conn, service_name.to_string()).await
}
