use std::sync::Once;

/// Ensures a process-wide rustls crypto provider is installed.
///
/// The WASI fork uses `ring`, which supports the target without a native C
/// toolchain dependency.
pub fn ensure_rustls_crypto_provider() {
    static RUSTLS_PROVIDER_INIT: Once = Once::new();
    RUSTLS_PROVIDER_INIT.call_once(|| {
        if rustls::crypto::ring::default_provider()
            .install_default()
            .is_err()
        {
            // Preserve the previous best-effort behavior for embedded hosts that
            // install a process-global provider before Codex can install one.
            return;
        }

        assert!(
            rustls::crypto::CryptoProvider::get_default().is_some(),
            "ring rustls crypto provider should be installed"
        );
    });
}
