// - In the default output mode, it is paramount that the only thing written to
//   stdout is the final message (if any).
// - In --json mode, stdout must be valid JSONL, one event per line.
// For both modes, any other output must be written to stderr.
#![cfg_attr(not(target_os = "wasi"), deny(clippy::print_stdout))]

// On WASI, the full codex implementation is not yet available.
// Platform-specific dependencies (tokio process/signal, portable-pty,
// network proxy, sandbox) are gated out. Only a stub entry point is provided.
#[cfg(target_os = "wasi")]
pub fn wasi_stub_main() -> anyhow::Result<()> {
    eprintln!("codex: WASI runtime support is under development");
    Ok(())
}

// Full native implementation — includes all platform-specific dependencies
#[cfg(not(target_os = "wasi"))]
include!("lib_native.rs");
