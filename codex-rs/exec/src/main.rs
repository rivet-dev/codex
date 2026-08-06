//! Entry-point for the `codex-exec` binary.
//!
//! When this CLI is invoked normally, it parses the standard `codex-exec` CLI
//! options and launches the non-interactive Codex agent. However, if it is
//! invoked with arg0 as `codex-linux-sandbox`, we instead treat the invocation
//! as a request to run the logic for the standalone `codex-linux-sandbox`
//! executable (i.e., parse any -s args and then run a *sandboxed* command under
//! Landlock + seccomp.
//!
//! This allows us to ship a completely separate set of functionality as part
//! of the `codex-exec` binary.
#[cfg(target_os = "wasi")]
fn main() -> anyhow::Result<()> {
    codex_exec::wasi_stub_main()
}

#[cfg(not(target_os = "wasi"))]
fn main() -> anyhow::Result<()> {
    use clap::Parser;
    use codex_arg0::Arg0DispatchPaths;
    use codex_arg0::arg0_dispatch_or_else;
    use codex_exec::Cli;
    use codex_exec::run_main;
    use codex_utils_cli::CliConfigOverrides;

    #[derive(Parser, Debug)]
    struct TopCli {
        #[clap(flatten)]
        config_overrides: CliConfigOverrides,

        #[clap(flatten)]
        inner: Cli,
    }

    arg0_dispatch_or_else(|arg0_paths: Arg0DispatchPaths| async move {
        let top_cli = TopCli::parse();
        // Merge root-level overrides into inner CLI struct so downstream logic remains unchanged.
        let mut inner = top_cli.inner;
        inner
            .config_overrides
            .prepend_root_overrides(top_cli.config_overrides);

        run_main(inner, arg0_paths).await?;
        Ok(())
    })
}

#[cfg(all(test, not(target_os = "wasi")))]
#[path = "main_tests.rs"]
mod tests;
