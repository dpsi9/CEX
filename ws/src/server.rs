use actix_web::{middleware::Logger, web, App, HttpServer};
use redis::RedisManager;
use shared::CexError;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tracing::info;

use crate::handlers;

#[derive(Clone)]
pub struct WsState {
    pub broadcaster: broadcast::Sender<String>,
}

pub async fn run(bind_addr: &str, redis_url: &str) -> Result<(), CexError> {
    let redis = RedisManager::new(redis_url).await?;
    let (tx, _rx) = broadcast::channel::<String>(512);
    spawn_redis_forwarder(redis, tx.clone());

    info!(%bind_addr, "starting ws server");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(WsState {
                broadcaster: tx.clone(),
            }))
            .wrap(Logger::default())
            .service(handlers::ws_upgrade)
    })
    .bind(bind_addr)
    .map_err(|e| CexError::Internal(format!("bind error: {e}")))?
    .run()
    .await
    .map_err(|e| CexError::Internal(format!("server error: {e}")))
}

fn spawn_redis_forwarder(manager: RedisManager, broadcaster: broadcast::Sender<String>) {
    tokio::spawn(async move {
        match manager.subscribe_events().await {
            Ok(sub) => {
                let mut stream = redis::manager::subscriber_stream(sub);
                while let Some(msg) = stream.next().await {
                    match msg {
                        Ok(payload) => {
                            let _ = broadcaster.send(payload);
                        }
                        Err(err) => {
                            tracing::error!("redis subscriber error: {err}");
                            break;
                        }
                    }
                }
            }
            Err(err) => tracing::error!("failed to subscribe to events: {err}"),
        }
    });
}
