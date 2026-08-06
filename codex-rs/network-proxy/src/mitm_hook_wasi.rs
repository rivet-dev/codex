//! Configuration-only boundary for managed-proxy MITM hooks on agentOS.
//!
//! The trusted sidecar owns TLS interception and hook execution. The guest
//! preserves the exact serializable config shape, but enabling any hook returns
//! a hard error before a proxy or credential broker can be constructed.

use std::collections::BTreeMap;

use anyhow::Result;
use anyhow::bail;
use serde::Deserialize;
use serde::Serialize;

use crate::config::NetworkProxyConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct MitmHookConfig {
    pub host: String,
    #[serde(rename = "match", default)]
    pub matcher: MitmHookMatchConfig,
    #[serde(default)]
    pub actions: MitmHookActionsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct MitmHookMatchConfig {
    pub methods: Vec<String>,
    pub path_prefixes: Vec<String>,
    pub query: BTreeMap<String, Vec<String>>,
    pub headers: BTreeMap<String, Vec<String>>,
    pub body: Option<MitmHookBodyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct MitmHookActionsConfig {
    pub strip_request_headers: Vec<String>,
    pub inject_request_headers: Vec<InjectedHeaderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct InjectedHeaderConfig {
    pub name: String,
    pub secret_env_var: Option<String>,
    pub secret_file: Option<String>,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct MitmHookBodyConfig(pub serde_json::Value);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MitmHookActions;

pub type MitmHooksByHost = BTreeMap<String, Vec<MitmHookActions>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookEvaluation {
    NoHooksForHost,
    Matched { actions: MitmHookActions },
    HookedHostNoMatch,
}

pub(crate) fn validate_mitm_hook_config(config: &NetworkProxyConfig) -> Result<()> {
    if config.mitm_hooks.is_empty() {
        return Ok(());
    }
    bail!(
        "Codex's managed-proxy MITM hooks cannot run inside an agentOS VM; TLS interception and credential injection are owned by the trusted agentOS sidecar"
    )
}

pub(crate) fn compile_mitm_hooks(config: &NetworkProxyConfig) -> Result<MitmHooksByHost> {
    validate_mitm_hook_config(config)?;
    Ok(BTreeMap::new())
}
