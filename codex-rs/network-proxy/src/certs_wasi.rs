//! Host-boundary definitions for Codex's managed-proxy CA on agentOS.
//!
//! TLS for guest HTTP is terminated by the trusted agentOS sidecar. The guest
//! must not mint a second interception CA or modify trust policy. These types
//! keep shared configuration code portable, while any attempt to enable the
//! in-process MITM path fails explicitly.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use anyhow::bail;

pub const CUSTOM_CA_ENV_KEYS: [&str; 11] = [
    "CODEX_CA_CERTIFICATE",
    "SSL_CERT_FILE",
    "REQUESTS_CA_BUNDLE",
    "CURL_CA_BUNDLE",
    "NODE_EXTRA_CA_CERTS",
    "GIT_SSL_CAINFO",
    "CARGO_HTTP_CAINFO",
    "PIP_CERT",
    "BUNDLE_SSL_CA_CERT",
    "npm_config_cafile",
    "NPM_CONFIG_CAFILE",
];

pub(crate) fn ca_env_from_process() -> HashMap<&'static str, String> {
    HashMap::new()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManagedMitmCaTrustBundle {
    pub(crate) path: PathBuf,
    pub(crate) startup_env_values: HashMap<&'static str, String>,
}

pub(crate) fn managed_ca_trust_bundle(
    _env: &HashMap<&'static str, String>,
) -> Result<ManagedMitmCaTrustBundle> {
    bail!(
        "Codex's in-process MITM CA is unavailable in agentOS VMs; TLS trust and network policy are owned by the trusted agentOS sidecar"
    )
}

pub fn is_managed_mitm_ca_trust_bundle_path(_path: &str) -> bool {
    false
}
