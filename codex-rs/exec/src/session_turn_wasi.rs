//! wasm32-wasip1 session-turn engine.
//!
//! Drives the REAL codex-core agent (no bespoke provider loop) and speaks the
//! secure-exec EE newline-JSON protocol that `@rivet-ee/agent-os-codex-agent`
//! expects from `codex-exec --session-turn`:
//!   stdin:  {type:"start", cwd, model, prompt, ...}  then
//!           {type:"permission_response", decision}
//!   stdout: {type:"start"} / {type:"text_delta",delta} /
//!           {type:"tool_call_update",tool_call_id,status} /
//!           {type:"permission_request",tool_call_id} / {type:"done"} /
//!           {type:"error",message}

use std::io::Write;

use anyhow::Context as _;
use codex_core::AuthManager;
use codex_core::ThreadManager;
use codex_core::config::Config;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::protocol::SessionSource;
use codex_protocol::user_input::UserInput;
use serde_json::Value;
use serde_json::json;

fn emit(v: Value) {
    let mut out = std::io::stdout();
    let _ = writeln!(out, "{v}");
    let _ = out.flush();
}

fn read_line() -> Option<String> {
    let mut s = String::new();
    match std::io::stdin().read_line(&mut s) {
        Ok(0) => None,
        Ok(_) => Some(s),
        Err(_) => None,
    }
}

pub fn run() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(session_turn())
}

async fn session_turn() -> anyhow::Result<()> {
    let start_raw = read_line().context("no start message on stdin")?;
    let start: Value = serde_json::from_str(start_raw.trim()).context("bad start json")?;
    let model = start["model"].as_str().unwrap_or("gpt-5").to_string();
    let cwd = start["cwd"].as_str().unwrap_or(".").to_string();
    let prompt = start["prompt"].as_str().unwrap_or("").to_string();

    emit(json!({ "type": "start" }));

    let overrides = vec![
        ("model".to_string(), toml::Value::String(model)),
        (
            "approval_policy".to_string(),
            toml::Value::String("on-request".to_string()),
        ),
    ];
    eprintln!("DBG: loading config");
    let mut config = Config::load_with_cli_overrides(overrides)
        .await
        .context("load config")?;
    config.cwd = std::path::PathBuf::from(cwd);
    eprintln!("DBG: config loaded; creating auth");

    let auth_manager = AuthManager::shared(
        config.codex_home.clone(),
        /*enable_codex_api_key_env*/ true,
        config.cli_auth_credentials_store_mode,
    );
    eprintln!("DBG: auth created; ThreadManager::new");

    let manager = ThreadManager::new(
        &config,
        auth_manager,
        SessionSource::Exec,
        Default::default(),
    );
    eprintln!("DBG: manager created; start_thread");
    let new_thread = manager
        .start_thread(config.clone())
        .await
        .context("start thread")?;
    let thread = new_thread.thread;
    eprintln!("DBG: thread started; submitting prompt");

    thread
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: prompt,
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
        })
        .await
        .context("submit prompt")?;
    eprintln!("DBG: prompt submitted; entering event loop");

    loop {
        eprintln!("DBG: awaiting next_event");
        let Event { id, msg } = thread.next_event().await.context("next_event")?;
        eprintln!("DBG: got event: {msg:?}");
        match msg {
            EventMsg::AgentMessageDelta(d) => {
                emit(json!({ "type": "text_delta", "delta": d.delta }));
            }
            EventMsg::ExecCommandBegin(e) => {
                emit(json!({
                    "type": "tool_call_update",
                    "tool_call_id": e.call_id,
                    "status": "in_progress",
                }));
            }
            EventMsg::ExecCommandEnd(e) => {
                emit(json!({
                    "type": "tool_call_update",
                    "tool_call_id": e.call_id,
                    "status": "completed",
                }));
            }
            EventMsg::ExecApprovalRequest(e) => {
                emit(json!({
                    "type": "permission_request",
                    "tool_call_id": e.call_id,
                }));
                let resp = read_line().unwrap_or_default();
                let resp: Value = serde_json::from_str(resp.trim()).unwrap_or_else(|_| json!({}));
                let decision = match resp["decision"].as_str() {
                    Some("allow") | Some("approve") | Some("allow_always") => {
                        ReviewDecision::Approved
                    }
                    _ => ReviewDecision::Denied,
                };
                let _ = thread
                    .submit(Op::ExecApproval {
                        id: id.clone(),
                        turn_id: None,
                        decision,
                    })
                    .await;
            }
            EventMsg::TurnComplete(_) => {
                emit(json!({ "type": "done" }));
                break;
            }
            EventMsg::Error(err) => {
                emit(json!({ "type": "error", "message": format!("{err:?}") }));
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

/// wasm32-wasip1: sqlx's bundled sqlite is built without extension loading, but the
/// bindings still reference `sqlite3_load_extension`, leaving it as an unresolved host
/// import (`env.sqlite3_load_extension`) that the secure-exec VM runtime doesn't
/// provide. Define a no-op so the module instantiates; codex never loads sqlite
/// extensions, so returning SQLITE_ERROR is correct.
#[unsafe(no_mangle)]
pub extern "C" fn sqlite3_load_extension(
    _db: *mut core::ffi::c_void,
    _z_file: *const core::ffi::c_char,
    _z_proc: *const core::ffi::c_char,
    _pz_err_msg: *mut *mut core::ffi::c_char,
) -> core::ffi::c_int {
    1 // SQLITE_ERROR
}
