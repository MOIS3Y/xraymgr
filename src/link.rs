//! Utilities for generating and displaying VLESS links and QR codes.

use crate::crypto::derive_public_key;
use anyhow::{anyhow, Result};
use console::style;
use qrcode::{render::unicode, EcLevel, QrCode};
use serde_json::Value;
use urlencoding::encode;

/// Configuration parameters required to generate a VLESS link.
pub struct VlessConfig {
    /// Client UUID.
    pub uuid: String,
    /// Server address (IP or domain).
    pub address: String,
    /// Server port.
    pub port: u16,
    /// Flow control setting (e.g., `xtls-rprx-vision`).
    pub flow: String,
    /// Server Name Indication (SNI) for Reality.
    pub sni: String,
    /// Derived public key for Reality.
    pub pbk: String,
    /// Short ID for Reality.
    pub sid: String,
    /// Client email or identifier.
    pub email: String,
}

/// Generates a valid VLESS sharing link based on the provided configuration.
pub fn generate_link(config: &VlessConfig) -> String {
    format!(
        "vless://{}@{}:{}?security=reality&sni={}&fp=chrome&pbk={}&sid={}&flow={}#{}",
        config.uuid,
        config.address,
        config.port,
        encode(&config.sni),
        encode(&config.pbk),
        encode(&config.sid),
        encode(&config.flow),
        encode(&config.email)
    )
}

/// Renders the VLESS link as a compact QR code in the terminal and prints connection details.
pub fn display_qr(link: &str, config: &VlessConfig) -> Result<()> {
    let code = QrCode::with_error_correction_level(link, EcLevel::L)
        .map_err(|e| anyhow!("Failed to generate QR code: {}", e))?;

    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build();

    println!("\n{}", image);
    println!("\n{}", style(link).cyan().italic());

    println!("\n{}", style("Connection Details:").bold().underlined());
    println!(
        "{:<15} {}",
        style("Address:").dim(),
        style(&config.address).yellow()
    );
    println!(
        "{:<15} {}",
        style("Port:").dim(),
        style(&config.port).yellow()
    );
    println!(
        "{:<15} {}",
        style("UUID:").dim(),
        style(&config.uuid).yellow()
    );
    println!(
        "{:<15} {}",
        style("Flow:").dim(),
        style(&config.flow).yellow()
    );
    println!(
        "{:<15} {}",
        style("SNI:").dim(),
        style(&config.sni).yellow()
    );
    println!(
        "{:<15} {}",
        style("PublicKey:").dim(),
        style(&config.pbk).yellow()
    );
    println!(
        "{:<15} {}",
        style("ShortID:").dim(),
        style(&config.sid).yellow()
    );

    Ok(())
}

/// Extracts all necessary parameters from the Xray configuration to build a `VlessConfig`.
///
/// This function expects the `inbound` to have Reality settings and a valid client.
pub fn extract_vless_config(inbound: &Value, email: &str, address: &str) -> Result<VlessConfig> {
    let port = inbound
        .get("port")
        .and_then(|p| p.as_u64())
        .ok_or_else(|| anyhow!("Port not found"))? as u16;

    let clients = inbound
        .get("settings")
        .and_then(|s| s.get("clients"))
        .and_then(|c| c.as_array())
        .ok_or_else(|| anyhow!("Clients not found"))?;

    let client = clients
        .iter()
        .find(|c| c.get("email").and_then(|e| e.as_str()) == Some(email))
        .ok_or_else(|| anyhow!("Client with email '{}' not found", email))?;

    let uuid = client
        .get("id")
        .and_then(|i| i.as_str())
        .ok_or_else(|| anyhow!("Client ID not found"))?
        .to_string();
    let flow = client
        .get("flow")
        .and_then(|f| f.as_str())
        .unwrap_or("")
        .to_string();

    let stream_settings = inbound
        .get("streamSettings")
        .ok_or_else(|| anyhow!("streamSettings not found"))?;
    let reality_settings = stream_settings
        .get("realitySettings")
        .ok_or_else(|| anyhow!("realitySettings not found"))?;

    let sni = reality_settings
        .get("serverNames")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|f| f.as_str())
        .ok_or_else(|| anyhow!("SNI (serverNames) not found"))?
        .to_string();

    let priv_key = reality_settings
        .get("privateKey")
        .and_then(|k| k.as_str())
        .ok_or_else(|| anyhow!("privateKey not found"))?;

    let pbk = derive_public_key(priv_key)?;

    let sid = reality_settings
        .get("shortIds")
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|f| f.as_str())
        .ok_or_else(|| anyhow!("shortId not found"))?
        .to_string();

    Ok(VlessConfig {
        uuid,
        address: address.to_string(),
        port,
        flow,
        sni,
        pbk,
        sid,
        email: email.to_string(),
    })
}
