use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use redis::RedisManager;
use shared::CexError;
use std::sync::Arc;
use tracing::info;

use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub redis: Arc<RedisManager>,
}

pub async fn run(bind_addr: &str, redis_url: &str) -> Result<(), CexError> {
    let redis = Arc::new(RedisManager::new(redis_url).await?);

    info!(%bind_addr, "starting api server");
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(AppState {
                redis: redis.clone(),
            }))
            .wrap(Logger::default())
            .wrap(cors)
            .configure(routes::configure)
    })
    .bind(bind_addr)
    .map_err(|e| CexError::Internal(format!("failed to bind: {e}")))?
    .run()
    .await
    .map_err(|e| CexError::Internal(format!("server error: {e}")))
}
