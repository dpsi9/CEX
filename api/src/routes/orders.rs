use actix_web::{post, web, HttpResponse, Responder};
use redis::queues::{QUEUE_ORDER_CANCEL, QUEUE_ORDER_NEW};
use redis::RedisManager;
use serde::Deserialize;
use shared::types::{new_order as build_new_order, CancelOrder, OrderSide, OrderType};
use shared::{to_json, CexError, Envelope, Event};
use uuid::Uuid;

use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct NewOrderRequest {
    pub user_id: Uuid,
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: rust_decimal::Decimal,
    pub quantity: rust_decimal::Decimal,
}

#[post("/order/new")]
pub async fn new_order_route(
    state: web::Data<AppState>,
    payload: web::Json<NewOrderRequest>,
) -> impl Responder {
    match handle_new_order(state.redis.clone(), payload.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => error_response(err),
    }
}

async fn handle_new_order(
    redis: std::sync::Arc<RedisManager>,
    req: NewOrderRequest,
) -> Result<(), CexError> {
    let order = build_new_order(
        req.user_id,
        req.pair,
        req.side,
        req.order_type,
        req.price,
        req.quantity,
    );
    let envelope = Envelope::new("api", Event::OrderNew(order));
    let body = to_json(&envelope)?;
    redis.push(QUEUE_ORDER_NEW, &body).await
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
    pub user_id: Uuid,
    pub order_id: Uuid,
    pub pair: String,
}

#[post("/order/cancel")]
pub async fn cancel_order_route(
    state: web::Data<AppState>,
    payload: web::Json<CancelOrderRequest>,
) -> impl Responder {
    match handle_cancel_order(state.redis.clone(), payload.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => error_response(err),
    }
}

async fn handle_cancel_order(
    redis: std::sync::Arc<RedisManager>,
    req: CancelOrderRequest,
) -> Result<(), CexError> {
    let cancel = CancelOrder {
        order_id: req.order_id,
        user_id: req.user_id,
        pair: req.pair,
    };
    let envelope = Envelope::new(
        "api",
        Event::OrderCancel {
            order_id: cancel.order_id,
        },
    );
    let body = to_json(&envelope)?;
    redis.push(QUEUE_ORDER_CANCEL, &body).await
}

fn error_response(err: CexError) -> HttpResponse {
    match err {
        CexError::Validation(msg) => HttpResponse::BadRequest().body(msg),
        _ => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
