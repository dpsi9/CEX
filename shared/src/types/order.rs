use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type OrderId = Uuid;
pub type UserId = Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrder {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrder {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub pair: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialFill {
    pub order_id: OrderId,
    pub filled_qty: Decimal,
    pub price: Decimal,
    pub remaining_qty: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled: Decimal,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
}

impl Order {
    pub fn remaining(&self) -> Decimal {
        if self.quantity > self.filled {
            self.quantity - self.filled
        } else {
            Decimal::ZERO
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self.status, OrderStatus::New | OrderStatus::PartiallyFilled)
    }

    pub fn from_new(new: NewOrder) -> Self {
        Self {
            order_id: new.order_id,
            user_id: new.user_id,
            pair: new.pair,
            side: new.side,
            order_type: new.order_type,
            price: new.price,
            quantity: new.quantity,
            filled: Decimal::ZERO,
            status: OrderStatus::New,
            created_at: new.created_at,
        }
    }
}

pub fn new_order(
    user_id: UserId,
    pair: String,
    side: OrderSide,
    order_type: OrderType,
    price: Decimal,
    quantity: Decimal,
) -> NewOrder {
    NewOrder {
        order_id: Uuid::new_v4(),
        user_id,
        pair,
        side,
        order_type,
        price,
        quantity,
        created_at: Utc::now(),
    }
}
