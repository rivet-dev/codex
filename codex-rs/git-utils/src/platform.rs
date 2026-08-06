use std::path::Path;

use crate::GitToolingError;

#[cfg(unix)]
pub fn create_symlink(
    _source: &Path,
    link_target: &Path,
    destination: &Path,
) -> Result<(), GitToolingError> {
    use std::os::unix::fs::symlink;

    symlink(link_target, destination)?;
    Ok(())
}

#[cfg(windows)]
pub fn create_symlink(
    source: &Path,
    link_target: &Path,
    destination: &Path,
) -> Result<(), GitToolingError> {
    use std::os::windows::fs::FileTypeExt;
    use std::os::windows::fs::symlink_dir;
    use std::os::windows::fs::symlink_file;

    let metadata = std::fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink_dir() {
        symlink_dir(link_target, destination)?;
    } else {
        symlink_file(link_target, destination)?;
    }
    Ok(())
}

// wasm32-wasip1: std's `os::wasi::fs::symlink_path` is behind the unstable `wasi_ext`
// feature (unnamable from a stable crate). Git worktree symlink creation is not on the
// agent session-turn path inside the VM, so provide a compile-only Unsupported stub
// rather than pulling libc / enabling nightly features into codex-git.
#[cfg(target_os = "wasi")]
pub fn create_symlink(
    _source: &Path,
    _link_target: &Path,
    _destination: &Path,
) -> Result<(), GitToolingError> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "symlink creation is not supported in the secure-exec wasm VM",
    )
    .into())
}

#[cfg(not(any(unix, windows, target_os = "wasi")))]
compile_error!("codex-git symlink support is only implemented for Unix, Windows, and WASI");
