use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct MarketInfo {
    pub pair: String,
    pub base: String,
    pub quote: String,
    pub tick_size: String,
    pub lot_size: String,
}

#[get("/markets")]
pub async fn list_markets() -> impl Responder {
    let markets = vec![MarketInfo {
        pair: "SOLUSDC".to_string(),
        base: "SOL".to_string(),
        quote: "USDC".to_string(),
        tick_size: "0.0001".to_string(),
        lot_size: "0.001".to_string(),
    }];
    HttpResponse::Ok().json(markets)
}
