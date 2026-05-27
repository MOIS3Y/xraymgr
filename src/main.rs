//! Xray Manager CLI (`xraymgr`)
//!
//! A simple command-line utility for managing Xray JSON configurations,
//! specifically focused on VLESS + Reality setups. It allows adding,
//! removing, and listing clients, as well as generating VLESS sharing links
//! and QR codes.

mod cli;
mod config;
mod crypto;
mod link;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use config::{
    find_inbound, find_inbound_mut, get_clients, get_clients_mut, load_config, save_config,
};
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use link::{display_qr, extract_vless_config, generate_link};
use serde_json::json;
use uuid::Uuid;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("∗ ", "");
static SUCCESS: Emoji<'_, '_> = Emoji("✔ ", "");
static GEAR: Emoji<'_, '_> = Emoji("⚙  ", "");

fn main() -> Result<()> {
    let cli = Cli::parse();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message("Loading config...");
    pb.enable_steady_tick(std::time::Duration::from_millis(120));

    let mut config = load_config(&cli.config)?;
    pb.finish_and_clear();

    match cli.command {
        Commands::List => {
            let inbound = find_inbound(&config, &cli.tag)?;
            let clients = get_clients(inbound)?;

            println!(
                "{} Clients in inbound {}:",
                LOOKING_GLASS,
                style(&cli.tag).yellow().bold()
            );
            println!(
                "\n{:<30} {:<40}",
                style("Email").bold().underlined(),
                style("ID").bold().underlined()
            );

            for client in clients {
                let email = client
                    .get("email")
                    .and_then(|e| e.as_str())
                    .unwrap_or("unknown");
                let id = client
                    .get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("unknown");
                println!("{:<30} {:<40}", style(email).cyan(), style(id).dim());
            }
        }
        Commands::Add { email } => {
            let inbound = find_inbound_mut(&mut config, &cli.tag)?;
            let clients = get_clients_mut(inbound)?;

            if clients
                .iter()
                .any(|c| c.get("email").and_then(|e| e.as_str()) == Some(&email))
            {
                anyhow::bail!("Client with email '{}' already exists", email);
            }

            let new_client = json!({
                "id": Uuid::new_v4().to_string(),
                "email": email,
                "level": 0,
                "flow": "xtls-rprx-vision"
            });

            clients.push(new_client);
            save_config(&cli.config, &config)?;
            println!(
                "{} Client {} {} successfully",
                SUCCESS,
                style(&email).green().bold(),
                style("added").green()
            );
        }
        Commands::Remove { email } => {
            let inbound = find_inbound_mut(&mut config, &cli.tag)?;
            let clients = get_clients_mut(inbound)?;

            let pos = clients
                .iter()
                .position(|c| c.get("email").and_then(|e| e.as_str()) == Some(&email))
                .ok_or_else(|| anyhow::anyhow!("Client with email '{}' not found", email))?;

            clients.remove(pos);
            save_config(&cli.config, &config)?;
            println!(
                "{} Client {} {} successfully",
                SUCCESS,
                style(&email).red().bold(),
                style("removed").red()
            );
        }
        Commands::Show { email, address } => {
            let final_address = address.ok_or_else(|| {
                anyhow::anyhow!(
                    "Server address is required. Use --address or set the XRAYMGR_ADDRESS environment variable."
                )
            })?;

            let inbound = find_inbound(&config, &cli.tag)?;
            let vless_config = extract_vless_config(inbound, &email, &final_address)?;
            let link = generate_link(&vless_config);

            println!(
                "{} Generating VLESS config for {} (using address: {})...",
                GEAR,
                style(&email).magenta().bold(),
                style(&final_address).yellow()
            );
            display_qr(&link, &vless_config)?;
        }
    }

    Ok(())
}
