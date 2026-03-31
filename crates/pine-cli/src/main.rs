//! Pine Script v6 CLI
//!
//! Command-line interface for the Pine Script interpreter.

use clap::{Parser, Subcommand};
use miette::{miette, Result};

#[derive(Parser)]
#[command(name = "pine")]
#[command(about = "Pine Script v6 interpreter")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Pine Script file
    Run {
        /// Path to the Pine Script file
        script: String,
        /// Path to the data CSV file
        #[arg(short, long)]
        data: Option<String>,
    },
    /// Check a Pine Script file for errors
    Check {
        /// Path to the Pine Script file
        script: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { script, data } => {
            println!("Running script: {}", script);
            if let Some(data_path) = data {
                println!("Using data: {}", data_path);
            }
            // TODO: Implement script execution
            Ok(())
        }
        Commands::Check { script } => {
            println!("Checking script: {}", script);
            // TODO: Implement script checking
            Ok(())
        }
    }
}
