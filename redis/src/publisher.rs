use shared::CexError;

use crate::manager::RedisManager;
use crate::queues::{CHANNEL_EVENTS, QUEUE_ORDER_CANCEL, QUEUE_ORDER_NEW};

pub struct RedisPublisher<'a> {
    manager: &'a RedisManager,
}

impl<'a> RedisPublisher<'a> {
    pub fn new(manager: &'a RedisManager) -> Self {
        Self { manager }
    }

    pub async fn publish_event(&self, payload: &str) -> Result<(), CexError> {
        self.manager.publish(CHANNEL_EVENTS, payload).await
    }

    pub async fn enqueue_new_order(&self, payload: &str) -> Result<(), CexError> {
        self.manager.push(QUEUE_ORDER_NEW, payload).await
    }

    pub async fn enqueue_cancel_order(&self, payload: &str) -> Result<(), CexError> {
        self.manager.push(QUEUE_ORDER_CANCEL, payload).await
    }
}
