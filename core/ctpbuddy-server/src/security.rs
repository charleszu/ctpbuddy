//! Startup security configuration. Secret values never appear in diagnostics.
//!
//! CURVE/ZAP transport encryption was removed (ADR-0002): both endpoints of
//! this simulator are the operator's own programs, so transport protection
//! belongs to the deployment layer (VPN / SSH tunnel / isolated network), not
//! to the counter. The ADMIN token remains the control-plane identity check.
use std::io;

pub(crate) struct Security {
    pub admin_token: Option<String>,
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
    pub fn load() -> io::Result<Self> {
        Ok(Self {
            admin_token: configured_secret(crate::ADMIN_TOKEN_ENV)?,
        })
    }
}
