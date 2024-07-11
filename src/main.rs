#![feature(try_blocks)]
use clap::{Parser, Subcommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { server, path } => {
            berg_cli::commands::init(server, path).await?;
        }
        Commands::Sync {} => {
            berg_cli::commands::sync().await?;
        }
        Commands::Authenticate {} => {
            berg_cli::commands::authenticate().await?;
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        #[arg()]
        server: String,
        #[arg()]
        path: Option<String>,
    },
    Sync {},
    Authenticate {},
}
