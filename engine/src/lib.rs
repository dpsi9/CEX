pub mod orderbook;
pub mod processor;

pub use processor::Engine;

/// Convenience entry point used by the binary.
pub async fn run(redis_url: &str) -> Result<(), shared::CexError> {
    let mut engine = Engine::new(redis_url).await?;
    engine.run().await
}
