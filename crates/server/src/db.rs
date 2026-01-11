use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod};
use std::time::Duration;
use tokio_postgres::NoTls;

pub type DbPool = Pool;

/// Initialize database connection pool from environment
pub fn create_pool(url: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    let config = url.parse::<tokio_postgres::Config>()?;

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

/// Create a mock pool for testing that won't actually connect
#[cfg(test)]
pub fn create_mock_pool() -> DbPool {
    let config = "host=localhost user=test dbname=test"
        .parse::<tokio_postgres::Config>()
        .unwrap();
    let mgr_config = ManagerConfig { recycling_method: RecyclingMethod::Fast };
    let mgr = Manager::from_config(config, NoTls, mgr_config);
    Pool::builder(mgr).max_size(1).build().unwrap()
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
                    error = %e,
                    attempt = attempts,
                    max_retries = max_retries,
                    retry_delay_ms = delay.as_millis(),
                    "Failed to get database connection, retrying"
                );
                tokio::time::sleep(delay).await;
                delay = delay.saturating_mul(2).min(Duration::from_secs(5));
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    attempts = attempts,
                    "Failed to get database connection after retries"
                );
                return Err(e);
            }
        }
    }
}
