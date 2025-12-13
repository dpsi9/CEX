use std::path::Path;
use std::str::FromStr;

use db::{insert_event, migrate, Db};
use engine::Engine;
use redis::queues::QUEUE_ORDER_NEW;
use redis::RedisManager;
use rust_decimal::Decimal;
use shared::types::{new_order, OrderSide, OrderType};
use shared::{to_json, CexError, Envelope, Event};
use testcontainers::clients::Cli;
use testcontainers::images::generic::GenericImage;
use tokio::time::{timeout, Duration};
use tokio_stream::StreamExt;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn end_to_end_trade_persists_events() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("SKIP_E2E").is_ok() {
        eprintln!("skipping e2e: SKIP_E2E set");
        return Ok(());
    }

    if !Path::new("/var/run/docker.sock").exists() {
        eprintln!("skipping e2e: docker socket not found");
        return Ok(());
    }

    // Spin up isolated Redis + Postgres via testcontainers (requires Docker running)
    let docker = Cli::default();

    let redis_image = GenericImage::new("redis", "7-alpine");
    let redis_node = docker.run(redis_image);
    let redis_port = redis_node.get_host_port_ipv4(6379);
    let redis_url = format!("redis://127.0.0.1:{redis_port}/");

    let pg_image = GenericImage::new("postgres", "16-alpine")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_DB", "postgres");
    let pg_node = docker.run(pg_image);
    let pg_port = pg_node.get_host_port_ipv4(5432);
    let db_url = format!("postgres://postgres:postgres@127.0.0.1:{pg_port}/postgres");

    // Give services a brief moment to accept connections
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Prepare DB (migrations) and start a lightweight event consumer (db_filler-like)
    let db = Db::new(&db_url, 5).await?;
    migrate(db.pool()).await?;
    let pool = db.pool().clone();
    let pool_for_consumer = pool.clone();

    let consumer_handle = tokio::spawn({
        let redis_url = redis_url.clone();
        async move {
            let redis = RedisManager::new(&redis_url).await?;
            let sub = redis.subscribe_events().await?;
            let mut stream = redis::manager::subscriber_stream(sub);
            let mut inserted = 0usize;
            while inserted < 3 {
                let next = timeout(Duration::from_secs(15), stream.next()).await;
                match next {
                    Ok(Some(Ok(payload))) => {
                        insert_event(&pool_for_consumer, &payload).await?;
                        inserted += 1;
                    }
                    Ok(Some(Err(err))) => return Err(err),
                    _ => break,
                }
            }
            Ok::<_, CexError>(())
        }
    });

    // Start engine loop in background
    let mut engine = Engine::new(&redis_url).await?;
    let engine_handle = tokio::spawn(async move {
        let _ = engine.run().await;
    });

    // Enqueue two crossing limit orders
    let redis_push = RedisManager::new(&redis_url).await?;
    let buy = new_order(
        Uuid::new_v4(),
        "SOLUSDC".to_string(),
        OrderSide::Buy,
        OrderType::Limit,
        Decimal::from_str("31.0").unwrap(),
        Decimal::from_str("5").unwrap(),
    );
    let sell = new_order(
        Uuid::new_v4(),
        "SOLUSDC".to_string(),
        OrderSide::Sell,
        OrderType::Limit,
        Decimal::from_str("31.0").unwrap(),
        Decimal::from_str("5").unwrap(),
    );

    for evt in [Event::OrderNew(buy), Event::OrderNew(sell)] {
        let env = Envelope::new("e2e", evt);
        let body = to_json(&env)?;
        redis_push.push(QUEUE_ORDER_NEW, &body).await?;
    }

    // Wait for consumer to persist events or timeout
    let consumer_result = timeout(Duration::from_secs(30), consumer_handle).await;

    // Stop the engine loop
    engine_handle.abort();

    match consumer_result {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(err))) => panic!("consumer error: {err}"),
        Ok(Err(join_err)) => panic!("consumer join error: {join_err}"),
        Err(_) => panic!("consumer timed out"),
    }

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM events")
        .fetch_one(&pool)
        .await?;

    assert!(
        count >= 2,
        "expected at least trade/depth events persisted, got {count}"
    );

    Ok(())
}
