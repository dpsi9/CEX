use actix_web::web;

pub mod health;
pub mod markets;
pub mod orders;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(orders::new_order_route)
        .service(orders::cancel_order_route)
        .service(markets::list_markets)
        .service(health::health);
}
