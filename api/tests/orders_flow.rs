use actix_web::{test, App};
use api::routes;
use redis::RedisManager;
use shared::types::{OrderSide, OrderType};
use serde_json::json;
use uuid::Uuid;

#[actix_rt::test]
async fn enqueue_new_order_returns_200() {
    // Requires a running Redis at REDIS_URL; skip gracefully if not available
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let redis = match RedisManager::new(&redis_url).await {
        Ok(r) => r,
        Err(_) => {
            eprintln!("skipping: redis not available at {redis_url}");
            return;
        }
    };
    let state = api::server::AppState {
        redis: std::sync::Arc::new(redis),
    };

    let app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let payload = json!({
        "user_id": Uuid::new_v4(),
        "pair": "SOLUSDC",
        "side": OrderSide::Buy,
        "order_type": OrderType::Limit,
        "price": "10.0",
        "quantity": "1.0"
    });

    let req = test::TestRequest::post()
        .uri("/order/new")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
