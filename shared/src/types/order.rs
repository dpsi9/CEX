use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub user_id: String,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub side: Side,
    pub pair: String,
    pub filled_quantity: Decimal,
    pub created_at: i64,
}

impl Order {
    pub fn new_limit(
        user_id: String,
        pair: impl Into<String>,
        side: Side,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        Self {
            order_id: Uuid::new_v4().to_string(),
            user_id,
            order_type: OrderType::Limit,
            price,
            quantity,
            side,
            pair: pair.into(),
            filled_quantity: Decimal::new(0, 0),
            created_at: Utc::now().timestamp(),
        }
    }

    pub fn new_market(
        user_id: String,
        pair: impl Into<String>,
        side: Side,
        quantity: Decimal,
    ) -> Self {
        Self {
            order_id: Uuid::new_v4().to_string(),
            user_id,
            order_type: OrderType::Market,
            price: Decimal::new(0, 0),
            quantity,
            side,
            pair: pair.into(),
            filled_quantity: Decimal::new(0, 0),
            created_at: Utc::now().timestamp(),
        }
    }
}
