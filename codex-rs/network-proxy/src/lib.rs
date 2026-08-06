#![deny(clippy::print_stdout, clippy::print_stderr)]

#[cfg(not(target_os = "wasi"))]
mod attribution;
#[cfg(target_os = "wasi")]
#[path = "attribution_wasi.rs"]
mod attribution;
#[cfg(not(target_os = "wasi"))]
mod certs;
#[cfg(target_os = "wasi")]
#[path = "certs_wasi.rs"]
mod certs;
mod config;
#[cfg(not(target_os = "wasi"))]
mod connect_policy;
#[cfg(not(target_os = "wasi"))]
mod credential_broker;
#[cfg(target_os = "wasi")]
#[path = "credential_broker_wasi.rs"]
mod credential_broker;
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
#[cfg(not(target_os = "wasi"))]
mod mitm_hook;
#[cfg(target_os = "wasi")]
#[path = "mitm_hook_wasi.rs"]
mod mitm_hook;
#[cfg(not(target_os = "wasi"))]
mod native_certs;
mod network_policy;
mod policy;
mod proxy;
mod reasons;
mod remote_config;
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
#[cfg(target_os = "windows")]
mod windows_proxy_ingress;
#[cfg(target_os = "windows")]
mod windows_tcp_attribution;

pub use attribution::PROXY_ATTRIBUTION_TOKEN_ENV_KEY;
pub use attribution::write_attribution_frame;
pub use certs::CUSTOM_CA_ENV_KEYS;
pub use certs::is_managed_mitm_ca_trust_bundle_path;
pub use config::NetworkDomainPermission;
pub use config::NetworkDomainPermissionEntry;
pub use config::NetworkDomainPermissions;
pub use config::NetworkMode;
pub use config::NetworkProxyConfig;
pub use config::NetworkUnixSocketPermission;
pub use config::NetworkUnixSocketPermissions;
pub use config::host_and_port_from_network_addr;
pub use config::managed_proxy_ports;
pub use credential_broker::CREDENTIAL_BROKER_ACTIVE_ENV_KEY;
pub use credential_broker::brokered_credential_dummy_env_keys;
pub use credential_broker::brokered_credential_env_keys;
pub use mitm_hook::InjectedHeaderConfig;
pub use mitm_hook::MitmHookActionsConfig;
pub use mitm_hook::MitmHookBodyConfig;
pub use mitm_hook::MitmHookConfig;
pub use mitm_hook::MitmHookMatchConfig;
pub use network_policy::NetworkDecision;
pub use network_policy::NetworkDecisionSource;
pub use network_policy::NetworkPolicyDecider;
pub use network_policy::NetworkPolicyDeciderFuture;
pub use network_policy::NetworkPolicyDecision;
pub use network_policy::NetworkPolicyRequest;
pub use network_policy::NetworkPolicyRequestArgs;
pub use network_policy::NetworkProtocol;
pub use policy::normalize_host;
pub use proxy::ALL_PROXY_ENV_KEYS;
pub use proxy::ALLOW_LOCAL_BINDING_ENV_KEY;
pub use proxy::Args;
#[cfg(target_os = "macos")]
pub use proxy::CODEX_PROXY_GIT_SSH_COMMAND_MARKER;
pub use proxy::DEFAULT_NO_PROXY_VALUE;
pub use proxy::ManagedNetworkSandboxContext;
pub use proxy::NO_PROXY_ENV_KEYS;
pub use proxy::NetworkProxy;
pub use proxy::NetworkProxyBuilder;
pub use proxy::NetworkProxyHandle;
pub use proxy::PROXY_ACTIVE_ENV_KEY;
pub use proxy::PROXY_ENV_KEYS;
#[cfg(target_os = "macos")]
pub use proxy::PROXY_GIT_SSH_COMMAND_ENV_KEY;
pub use proxy::PROXY_URL_ENV_KEYS;
pub use proxy::PreparedManagedNetwork;
pub use proxy::has_proxy_url_env_vars;
pub use proxy::is_managed_proxy_env_var;
pub use proxy::proxy_url_env_value;
pub use proxy::strip_managed_proxy_env;
pub use remote_config::RemoteNetworkProxyConfig;
pub use remote_config::RemoteNetworkProxyLaunchConfig;
pub use runtime::BlockedRequest;
pub use runtime::BlockedRequestArgs;
pub use runtime::BlockedRequestObserver;
pub use runtime::BlockedRequestObserverFuture;
pub use runtime::ConfigReloader;
pub use runtime::ConfigReloaderFuture;
pub use runtime::ConfigState;
pub use runtime::NetworkProxyState;
pub use state::NetworkProxyAuditMetadata;
pub use state::NetworkProxyConstraintError;
pub use state::NetworkProxyConstraints;
pub use state::PartialNetworkProxyConfig;
pub use state::build_config_state;
pub use state::validate_policy_against_constraints;
