use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod};
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
