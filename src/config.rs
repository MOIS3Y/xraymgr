//! Xray JSON configuration manipulation utilities.
//!
//! Provides functions to load, save, and modify the Xray `config.json` file,
//! specifically targeting the `inbounds` and `clients` structures.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Loads and parses the Xray configuration from the specified JSON file.
pub fn load_config(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file at {:?}", path))?;
    let config: Value =
        serde_json::from_str(&content).with_context(|| "Failed to parse config JSON")?;
    Ok(config)
}

/// Serializes and saves the Xray configuration to the specified JSON file.
pub fn save_config(path: &Path, config: &Value) -> Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)
        .with_context(|| format!("Failed to write config file to {:?}", path))?;
    Ok(())
}

/// Finds a mutable reference to an inbound block matching the specified tag.
pub fn find_inbound_mut<'a>(config: &'a mut Value, tag: &str) -> Result<&'a mut Value> {
    config
        .get_mut("inbounds")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| anyhow!("No 'inbounds' array found in config"))?
        .iter_mut()
        .find(|i| i.get("tag").and_then(|t| t.as_str()) == Some(tag))
        .ok_or_else(|| anyhow!("Inbound with tag '{}' not found", tag))
}

/// Finds a read-only reference to an inbound block matching the specified tag.
pub fn find_inbound<'a>(config: &'a Value, tag: &str) -> Result<&'a Value> {
    config
        .get("inbounds")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("No 'inbounds' array found in config"))?
        .iter()
        .find(|i| i.get("tag").and_then(|t| t.as_str()) == Some(tag))
        .ok_or_else(|| anyhow!("Inbound with tag '{}' not found", tag))
}

/// Retrieves a read-only list of clients from the given inbound configuration block.
pub fn get_clients(inbound: &Value) -> Result<&Vec<Value>> {
    inbound
        .get("settings")
        .and_then(|s| s.get("clients"))
        .and_then(|c| c.as_array())
        .ok_or_else(|| anyhow!("No clients found in inbound settings"))
}

/// Retrieves a mutable list of clients from the given inbound configuration block.
pub fn get_clients_mut(inbound: &mut Value) -> Result<&mut Vec<Value>> {
    inbound
        .get_mut("settings")
        .and_then(|s| s.get_mut("clients"))
        .and_then(|c| c.as_array_mut())
        .ok_or_else(|| anyhow!("No clients found in inbound settings"))
}
