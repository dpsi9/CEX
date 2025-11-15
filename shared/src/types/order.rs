use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
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
    pub filled: Decimal,
    pub status: OrderStatus,
    pub created_at: i64,
}

impl Order {
    pub fn remaining(&self) -> Decimal {
        if self.quantity > self.filled {
            self.quantity - self.filled
        } else {
            Decimal::new(0, 0)
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self.status, OrderStatus::New | OrderStatus::PartiallyFilled)
    }

    pub fn create_order(user_id: String, order: CreateOrder) -> Self {
        Order {
            order_id: Uuid::new_v4().to_string(),
            user_id,
            order_type: order.order_type,
            price: order.price,
            quantity: order.quantity,
            side: order.side,
            pair: order.pair,
            filled: Decimal::new(0, 0),
            status: OrderStatus::New,
            created_at: Utc::now().timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrder {
    pub pair: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrder {
    pub order_id: String,
    pub user_id: String,
    pub pair: String,
}
