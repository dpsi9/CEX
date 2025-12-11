use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: Uuid,
    pub pair: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl Trade {
    pub fn new(
        pair: impl Into<String>,
        price: Decimal,
        quantity: Decimal,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
    ) -> Self {
        Self {
            trade_id: Uuid::new_v4(),
            pair: pair.into(),
            price,
            quantity,
            buy_order_id,
            sell_order_id,
            timestamp: Utc::now(),
        }
    }
}
