#[tokio::main]
async fn main() -> malfestio_core::Result<()> {
    dotenvy::dotenv().ok();
    if let Ok(file) = std::env::var("ENV_FILE") {
        dotenvy::from_filename(file).ok();
    }

    malfestio_server::start().await
}
