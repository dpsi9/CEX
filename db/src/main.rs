#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("db crate is library-focused; no standalone runtime.");
    Ok(())
}
