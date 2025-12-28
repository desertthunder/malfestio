use clap::{Parser, Subcommand};

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
}

#[tokio::main]
async fn main() -> malfestio_core::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => {
            malfestio_server::start().await?;
        }
    }

    Ok(())
}
