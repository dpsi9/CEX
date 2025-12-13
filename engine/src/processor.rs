use std::collections::HashMap;

use redis::{queues::CHANNEL_EVENTS, RedisManager};
use shared::types::{NewOrder, Order};
use shared::{from_json, to_json, CexError, Envelope, Event};
use tracing::{error, info};

use crate::orderbook::OrderBook;

const ENGINE_SOURCE: &str = "engine";

pub struct Engine {
    redis: RedisManager,
    books: HashMap<String, OrderBook>,
}

impl Engine {
    pub async fn new(redis_url: &str) -> Result<Self, CexError> {
        let redis = RedisManager::new(redis_url).await?;
        Ok(Self {
            redis,
            books: HashMap::new(),
        })
    }

    pub async fn run(&mut self) -> Result<(), CexError> {
        loop {
            tokio::select! {
                new_msg = self.redis.pop_new_order(1) => {
                    if let Ok(Some(payload)) = new_msg {
                        if let Err(err) = self.handle_payload(payload).await {
                            error!("new order handling error: {err}");
                        }
                    }
                }
                cancel_msg = self.redis.pop_cancel_order(1) => {
                    if let Ok(Some(payload)) = cancel_msg {
                        if let Err(err) = self.handle_payload(payload).await {
                            error!("cancel order handling error: {err}");
                        }
                    }
                }
            }
        }
    }

    async fn handle_payload(&mut self, payload: String) -> Result<(), CexError> {
        let envelope: Envelope = from_json(&payload)?;
        match envelope.event {
            Event::OrderNew(new_order) => self.process_new_order(new_order).await?,
            Event::OrderCancel { order_id } => self.process_cancel(order_id).await?,
            _ => {
                info!("ignoring unsupported event from queue");
            }
        }
        Ok(())
    }

    async fn process_new_order(&mut self, new_order: NewOrder) -> Result<(), CexError> {
        let pair = new_order.pair.clone();
        let book = self
            .books
            .entry(pair.clone())
            .or_insert_with(|| OrderBook::new(pair.clone()));
        let (trades, depth) = {
            let order = Order::from_new(new_order);
            let (trades, _last_fill) = book.upsert(order);
            let depth = book.depth();
            (trades, depth)
        };

        for trade in trades {
            self.publish_event(Event::TradeExecuted(trade)).await?;
        }

        self.publish_event(Event::DepthSnapshot {
            pair: depth.pair.clone(),
            bids: depth
                .bids
                .iter()
                .map(|lvl| (lvl.price, lvl.quantity))
                .collect(),
            asks: depth
                .asks
                .iter()
                .map(|lvl| (lvl.price, lvl.quantity))
                .collect(),
            ts: depth.timestamp,
        })
        .await?;

        Ok(())
    }

    async fn process_cancel(&mut self, order_id: uuid::Uuid) -> Result<(), CexError> {
        let mut removed = false;
        for book in self.books.values_mut() {
            if book.cancel(order_id) {
                removed = true;
                break;
            }
        }
        if removed {
            self.publish_event(Event::OrderCancel { order_id }).await?;
        }
        Ok(())
    }

    async fn publish_event(&self, event: Event) -> Result<(), CexError> {
        let envelope = Envelope::new(ENGINE_SOURCE, event);
        let payload = to_json(&envelope)?;
        self.redis.publish(CHANNEL_EVENTS, &payload).await
    }
}
