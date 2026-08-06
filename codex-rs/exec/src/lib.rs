// - In the default output mode, it is paramount that the only thing written to
//   stdout is the final message (if any).
// - In --json mode, stdout must be valid JSONL, one event per line.
// For both modes, any other output must be written to stderr.
#![cfg_attr(not(target_os = "wasi"), deny(clippy::print_stdout))]

#[cfg(target_os = "wasi")]
mod session_turn_wasi;

#[cfg(target_os = "wasi")]
pub fn wasi_stub_main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--session-turn") {
        session_turn_wasi::run()
    } else {
        eprintln!("codex-exec: only --session-turn is supported on wasm32-wasip1");
        Ok(())
    }
}

#[cfg(not(target_os = "wasi"))]
include!("lib_native.rs");
