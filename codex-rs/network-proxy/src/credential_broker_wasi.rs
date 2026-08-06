//! Host boundary for managed-proxy credential injection on agentOS.
//!
//! Credential injection is coupled to the sidecar-owned TLS interception path.
//! Configuration that enables it is rejected before this state is constructed;
//! disabled configurations retain the exact environment-cleanup behavior.

use std::collections::HashMap;

pub const CREDENTIAL_BROKER_ACTIVE_ENV_KEY: &str = "CODEX_NETWORK_PROXY_CREDENTIAL_BROKER_ACTIVE";
pub(crate) const BROKERED_CREDENTIALS_ENV_KEY: &str = "CODEX_NETWORK_PROXY_BROKERED_CREDENTIALS";

const CREDENTIAL_ENV_KEYS: &[&str] = &[
    "GH_HOST",
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "GH_ENTERPRISE_TOKEN",
    "GITHUB_ENTERPRISE_TOKEN",
    "OPENAI_API_KEY",
];

#[derive(Clone)]
pub(crate) struct CredentialBroker {
    enabled: bool,
}

impl CredentialBroker {
    pub(crate) fn new(enabled: bool) -> Self {
        assert!(
            !enabled,
            "managed-proxy credential injection must be rejected before constructing an agentOS guest proxy; the trusted sidecar owns credential injection"
        );
        Self { enabled }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn virtualize_child_env(&self, env: &mut HashMap<String, String>) {
        debug_assert!(!self.enabled);
        env.remove(CREDENTIAL_BROKER_ACTIVE_ENV_KEY);
        env.remove(BROKERED_CREDENTIALS_ENV_KEY);
    }

    pub(crate) fn host_requires_mitm(&self, _host: &str) -> bool {
        debug_assert!(!self.enabled);
        false
    }
}

pub fn brokered_credential_dummy_env_keys(env: &HashMap<String, String>) -> Vec<String> {
    env.get(BROKERED_CREDENTIALS_ENV_KEY)
        .and_then(|marker| serde_json::from_str::<Vec<(String, String)>>(marker).ok())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(key, dummy_value)| {
            (CREDENTIAL_ENV_KEYS.contains(&key.as_str()) && env.get(&key) == Some(&dummy_value))
                .then_some(key)
        })
        .collect()
}

pub fn brokered_credential_env_keys(
    env: &HashMap<String, String>,
) -> impl Iterator<Item = &'static str> {
    let active = env
        .get(CREDENTIAL_BROKER_ACTIVE_ENV_KEY)
        .is_some_and(|value| value == "1");
    CREDENTIAL_ENV_KEYS.iter().copied().filter(move |_| active)
}
