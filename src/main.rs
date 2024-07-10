#![feature(try_blocks)]
use clap::{Parser, Subcommand};

mod berg;
mod commands;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { server, path } => {
            commands::init(server, path).await?;
        }
        Commands::Sync {} => {
            commands::sync().await?;
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
}
