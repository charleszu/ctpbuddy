//! Startup security configuration. Secret values never appear in diagnostics.
use std::io;

pub(crate) struct Security {
    pub admin_token: Option<String>,
    pub curve_secret: Option<String>,
    pub curve_clients: Vec<[u8; 32]>,
}

pub(crate) fn configured_secret(key: &str) -> io::Result<Option<String>> {
    match std::env::var(key) {
        Err(std::env::VarError::NotPresent) => Ok(None),
        Ok(value) if !value.is_empty() => Ok(Some(value)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{key} must not be empty or non-Unicode"),
        )),
    }
}

impl Security {
    pub fn load(endpoint: &str) -> io::Result<Self> {
        let admin_token = configured_secret(crate::ADMIN_TOKEN_ENV)?;
        let curve_secret = configured_secret(crate::ZMQ_CURVE_SECRET_KEY_ENV)?;
        let public = configured_secret(crate::ZMQ_CURVE_PUBLIC_KEY_ENV)?;
        let server = configured_secret(crate::ZMQ_CURVE_SERVER_KEY_ENV)?;
        let clients = configured_secret("CTPBUDDY_ZMQ_CURVE_CLIENT_KEYS")?;
        let mut curve_clients = Vec::new();
        if curve_secret.is_some() || public.is_some() || server.is_some() || clients.is_some() {
            if !endpoint.starts_with("zmq://") || curve_secret.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "CURVE requires a zmq:// endpoint and CTPBUDDY_ZMQ_CURVE_SECRET_KEY",
                ));
            }
            for key in clients.as_deref().unwrap_or("").split(',') {
                let decoded = decode_z85_key(key.trim()).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput,
                        "CURVE requires CTPBUDDY_ZMQ_CURVE_CLIENT_KEYS: comma-separated Z85 public keys")
                })?;
                curve_clients.push(decoded);
            }
            for (key, value) in [
                (crate::ZMQ_CURVE_SECRET_KEY_ENV, &curve_secret),
                (crate::ZMQ_CURVE_PUBLIC_KEY_ENV, &public),
                (crate::ZMQ_CURVE_SERVER_KEY_ENV, &server),
            ] {
                if let Some(value) = value {
                    if !valid_z85_key(value) {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("{key} must be a valid 40-character Z85 key"),
                        ));
                    }
                }
            }
        }
        Ok(Self {
            admin_token,
            curve_secret,
            curve_clients,
        })
    }
}

pub fn valid_z85_key(value: &str) -> bool {
    decode_z85_key(value).is_some()
}

fn decode_z85_key(value: &str) -> Option<[u8; 32]> {
    const ALPHABET: &[u8] =
        b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    if value.len() != 40 {
        return None;
    }
    let mut result = [0; 32];
    for (i, group) in value.as_bytes().chunks(5).enumerate() {
        let mut decoded = 0u64;
        for c in group {
            let index = ALPHABET.iter().position(|b| b == c)?;
            decoded = decoded * 85 + index as u64;
        }
        let decoded = u32::try_from(decoded).ok()?;
        result[i * 4..i * 4 + 4].copy_from_slice(&decoded.to_be_bytes());
    }
    Some(result)
}
