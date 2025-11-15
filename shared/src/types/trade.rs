use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: String,
    pub pair: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub buy_order_id: String,
    pub sell_order_id: String,
    pub timestamp: i64,
}

impl Trade {
    pub fn new(
        pair: impl Into<String>,
        price: Decimal,
        quantity: Decimal,
        buy_order_id: String,
        sell_order_id: String,
    ) -> Self {
        Self {
            trade_id: Uuid::new_v4().to_string(),
            pair: pair.into(),
            price,
            quantity,
            buy_order_id,
            sell_order_id,
            timestamp: Utc::now().timestamp(),
        }
    }
}
