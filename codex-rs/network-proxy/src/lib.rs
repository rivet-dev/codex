#![deny(clippy::print_stdout, clippy::print_stderr)]

#[cfg(not(target_os = "wasi"))]
mod certs;
#[cfg(target_os = "wasi")]
#[path = "certs_wasi.rs"]
mod certs;
mod config;
#[cfg(not(target_os = "wasi"))]
mod http_proxy;
#[cfg(target_os = "wasi")]
#[path = "http_proxy_wasi.rs"]
mod http_proxy;
#[cfg(not(target_os = "wasi"))]
mod mitm;
#[cfg(target_os = "wasi")]
#[path = "mitm_wasi.rs"]
mod mitm;
mod network_policy;
mod policy;
mod proxy;
mod reasons;
#[cfg(not(target_os = "wasi"))]
mod responses;
#[cfg(target_os = "wasi")]
#[path = "responses_wasi.rs"]
mod responses;
mod runtime;
#[cfg(not(target_os = "wasi"))]
mod socks5;
#[cfg(target_os = "wasi")]
#[path = "socks5_wasi.rs"]
mod socks5;
mod state;
#[cfg(not(target_os = "wasi"))]
mod upstream;
#[cfg(target_os = "wasi")]
#[path = "upstream_wasi.rs"]
mod upstream;

pub use config::NetworkMode;
pub use config::NetworkProxyConfig;
pub use config::host_and_port_from_network_addr;
pub use network_policy::NetworkDecision;
pub use network_policy::NetworkDecisionSource;
pub use network_policy::NetworkPolicyDecider;
pub use network_policy::NetworkPolicyDecision;
pub use network_policy::NetworkPolicyRequest;
pub use network_policy::NetworkPolicyRequestArgs;
pub use network_policy::NetworkProtocol;
pub use policy::normalize_host;
pub use proxy::ALL_PROXY_ENV_KEYS;
pub use proxy::ALLOW_LOCAL_BINDING_ENV_KEY;
pub use proxy::Args;
pub use proxy::DEFAULT_NO_PROXY_VALUE;
pub use proxy::NO_PROXY_ENV_KEYS;
pub use proxy::NetworkProxy;
pub use proxy::NetworkProxyBuilder;
pub use proxy::NetworkProxyHandle;
pub use proxy::PROXY_URL_ENV_KEYS;
pub use proxy::has_proxy_url_env_vars;
pub use proxy::proxy_url_env_value;
pub use runtime::BlockedRequest;
pub use runtime::BlockedRequestArgs;
pub use runtime::BlockedRequestObserver;
pub use runtime::ConfigReloader;
pub use runtime::ConfigState;
pub use runtime::NetworkProxyState;
pub use state::NetworkProxyAuditMetadata;
pub use state::NetworkProxyConstraintError;
pub use state::NetworkProxyConstraints;
pub use state::PartialNetworkConfig;
pub use state::PartialNetworkProxyConfig;
pub use state::build_config_state;
pub use state::validate_policy_against_constraints;
