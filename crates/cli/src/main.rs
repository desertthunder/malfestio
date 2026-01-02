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
    /// Check OAuth flow and database state for a Bluesky handle
    Check {
        /// Bluesky handle to test (e.g., alice.bsky.social)
        handle: String,
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
        Commands::Check { handle } => {
            check_flow(handle).await?;
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

async fn check_flow(handle: &str) -> malfestio_core::Result<()> {
    println!("Checking OAuth flow for {}...\n", handle);

    // Get database URL
    let db_url = std::env::var("DB_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .map_err(|_| malfestio_core::Error::InvalidArgument("DB_URL or DATABASE_URL not set".to_string()))?;

    // Test database connection
    print!("• Testing database connection... ");
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls)
        .await
        .map_err(|e| malfestio_core::Error::Database(format!("Failed to connect: {}", e)))?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    println!("✓ Connected");

    let resolver = malfestio_server::oauth::resolver::IdentityResolver::new();

    print!("• Resolving handle to DID... ");
    let did = match resolver.resolve_handle(handle).await {
        Ok(did) => {
            println!("✓ {}", did);
            did
        }
        Err(e) => {
            println!("✗ Failed: {}", e);
            return Err(malfestio_core::Error::Other(format!("Handle resolution failed: {}", e)));
        }
    };

    print!("• Resolving DID to PDS... ");
    let _resolved = match resolver.resolve_did(&did).await {
        Ok(resolved) => {
            println!("✓ {}", resolved.pds_url);
            resolved
        }
        Err(e) => {
            println!("✗ Failed: {}", e);
            return Err(malfestio_core::Error::Other(format!("DID resolution failed: {}", e)));
        }
    };

    print!("• Checking OAuth tokens... ");
    let token_row = client
        .query_opt(
            "SELECT did, pds_url, created_at, updated_at FROM oauth_tokens WHERE did = $1",
            &[&did],
        )
        .await
        .map_err(|e| malfestio_core::Error::Database(format!("Token query failed: {}", e)))?;

    if let Some(row) = token_row {
        let updated_at: chrono::DateTime<chrono::Utc> = row.get(3);
        println!("✓ Found (last updated: {})", updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
    } else {
        println!("✗ Not found");
        println!("\nℹ No OAuth tokens stored yet. Complete OAuth login first:");
        println!("  1. Start server: just start");
        println!("  2. Start frontend: just web-dev");
        println!("  3. Navigate to http://localhost:3000/login");
        println!("  4. Enter handle: {}", handle);
        return Ok(());
    }

    print!("• Checking indexed decks... ");
    let deck_rows = client
        .query(
            "SELECT at_uri, title, indexed_at FROM indexed_decks WHERE did = $1 ORDER BY indexed_at DESC LIMIT 5",
            &[&did],
        )
        .await
        .map_err(|e| malfestio_core::Error::Database(format!("Deck query failed: {}", e)))?;

    if deck_rows.is_empty() {
        println!("0 decks");
    } else {
        println!("{} deck(s)", deck_rows.len());
        for row in &deck_rows {
            let at_uri: String = row.get(0);
            let title: Option<String> = row.get(1);
            let indexed_at: chrono::DateTime<chrono::Utc> = row.get(2);
            let time_ago = format_time_ago(indexed_at);
            println!("  - {} ({})", title.unwrap_or_else(|| "Untitled".to_string()), time_ago);
            println!("    {}", at_uri);
        }
    }

    print!("• Checking indexed cards... ");
    let card_count: i64 = client
        .query_one("SELECT COUNT(*) FROM indexed_cards WHERE did = $1", &[&did])
        .await
        .map_err(|e| malfestio_core::Error::Database(format!("Card count query failed: {}", e)))?
        .get(0);

    println!("{} card(s)", card_count);

    print!("• Checking indexed notes... ");
    let note_count: i64 = client
        .query_one("SELECT COUNT(*) FROM indexed_notes WHERE did = $1", &[&did])
        .await
        .map_err(|e| malfestio_core::Error::Database(format!("Note count query failed: {}", e)))?
        .get(0);

    println!("{} note(s)", note_count);

    println!("\n✓ Status: Ready for testing");
    println!("\nNext steps:");
    println!("  - Publish content via UI to see it indexed");
    println!("  - Check Bluesky profile: https://bsky.app/profile/{}", handle);
    println!("  - Inspect records: https://pdsls.dev/at/{}", did);

    Ok(())
}

fn format_time_ago(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(timestamp);

    if duration.num_seconds() < 60 {
        format!("{} seconds ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 30 {
        format!("{} days ago", duration.num_days())
    } else {
        format!("{} months ago", duration.num_days() / 30)
    }
}
