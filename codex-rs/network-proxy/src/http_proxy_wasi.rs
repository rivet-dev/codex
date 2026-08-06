//! Host boundary for Codex's managed HTTP proxy on agentOS.
//!
//! The trusted agentOS sidecar owns proxy execution and network policy. Starting
//! a second proxy in the untrusted guest is an architectural error, so this path
//! fails explicitly. This does not restrict guest TCP or HTTP capabilities.
use std::net::SocketAddr;
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;

use anyhow::Result;
use anyhow::bail;

use crate::network_policy::NetworkPolicyDecider;
use crate::state::NetworkProxyState;

pub async fn run_http_proxy(
    _state: Arc<NetworkProxyState>,
    _addr: SocketAddr,
    _policy_decider: Option<Arc<dyn NetworkPolicyDecider>>,
    _environment_id: Option<String>,
) -> Result<()> {
    bail!(
        "Codex's managed HTTP proxy cannot run inside an agentOS VM; proxy execution is owned by the trusted agentOS sidecar"
    )
}

pub async fn run_http_proxy_with_std_listener(
    _state: Arc<NetworkProxyState>,
    _listener: StdTcpListener,
    _policy_decider: Option<Arc<dyn NetworkPolicyDecider>>,
    _environment_id: Option<String>,
) -> Result<()> {
    bail!(
        "Codex's managed HTTP proxy cannot run inside an agentOS VM; proxy execution is owned by the trusted agentOS sidecar"
    )
}
