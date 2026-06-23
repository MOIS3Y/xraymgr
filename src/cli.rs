//! Command-line interface definitions and parsing logic.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Main entry point for the Xray manager CLI arguments.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to Xray config file
    #[arg(short, long, default_value = "config.json", env = "XRAYMGR_CONFIG")]
    pub config: PathBuf,

    /// Inbound tag to manage
    #[arg(short, long, default_value = "vless-in", env = "XRAYMGR_TAG")]
    pub tag: String,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands for managing Xray clients and configuration.
#[derive(Subcommand)]
pub enum Commands {
    /// List all clients in the specified inbound
    List,

    /// Add a new client
    Add {
        /// Client email
        email: String,
    },

    /// Remove an existing client
    Remove {
        /// Client email
        email: String,
    },

    /// Show VLESS link and QR code for a client
    Show {
        /// Client email
        email: String,

        /// Server address (IP or domain). Falls back to XRAYMGR_ADDRESS env or SNI from config.
        #[arg(short, long, env = "XRAYMGR_ADDRESS")]
        address: Option<String>,
    },

    /// Generate an HTML page with the client's connection info to stdout
    Html {
        /// Client email
        email: String,

        /// Server address (IP or domain). Falls back to XRAYMGR_ADDRESS env or SNI from config.
        #[arg(short, long, env = "XRAYMGR_ADDRESS")]
        address: Option<String>,

        /// Path to custom HTML template
        #[arg(long)]
        template: Option<PathBuf>,
    },
}
