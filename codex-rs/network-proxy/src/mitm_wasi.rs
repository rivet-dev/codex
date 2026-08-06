//! Host boundary for Codex's managed-proxy TLS interception on agentOS.
//!
//! The trusted agentOS sidecar owns TLS interception and trust policy. The
//! untrusted guest must not create a second interception authority.
use anyhow::Result;
use anyhow::bail;

#[derive(Debug)]
pub struct MitmState;

pub(crate) struct MitmUpstreamConfig {
    pub(crate) allow_upstream_proxy: bool,
}

impl MitmState {
    pub(crate) fn new(config: MitmUpstreamConfig) -> Result<Self> {
        let _ = config.allow_upstream_proxy;
        bail!(
            "Codex's in-process TLS interception cannot run inside an agentOS VM; TLS policy is owned by the trusted agentOS sidecar"
        )
    }
}
