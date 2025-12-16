#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let bind = std::env::var("API_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    api::run(&bind, &redis_url)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}
