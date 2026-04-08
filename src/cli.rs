use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "divinelight")]
#[command(about = "DivineLight - Unified AI Memory System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(short, long)]
    data_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Serve,
    Backup {
        #[arg(short, long)]
        path: String,
    },
    Restore {
        #[arg(short, long)]
        path: String,
    },
    Export {
        #[arg(short, long)]
        path: String,
    },
    Import {
        #[arg(short, long)]
        path: String,
    },
    Status,
}

fn main() {
    let cli = Cli::parse();

    let data_dir = cli.data_dir.unwrap_or_else(|| {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("divinelight")
    });

    match cli.command {
        Commands::Serve => {
            println!("Starting DivineLight server on port {}...", cli.port);
            println!("Data directory: {:?}", data_dir);
        }
        Commands::Backup { path } => {
            println!("Creating backup at: {}", path);
        }
        Commands::Restore { path } => {
            println!("Restoring from: {}", path);
        }
        Commands::Export { path } => {
            println!("Exporting to: {}", path);
        }
        Commands::Import { path } => {
            println!("Importing from: {}", path);
        }
        Commands::Status => {
            println!("Data directory: {:?}", data_dir);
            println!("Checking status...");
        }
    }
}
