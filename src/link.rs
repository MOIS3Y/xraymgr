//! Utilities for generating and displaying VLESS links and QR codes.

use crate::crypto::derive_public_key;
use anyhow::{Result, anyhow};
use console::style;
use qrcode::{EcLevel, QrCode, render::{unicode, svg}};
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
    /// Network protocol (e.g., tcp, ws, grpc).
    pub network: String,
    /// Security type (e.g., tls, reality).
    pub security: String,
    /// Fingerprint (e.g., chrome, firefox).
    pub fp: String,
}

/// Resolves the server address, preferring the user-provided one, but falling
/// back to the `listen` field in the inbound config.
pub fn resolve_address(user_address: Option<String>, inbound: &Value) -> Result<String> {
    if let Some(addr) = user_address {
        Ok(addr)
    } else if let Some(listen_addr) = inbound.get("listen").and_then(|v| v.as_str()) {
        eprintln!(
            "{} {} Server address not provided, using 'listen' field \
             from config: {}",
            style("!").yellow().bold(),
            style("Warning:").yellow().bold(),
            style(listen_addr).cyan().bold()
        );
        Ok(listen_addr.to_string())
    } else {
        anyhow::bail!(
            "Server address is required. Use --address, set \
             XRAYMGR_ADDRESS, or define 'listen' in your inbound config."
        );
    }
}

/// Generates a valid VLESS sharing link based on the provided configuration.
pub fn generate_link(config: &VlessConfig) -> String {
    format!(
        "vless://{}@{}:{}?security={}&sni={}&fp={}&pbk={}&sid={}&flow={}#{}",
        config.uuid,
        config.address,
        config.port,
        encode(&config.security),
        encode(&config.sni),
        encode(&config.fp),
        encode(&config.pbk),
        encode(&config.sid),
        encode(&config.flow),
        encode(&config.email)
    )
}

/// Generates a structured JSON configuration for a vanilla Xray client.
pub fn generate_client_json(config: &VlessConfig) -> String {
    let mut stream_settings = serde_json::json!({
        "network": config.network,
        "security": config.security,
    });

    if config.security == "reality" {
        stream_settings["realitySettings"] = serde_json::json!({
            "fingerprint": config.fp,
            "serverName": config.sni,
            "publicKey": config.pbk,
            "shortId": config.sid,
            "spiderX": ""
        });
    } else if config.security == "tls" {
        stream_settings["tlsSettings"] = serde_json::json!({
            "fingerprint": config.fp,
            "serverName": config.sni,
        });
    }

    let client_config = serde_json::json!({
        "protocol": "vless",
        "settings": {
            "vnext": [{
                "address": config.address,
                "port": config.port,
                "users": [{
                    "id": config.uuid,
                    "email": config.email,
                    "encryption": "none",
                    "flow": config.flow
                }]
            }]
        },
        "streamSettings": stream_settings
    });
    serde_json::to_string_pretty(&client_config).unwrap_or_default()
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

/// Generates an SVG string representation of the QR code
pub fn generate_qr_svg(link: &str) -> Result<String> {
    let code = QrCode::with_error_correction_level(link, EcLevel::L)
        .map_err(|e| anyhow!("Failed to generate QR code: {}", e))?;

    let svg = code
        .render::<svg::Color>()
        .min_dimensions(250, 250)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    Ok(svg)
}

/// Extracts all necessary parameters from the Xray configuration to build a `VlessConfig`.
///
/// This function expects the `inbound` to have Reality settings and a valid client.
pub fn extract_vless_config(
    inbound: &Value,
    email: &str,
    address: &str,
) -> Result<VlessConfig> {
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

    let network = stream_settings
        .get("network")
        .and_then(|n| n.as_str())
        .unwrap_or("tcp")
        .to_string();

    let security = stream_settings
        .get("security")
        .and_then(|s| s.as_str())
        .unwrap_or("none")
        .to_string();

    let mut sni = String::new();
    let mut pbk = String::new();
    let mut sid = String::new();
    let mut fp = "chrome".to_string();

    if security == "reality" {
        if let Some(reality_settings) = stream_settings.get("realitySettings") {
            sni = reality_settings
                .get("serverNames")
                .and_then(|s| s.as_array())
                .and_then(|a| a.first())
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();

            if let Some(priv_key) = reality_settings
                .get("privateKey")
                .and_then(|k| k.as_str())
            {
                pbk = derive_public_key(priv_key).unwrap_or_default();
            }

            sid = reality_settings
                .get("shortIds")
                .and_then(|s| s.as_array())
                .and_then(|a| a.first())
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();

            if let Some(fingerprint) = reality_settings
                .get("fingerprint")
                .and_then(|f| f.as_str())
            {
                fp = fingerprint.to_string();
            }
        }
    } else if security == "tls" {
        if let Some(tls_settings) = stream_settings.get("tlsSettings") {
            sni = tls_settings
                .get("serverName")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            if let Some(fingerprint) = tls_settings
                .get("fingerprint")
                .and_then(|f| f.as_str())
            {
                fp = fingerprint.to_string();
            }
        }
    }

    Ok(VlessConfig {
        uuid,
        address: address.to_string(),
        port,
        flow,
        sni,
        pbk,
        sid,
        email: email.to_string(),
        network,
        security,
        fp,
    })
}
