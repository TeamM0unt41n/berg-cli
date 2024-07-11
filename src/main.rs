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
        Commands::Sync { flagdump } => {
            berg_cli::commands::sync(*flagdump).await?;
        }
        Commands::Authenticate {} => {
            berg_cli::commands::authenticate().await?;
        }
        Commands::Submit { challenge, flag } => {
            berg_cli::commands::submit(challenge, flag).await?;
        }
        Commands::Instance { command } => match command {
            InstanceCommands::Start { challenge } => {
                berg_cli::commands::instance_start(challenge).await?;
            }
            InstanceCommands::Stop {} => {
                berg_cli::commands::instance_stop().await?;
            }
        },
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
    Sync {
        #[arg(long)]
        flagdump: bool,
    },
    Authenticate {},
    Submit {
        #[arg()]
        challenge: String,
        #[arg()]
        flag: String,
    },
    Instance {
        #[command(subcommand)]
        command: InstanceCommands,
    },
}

#[derive(Subcommand)]
pub enum InstanceCommands {
    Start {
        #[arg()]
        challenge: String,
    },
    Stop {},
}
