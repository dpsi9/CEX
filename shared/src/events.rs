use crate::types::{NewOrder, Trade};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    OrderNew(NewOrder),
    OrderCancel {
        order_id: Uuid,
    },
    TradeExecuted(Trade),
    DepthSnapshot {
        pair: String,
        bids: Vec<(Decimal, Decimal)>,
        asks: Vec<(Decimal, Decimal)>,
        ts: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub version: u8,
    pub source: String,
    pub event: Event,
    pub emitted_at: DateTime<Utc>,
}

impl Envelope {
    pub fn new(source: impl Into<String>, event: Event) -> Self {
        Self {
            version: 1,
            source: source.into(),
            event,
            emitted_at: Utc::now(),
        }
    }
}
