use std::collections::{BTreeMap, HashMap, VecDeque};

use chrono::Utc;
use rust_decimal::Decimal;
use shared::types::{
    DepthLevel, DepthSnapshot, Order, OrderId, OrderSide, OrderStatus, OrderType, PartialFill,
    Trade,
};

pub struct OrderBook {
    pair: String,
    bids: BTreeMap<Decimal, VecDeque<Order>>, // highest price last when iterating ascending
    asks: BTreeMap<Decimal, VecDeque<Order>>, // lowest price first
    index: HashMap<OrderId, (OrderSide, Decimal)>,
}

impl OrderBook {
    pub fn new(pair: impl Into<String>) -> Self {
        Self {
            pair: pair.into(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            index: HashMap::new(),
        }
    }

    pub fn upsert(&mut self, mut order: Order) -> (Vec<Trade>, Option<PartialFill>) {
        let mut trades = Vec::new();
        let mut last_fill: Option<PartialFill> = None;

        // Match against the opposite side first
        loop {
            let best_price = match self.best_opposite_price(order.side) {
                Some(p) => p,
                None => break,
            };

            if !self.price_crosses(&order, best_price) {
                break;
            }

            let target_queue_opt = match order.side {
                OrderSide::Buy => self.asks.get_mut(&best_price),
                OrderSide::Sell => self.bids.get_mut(&best_price),
            };

            let target_queue = match target_queue_opt {
                Some(q) => q,
                None => break,
            };

            let maybe_resting = target_queue.pop_front();
            let mut resting = match maybe_resting {
                Some(r) => r,
                None => {
                    continue;
                }
            };

            let resting_remaining = resting.remaining();
            let incoming_remaining = order.remaining();
            if resting_remaining <= Decimal::ZERO || incoming_remaining <= Decimal::ZERO {
                continue;
            }

            let executed_qty = if resting_remaining < incoming_remaining {
                resting_remaining
            } else {
                incoming_remaining
            };

            resting.filled += executed_qty;
            order.filled += executed_qty;

            if resting.remaining() == Decimal::ZERO {
                resting.status = OrderStatus::Filled;
            } else {
                resting.status = OrderStatus::PartiallyFilled;
            }

            if order.remaining() == Decimal::ZERO {
                order.status = OrderStatus::Filled;
            } else {
                order.status = OrderStatus::PartiallyFilled;
            }

            let trade_price = best_price;
            let trade_qty = executed_qty;
            let trade = match order.side {
                OrderSide::Buy => Trade::new(
                    order.pair.clone(),
                    trade_price,
                    trade_qty,
                    order.order_id,
                    resting.order_id,
                ),
                OrderSide::Sell => Trade::new(
                    order.pair.clone(),
                    trade_price,
                    trade_qty,
                    resting.order_id,
                    order.order_id,
                ),
            };
            trades.push(trade);

            last_fill = Some(PartialFill {
                order_id: order.order_id,
                filled_qty: order.filled,
                price: trade_price,
                remaining_qty: order.remaining(),
            });

            if resting.remaining() > Decimal::ZERO {
                target_queue.push_front(resting);
            } else {
                self.index.remove(&resting.order_id);
            }

            if target_queue.is_empty() {
                match order.side {
                    OrderSide::Buy => {
                        self.asks.remove(&best_price);
                    }
                    OrderSide::Sell => {
                        self.bids.remove(&best_price);
                    }
                }
            }

            if order.remaining() == Decimal::ZERO {
                break;
            }
        }

        // If still open and limit order, place into book
        if order.remaining() > Decimal::ZERO && order.order_type == OrderType::Limit {
            self.enqueue(order.clone());
        }

        (trades, last_fill)
    }

    pub fn cancel(&mut self, order_id: OrderId) -> bool {
        let (side, price) = match self.index.get(&order_id).copied() {
            Some(v) => v,
            None => return false,
        };
        let levels = match side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };

        if let Some(queue) = levels.get_mut(&price) {
            let mut i = 0usize;
            while i < queue.len() {
                if queue[i].order_id == order_id {
                    queue.remove(i);
                    self.index.remove(&order_id);
                    if queue.is_empty() {
                        levels.remove(&price);
                    }
                    return true;
                }
                i += 1;
            }
        }

        false
    }

    pub fn depth(&self) -> DepthSnapshot {
        let bids = self
            .bids
            .iter()
            .rev()
            .map(|(p, orders)| DepthLevel {
                price: *p,
                quantity: orders
                    .iter()
                    .fold(Decimal::ZERO, |acc, o| acc + o.remaining()),
            })
            .collect();

        let asks = self
            .asks
            .iter()
            .map(|(p, orders)| DepthLevel {
                price: *p,
                quantity: orders
                    .iter()
                    .fold(Decimal::ZERO, |acc, o| acc + o.remaining()),
            })
            .collect();

        DepthSnapshot {
            pair: self.pair.clone(),
            bids,
            asks,
            timestamp: Utc::now(),
        }
    }

    fn enqueue(&mut self, order: Order) {
        let levels = match order.side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };
        let queue = levels.entry(order.price).or_insert_with(VecDeque::new);
        queue.push_back(order.clone());
        self.index
            .insert(order.order_id, (order.side.clone(), order.price));
    }

    fn best_opposite_price(&self, side: OrderSide) -> Option<Decimal> {
        match side {
            OrderSide::Buy => self.asks.keys().next().copied(),
            OrderSide::Sell => self.bids.keys().next_back().copied(),
        }
    }

    fn price_crosses(&self, incoming: &Order, best_price: Decimal) -> bool {
        match incoming.order_type {
            OrderType::Market => true,
            OrderType::Limit => match incoming.side {
                OrderSide::Buy => incoming.price >= best_price,
                OrderSide::Sell => incoming.price <= best_price,
            },
        }
    }
}
