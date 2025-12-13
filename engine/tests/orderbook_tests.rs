use engine::orderbook::OrderBook;
use rust_decimal::Decimal;
use shared::types::{new_order, Order, OrderSide, OrderType};
use uuid::Uuid;

fn mk_order(user: &str, side: OrderSide, price: &str, qty: &str) -> Order {
    let o = new_order(
        Uuid::parse_str(user).unwrap(),
        "SOLUSDC".to_string(),
        side,
        OrderType::Limit,
        Decimal::from_str_exact(price).unwrap(),
        Decimal::from_str_exact(qty).unwrap(),
    );
    Order::from_new(o)
}

#[test]
fn matches_limit_cross_and_clears_book() {
    let mut book = OrderBook::new("SOLUSDC");

    // Place resting ask
    let ask = mk_order(
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        OrderSide::Sell,
        "30.0",
        "5",
    );
    let (_trades, _fill) = book.upsert(ask);

    // Incoming aggressive buy crosses and should fully consume
    let buy = mk_order(
        "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        OrderSide::Buy,
        "30.0",
        "5",
    );
    let (trades, last_fill) = book.upsert(buy);

    assert_eq!(trades.len(), 1, "one trade produced");
    let t = &trades[0];
    assert_eq!(t.price.to_string(), "30.0");
    assert_eq!(t.quantity.to_string(), "5");

    // Book should be empty after full match
    let depth = book.depth();
    assert!(depth.bids.is_empty());
    assert!(depth.asks.is_empty());

    // Last fill reflects full fill of the incoming order
    let fill = last_fill.expect("fill present");
    assert_eq!(fill.remaining_qty, Decimal::ZERO);
}

#[test]
fn partially_fills_and_resting_remains() {
    let mut book = OrderBook::new("SOLUSDC");

    // Resting ask qty 10
    let ask = mk_order(
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        OrderSide::Sell,
        "30.0",
        "10",
    );
    book.upsert(ask);

    // Incoming buy qty 4 should partially fill ask, leaving 6
    let buy = mk_order(
        "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        OrderSide::Buy,
        "30.0",
        "4",
    );
    let (trades, _fill) = book.upsert(buy);

    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].quantity.to_string(), "4");

    let depth = book.depth();
    assert!(depth.bids.is_empty());
    assert_eq!(depth.asks.len(), 1);
    assert_eq!(depth.asks[0].quantity.to_string(), "6");
}

#[test]
fn cancel_removes_order() {
    let mut book = OrderBook::new("SOLUSDC");
    let ask = mk_order(
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        OrderSide::Sell,
        "30.0",
        "5",
    );
    let id = ask.order_id;
    book.upsert(ask);
    assert!(book.cancel(id));
    let depth = book.depth();
    assert!(depth.asks.is_empty());
}
