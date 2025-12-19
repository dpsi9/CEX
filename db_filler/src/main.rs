use db::migrate;
use db::{insert_event, Db};
use redis::RedisManager;
use shared::from_json;
use shared::{CexError, Envelope};
use tokio_stream::StreamExt;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/cex".to_string());

    let redis = RedisManager::new(&redis_url).await.map_err(map_err_box)?;
    let db = Db::new(&db_url, 5).await.map_err(map_err_box)?;

    migrate(db.pool()).await.map_err(map_err_box)?;

    info!("db_filler started");

    if let Ok(sub) = redis.subscribe_events().await {
        let mut stream = redis::manager::subscriber_stream(sub);
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(payload) => {
                    if let Err(err) = handle_payload(&db, &payload).await {
                        error!("failed to persist event: {err}");
                    }
                }
                Err(err) => error!("redis error: {err}"),
            }
        }
    }

    Ok(())
}

async fn handle_payload(db: &Db, payload: &str) -> Result<(), CexError> {
    // Optionally ensure payload is valid JSON Envelope before writing
    let _: Envelope = from_json(payload)?;
    insert_event(db.pool(), payload).await
}

fn map_err_box(err: CexError) -> Box<dyn std::error::Error> {
    Box::new(err)
}
