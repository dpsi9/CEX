use redis_rs::aio::{MultiplexedConnection, PubSub};
use redis_rs::AsyncCommands;
use shared::CexError;

use crate::queues::{CHANNEL_EVENTS, QUEUE_ORDER_CANCEL, QUEUE_ORDER_NEW};
use crate::subscriber::RedisSubscriber;

pub struct RedisManager {
    client: redis_rs::Client,
}

impl RedisManager {
    pub async fn new(url: &str) -> Result<Self, CexError> {
        let client = redis_rs::Client::open(url)
            .map_err(|e| CexError::Redis(format!("failed to create client: {e}")))?;
        Ok(Self { client })
    }

    async fn connection(&self) -> Result<MultiplexedConnection, CexError> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| CexError::Redis(format!("connection error: {e}")))
    }

    pub async fn push(&self, queue: &str, payload: &str) -> Result<(), CexError> {
        let mut conn = self.connection().await?;
        conn.rpush(queue, payload)
            .await
            .map_err(|e| CexError::Redis(format!("rpush failed: {e}")))
            .map(|_: i64| ())
    }

    pub async fn pop(&self, queue: &str, timeout_secs: u64) -> Result<Option<String>, CexError> {
        let mut conn = self.connection().await?;
        let result: Option<(String, String)> = conn
            .brpop(queue, timeout_secs as f64)
            .await
            .map_err(|e| CexError::Redis(format!("brpop failed: {e}")))?;
        Ok(result.map(|(_, payload)| payload))
    }

    pub async fn publish(&self, channel: &str, payload: &str) -> Result<(), CexError> {
        let mut conn = self.connection().await?;
        conn.publish(channel, payload)
            .await
            .map_err(|e| CexError::Redis(format!("publish failed: {e}")))
            .map(|_: i64| ())
    }

    async fn pubsub(&self) -> Result<PubSub, CexError> {
        self.client
            .get_async_pubsub()
            .await
            .map_err(|e| CexError::Redis(format!("pubsub connection error: {e}")))
    }

    pub async fn subscribe(&self, channel: &str) -> Result<RedisSubscriber, CexError> {
        let mut pubsub = self.pubsub().await?;
        pubsub
            .subscribe(channel)
            .await
            .map_err(|e| CexError::Redis(format!("subscribe failed: {e}")))?;
        Ok(RedisSubscriber::new(pubsub))
    }

    pub async fn subscribe_events(&self) -> Result<RedisSubscriber, CexError> {
        self.subscribe(CHANNEL_EVENTS).await
    }

    pub async fn push_new_order(&self, payload: &str) -> Result<(), CexError> {
        self.push(QUEUE_ORDER_NEW, payload).await
    }

    pub async fn push_cancel_order(&self, payload: &str) -> Result<(), CexError> {
        self.push(QUEUE_ORDER_CANCEL, payload).await
    }

    pub async fn pop_new_order(&self, timeout_secs: u64) -> Result<Option<String>, CexError> {
        self.pop(QUEUE_ORDER_NEW, timeout_secs).await
    }

    pub async fn pop_cancel_order(&self, timeout_secs: u64) -> Result<Option<String>, CexError> {
        self.pop(QUEUE_ORDER_CANCEL, timeout_secs).await
    }
}

/// Helper to convert a `RedisSubscriber` into a payload stream.
pub fn subscriber_stream(
    sub: RedisSubscriber,
) -> tokio_stream::wrappers::ReceiverStream<Result<String, CexError>> {
    tokio_stream::wrappers::ReceiverStream::new(sub.into_channel())
}
