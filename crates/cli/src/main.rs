use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use tokio_postgres::NoTls;

#[derive(Parser)]
#[command(name = "malfestio")]
#[command(author = "Author <author@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "Malfestio CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the backend server
    Start,
    /// Run database migrations
    Migrate {
        /// Database URL (defaults to DB_URL env var)
        #[arg(long)]
        db_url: Option<String>,
    },
}

#[tokio::main]
async fn main() -> malfestio_core::Result<()> {
    let _ = dotenvy::from_filename(".env.local");
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => {
            malfestio_server::start().await?;
        }
        Commands::Migrate { db_url } => {
            run_migrations(db_url.as_deref()).await?;
        }
    }

    Ok(())
}

async fn run_migrations(db_url: Option<&str>) -> malfestio_core::Result<()> {
    let db_url = db_url
        .map(String::from)
        .or_else(|| std::env::var("DB_URL").ok())
        .ok_or_else(|| {
            malfestio_core::Error::InvalidArgument("DB_URL not provided via --db-url or DB_URL env var".to_string())
        })?;

    println!("Connecting to database...");
    let (mut client, connection) = tokio_postgres::connect(&db_url, NoTls)
        .await
        .map_err(|e| malfestio_core::Error::Database(format!("Failed to connect to database: {}", e)))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    println!("Connected to database");

    client
        .execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                id SERIAL PRIMARY KEY,
                version TEXT NOT NULL UNIQUE,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            &[],
        )
        .await
        .map_err(|e| malfestio_core::Error::Database(format!("Failed to create migrations table: {}", e)))?;

    let migrations_dir = Path::new("migrations");
    if !migrations_dir.exists() {
        return Err(malfestio_core::Error::InvalidArgument(
            "migrations directory not found".to_string(),
        ));
    }

    let mut entries: Vec<_> = fs::read_dir(migrations_dir)
        .map_err(|e| malfestio_core::Error::Other(format!("Failed to read migrations directory: {}", e)))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "sql")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    println!("Found {} migration files", entries.len());

    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();
        let version = filename.trim_end_matches(".sql");

        let row = client
            .query_opt("SELECT version FROM schema_migrations WHERE version = $1", &[&version])
            .await
            .map_err(|e| malfestio_core::Error::Database(format!("Failed to check migration status: {}", e)))?;

        if row.is_some() {
            println!("Skipping {}: already applied", filename);
            continue;
        }

        println!("Applying {}...", filename);

        let sql = fs::read_to_string(&path)
            .map_err(|e| malfestio_core::Error::Other(format!("Failed to read migration file: {}", e)))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| malfestio_core::Error::Database(format!("Failed to start transaction: {}", e)))?;

        tx.batch_execute(&sql)
            .await
            .map_err(|e| malfestio_core::Error::Database(format!("Failed to execute migration {}: {}", filename, e)))?;

        tx.execute("INSERT INTO schema_migrations (version) VALUES ($1)", &[&version])
            .await
            .map_err(|e| malfestio_core::Error::Database(format!("Failed to record migration: {}", e)))?;

        tx.commit()
            .await
            .map_err(|e| malfestio_core::Error::Database(format!("Failed to commit migration: {}", e)))?;

        println!("Applied {}", filename);
    }

    println!("All migrations complete!");

    Ok(())
}
