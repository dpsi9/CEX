use crate::types::{Order, Trade};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    OrderNew(Order),
    OrderCancel {
        order_id: String,
    },
    TradeExecuted(Trade),
    DepthSnapshot {
        pair: String,
        bids: Vec<(Decimal, Decimal)>,
        asks: Vec<(Decimal, Decimal)>,
        ts: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub version: u8,
    pub source: String,
    pub event: Event,
    pub emitted_at: i64,
}

impl Envelope {
    pub fn new(source: impl Into<String>, event: Event) -> Self {
        Self {
            version: 1,
            source: source.into(),
            event,
            emitted_at: Utc::now().timestamp(),
        }
    }
}
