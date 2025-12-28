use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod};
use std::time::Duration;
use tokio_postgres::NoTls;

pub type DbPool = Pool;

/// Initialize database connection pool from environment
pub fn create_pool() -> Result<DbPool, Box<dyn std::error::Error>> {
    let db_url = std::env::var("DB_URL").map_err(|_| "DB_URL environment variable not set")?;

    let config = db_url.parse::<tokio_postgres::Config>()?;

    let mut pool_config = Config::new();
    pool_config.dbname = config.get_dbname().map(String::from);
    pool_config.host = config.get_hosts().first().map(|h| match h {
        tokio_postgres::config::Host::Tcp(s) => s.clone(),
        #[cfg(unix)]
        tokio_postgres::config::Host::Unix(p) => p.to_string_lossy().to_string(),
    });
    pool_config.port = config.get_ports().first().copied();
    pool_config.user = config.get_user().map(String::from);
    pool_config.password = config.get_password().map(|p| String::from_utf8_lossy(p).to_string());

    let mgr_config = ManagerConfig { recycling_method: RecyclingMethod::Fast };
    let mgr = Manager::from_config(config, NoTls, mgr_config);

    Ok(Pool::builder(mgr).max_size(16).build()?)
}

/// Retry wrapper for getting database connections with exponential backoff
pub async fn get_connection_with_retry(
    pool: &DbPool, max_retries: u32,
) -> Result<deadpool_postgres::Object, deadpool_postgres::PoolError> {
    let mut attempts = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        match pool.get().await {
            Ok(conn) => return Ok(conn),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                tracing::warn!(
                    "Failed to get database connection (attempt {}/{}): {}. Retrying in {:?}...",
                    attempts,
                    max_retries,
                    e,
                    delay
                );
                tokio::time::sleep(delay).await;
                delay = delay.saturating_mul(2).min(Duration::from_secs(5));
            }
            Err(e) => {
                tracing::error!("Failed to get database connection after {} attempts: {}", attempts, e);
                return Err(e);
            }
        }
    }
}
