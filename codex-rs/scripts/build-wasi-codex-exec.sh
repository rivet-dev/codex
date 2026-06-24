#!/usr/bin/env bash
#
# Reproducible build of the wasm32-wasip1 `codex-exec` (the `--session-turn` engine) that runs the
# real codex-core agent inside the secure-exec VM. Produces `codex-exec` + `codex` artifacts and
# (optionally) installs them into the secure-exec registry.
#
# This script is self-contained and idempotent: it prepares the rustup sysroot for `-Z build-std`,
# builds, and restores the sysroot afterwards. See docs/wasi-build.md for the why.
#
# Usage:
#   SECURE_EXEC_DIR=/path/to/secure-exec scripts/build-wasi-codex-exec.sh
#
# Env:
#   SECURE_EXEC_DIR  secure-exec checkout (for the wasi-sdk C toolchain + install dest). Required
#                    unless WASI_SDK_DIR is set explicitly.
#   WASI_SDK_DIR     wasi-sdk dir (default: $SECURE_EXEC_DIR/registry/native/c/vendor/wasi-sdk)
#   TOOLCHAIN        rust toolchain (default: nightly-2026-03-01)
#   INSTALL          if "1" (default when SECURE_EXEC_DIR is set), copy artifacts into
#                    $SECURE_EXEC_DIR/registry/software/codex/wasm/{codex-exec,codex}
#   KEEP_SYSROOT     if "1", skip restoring the stashed prebuilt rlibs after the build (faster
#                    iteration; the sysroot stays in build-std-only mode).
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLCHAIN="${TOOLCHAIN:-nightly-2026-03-01}"
SECURE_EXEC_DIR="${SECURE_EXEC_DIR:-}"
WASI_SDK_DIR="${WASI_SDK_DIR:-${SECURE_EXEC_DIR:+$SECURE_EXEC_DIR/registry/native/c/vendor/wasi-sdk}}"
INSTALL="${INSTALL:-${SECURE_EXEC_DIR:+1}}"

[ -n "$WASI_SDK_DIR" ] || { echo "ERROR: set SECURE_EXEC_DIR or WASI_SDK_DIR"; exit 1; }
[ -x "$WASI_SDK_DIR/bin/clang" ] || { echo "ERROR: wasi-sdk clang not found at $WASI_SDK_DIR/bin/clang"; exit 1; }

SYSROOT="$(rustc "+$TOOLCHAIN" --print sysroot)"
LIBDIR="$SYSROOT/lib/rustlib/wasm32-wasip1/lib"
STASH="$LIBDIR/.prebuilt-stash"

echo "== ensure toolchain + target =="
rustup toolchain list | grep -q "$TOOLCHAIN" || rustup toolchain install "$TOOLCHAIN"
# The prebuilt target supplies the self-contained CRT objects that build-std linking needs.
[ -d "$LIBDIR/self-contained" ] || rustup "+$TOOLCHAIN" target add wasm32-wasip1

# -Z build-std injects its own libcore; if the prebuilt target's libcore is also on the sysroot link
# path the bin link fails E0152 "duplicate lang item core". Stash the prebuilt std rlibs/rmeta out of
# the way (keeping self-contained/ CRT) so build-std's core is the only one. Idempotent.
echo "== stash prebuilt std rlibs (keep self-contained CRT) =="
mkdir -p "$STASH"
shopt -s nullglob
for f in "$LIBDIR"/*.rlib "$LIBDIR"/*.rmeta; do mv "$f" "$STASH/"; done
shopt -u nullglob

WSDK="$WASI_SDK_DIR"
export CC_wasm32_wasip1="$WSDK/bin/clang"
export AR_wasm32_wasip1="$WSDK/bin/llvm-ar"
export CFLAGS_wasm32_wasip1="--sysroot=$WSDK/share/wasi-sysroot -D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_PTHREAD -D_WASI_EMULATED_MMAN -D_WASI_EMULATED_PROCESS_CLOCKS"

restore_sysroot() {
	[ "${KEEP_SYSROOT:-0}" = "1" ] && return 0
	shopt -s nullglob
	for f in "$STASH"/*; do mv "$f" "$LIBDIR/"; done
	shopt -u nullglob
	rmdir "$STASH" 2>/dev/null || true
}
trap restore_sysroot EXIT

cd "$REPO_DIR"

# First pass builds build-std's panic_abort into target deps. The bin resolves -Cpanic=abort's
# panic runtime from the *sysroot*, so we then copy build-std's ABI-matched panic_abort there.
echo "== build (build-std, panic=abort) =="
cargo "+$TOOLCHAIN" build -p codex-exec --release --target wasm32-wasip1 \
	-Z build-std --config 'profile.release.panic="abort"' 2>&1 | tail -20 || true

PA="$(ls target/wasm32-wasip1/release/deps/libpanic_abort-*.rlib 2>/dev/null | head -1 || true)"
if [ -n "$PA" ] && [ ! -e "$LIBDIR/$(basename "$PA")" ]; then
	echo "== place build-std panic_abort into sysroot =="
	cp "$PA" "$LIBDIR/"
	cp "${PA%.rlib}.rmeta" "$LIBDIR/" 2>/dev/null || true
	cargo "+$TOOLCHAIN" build -p codex-exec --release --target wasm32-wasip1 \
		-Z build-std --config 'profile.release.panic="abort"' 2>&1 | tail -20
fi

RAW="target/wasm32-wasip1/release/codex-exec.wasm"
[ -f "$RAW" ] || { echo "ERROR: no codex-exec.wasm artifact"; exit 1; }
echo "== built: $(stat -c%s "$RAW") bytes =="

OUT="$REPO_DIR/target/wasm32-wasip1/release/codex-exec.opt.wasm"
if command -v wasm-opt >/dev/null 2>&1; then
	wasm-opt --all-features -O2 "$RAW" -o "$OUT"
else
	echo "wasm-opt not found — shipping unoptimized"; cp "$RAW" "$OUT"
fi
echo "== optimized: $(stat -c%s "$OUT") bytes =="

if [ "${INSTALL:-0}" = "1" ]; then
	DEST="$SECURE_EXEC_DIR/registry/software/codex/wasm"
	mkdir -p "$DEST"
	cp "$OUT" "$DEST/codex-exec"
	cp "$OUT" "$DEST/codex"
	echo "== installed to $DEST/{codex-exec,codex} =="
fi
echo "DONE"
