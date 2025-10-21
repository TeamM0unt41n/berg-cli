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
            InstanceCommands::Start { challenge, force } => {
                berg_cli::commands::instance_start(challenge, *force).await?;
            }
            InstanceCommands::Stop {} => {
                berg_cli::commands::instance_stop().await?;
            }
            InstanceCommands::Info {} => {
                berg_cli::commands::instance_info().await?;
            }
            InstanceCommands::Exploit {
                script,
                cmd,
                start,
                stop,
                force,
            } => {
                berg_cli::commands::instance_exploit(script, cmd, *start, *stop, *force).await?;
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
        #[arg(long, default_value = "false")]
        force: bool,
    },
    Stop {},
    Info {},
    Exploit {
        /// Path to the exploit python script
        #[arg()]
        script: String,
        /// Command to run the exploit script with
        #[arg(long, default_value = "python")]
        cmd: String,
        /// Whether to start the instance after running the exploit
        #[arg(long, default_value = "true")]
        start: bool,
        /// Whether to stop the instance after running the exploit
        #[arg(long, default_value = "false")]
        stop: bool,
        /// Whether to force start the instance even if another instance is running (will stop the running instance first)
        #[arg(long, default_value = "false")]
        force: bool,
    },
}
