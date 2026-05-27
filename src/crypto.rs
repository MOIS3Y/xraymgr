//! Cryptographic utilities for Xray configuration.

use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use x25519_dalek::{PublicKey, StaticSecret};

/// Derives an X25519 public key from a base64-encoded URL-safe private key.
///
/// This is used to compute the `pbk` (PublicKey) required for Reality
/// connections from the `privateKey` stored in the Xray configuration.
pub fn derive_public_key(private_key_b64: &str) -> Result<String> {
    let private_key_bytes = URL_SAFE_NO_PAD
        .decode(private_key_b64)
        .with_context(|| "Failed to decode private key from base64")?;

    if private_key_bytes.len() != 32 {
        return Err(anyhow::anyhow!(
            "Private key must be 32 bytes, got {}",
            private_key_bytes.len()
        ));
    }

    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&private_key_bytes);

    let secret = StaticSecret::from(bytes);
    let public = PublicKey::from(&secret);

    Ok(URL_SAFE_NO_PAD.encode(public.as_bytes()))
}
