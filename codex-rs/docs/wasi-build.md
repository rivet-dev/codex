# Building codex-exec for wasm32-wasip1 (secure-exec VM)

`codex-exec --session-turn` runs the real codex-core agent inside the secure-exec V8/WASM VM and
speaks the EE newline-JSON protocol (`exec/src/session_turn_wasi.rs`). This page documents how the
wasm artifact is built and the toolchain constraints behind it.

## TL;DR

```sh
SECURE_EXEC_DIR=/path/to/secure-exec scripts/build-wasi-codex-exec.sh
```

Produces an optimized `codex-exec` (and `codex` alias) and installs them into
`$SECURE_EXEC_DIR/registry/software/codex/wasm/`. The script is idempotent and restores the rustup
sysroot on exit (pass `KEEP_SYSROOT=1` to skip restore for faster iteration).

## Why it's not a plain `cargo build`

- **`-Z build-std`** is required: wasm32-wasip1 has no OS threads, no `tokio::net` reactor, and no
  fork/exec as raw syscalls. We compile std from a *patched* rust-src so the WASI platform layer can
  bridge those to the host (see `.cargo/config.toml` and the secure-exec `registry/native` patches).
- **C deps** (ring, sqlite) need the wasi-sdk clang, hence `CC_wasm32_wasip1` / `CFLAGS_wasm32_wasip1`
  pointing at `secure-exec/registry/native/c/vendor/wasi-sdk`. The host build (proc-macros, host-side
  ring) keeps the system `cc`/`gcc` — only the *target* compiler is overridden.
- **`--cfg tokio_unstable`** lifts tokio's wasm `compile_error!` so the patched tokio process/net
  modules compile.
- **panic strategy**: the wasm target forces `-Cpanic=abort`, but cargo's build-std injects
  `panic_unwind` via `--extern` and rustc resolves the *actual* panic runtime (`panic_abort`) from
  the **sysroot**. So the build (a) stashes the prebuilt `wasm32-wasip1` `*.rlib`/`*.rmeta` out of the
  sysroot lib dir — keeping `self-contained/` CRT — so build-std's `libcore` is the only one (avoids
  `E0152 duplicate lang item core` at the bin link), and (b) copies build-std's freshly-built,
  ABI-matched `libpanic_abort` into that sysroot dir, then builds with
  `--config 'profile.release.panic="abort"'`. The script automates all of this.

## Runtime gates (why the agent loop works on a single thread)

codex-core spawns its agent turn on an internally `tokio::spawn`ed submission loop; on the
single-threaded current-thread runtime this only advances if **no task blocks the executor thread**.
Three `#[cfg(target_os = "wasi")]` gates in the fork keep that true:

- `core/src/shell_snapshot.rs` — skip the shell-snapshot subprocess (it blocks on a child wait that
  the VM can't deliver, deadlocking the loop).
- `core/src/git_info.rs` — skip git-subprocess metadata collection (same blocking-child class).
- `core/src/rollout/recorder.rs` — the rollout writer drains/acks but does not persist (the file path
  fails on the runtime and would kill the task; the VM is ephemeral and agent-os resumes via
  adapter-supplied `history`, not codex's on-disk rollout).
- `core/src/state_db.rs` — skip sqlx-sqlite (fails ENOTSUP on the thread-less runtime).

## Folding into `make -C registry/native wasm` (remaining work)

The secure-exec native Makefile already builds the *stub* `crates/commands/{codex,codex-exec}` with
the same `-Z build-std=std,panic_abort` + self-contained link-arg flags. To ship the real engine from
there, either (a) make those command crates depend on the real `codex-core`/`codex-exec` from this
fork (vendored), or (b) have the Makefile invoke this script. The remaining blocker to fully drop the
sysroot stash is to source the self-contained CRT from wasi-sdk and build with the prebuilt rust
target *uninstalled*, so build-std is unambiguously the only std — then no stash is needed.
