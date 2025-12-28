#[tokio::main]
async fn main() -> malfestio_core::Result<()> {
    malfestio_server::start().await
}
