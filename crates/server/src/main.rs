#[tokio::main]
async fn main() -> malfestio_core::Result<()> {
    // TODO: default to .env, pass arg/param into call
    dotenvy::from_filename(".env.local").ok();
    malfestio_server::start().await
}
