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
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::InitialHistory;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::ReviewDecision;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::SessionSource;
use codex_protocol::user_input::UserInput;
use serde_json::Value;
use serde_json::json;

/// Map the EE protocol `history` array (`[{ "role": "user"|"assistant", "content": "..." }]`) into
/// codex `RolloutItem`s for `InitialHistory::Forked`, so a resumed multi-turn session replays prior
/// context. Each turn of `codex-exec --session-turn` is stateless; the adapter sends the full prior
/// transcript here. Unknown roles are treated as user input.
fn history_from_start(start: &Value) -> Vec<RolloutItem> {
    let Some(entries) = start.get("history").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut items = Vec::with_capacity(entries.len());
    for entry in entries {
        let role = entry
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("user")
            .to_string();
        let text = entry
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if text.is_empty() {
            continue;
        }
        let content = if role == "assistant" {
            vec![ContentItem::OutputText { text }]
        } else {
            vec![ContentItem::InputText { text }]
        };
        items.push(RolloutItem::ResponseItem(ResponseItem::Message {
            id: None,
            role,
            content,
            end_turn: None,
            phase: None,
        }));
    }
    items
}

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
    // Surface codex-core's internal tracing to stderr (RUST_LOG-gated) so runtime
    // hangs in the agent loop are diagnosable inside the VM.
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("error")),
        )
        .try_init();
    // wasm32-wasip1 is single-threaded with no tokio::net reactor. codex-core drives the agent turn
    // on an internally `tokio::spawn`ed submission loop, so the runtime must cooperatively schedule
    // background tasks while our event loop awaits. The current-thread runtime does this correctly as
    // long as no task blocks the single executor thread (see the shell-snapshot wasi gate). Only the
    // time driver is needed; there is no I/O reactor on wasi (network I/O is host-brokered and
    // synchronous from the guest's perspective).
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
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
        (
            "features.shell_snapshot".to_string(),
            toml::Value::Boolean(false),
        ),
    ];
    let mut config = Config::load_with_cli_overrides(overrides)
        .await
        .context("load config")?;
    config.cwd = std::path::PathBuf::from(cwd);
    // wasm32-wasip1 has no tokio::net readiness reactor, so the WebSocket transport
    // (tokio-tungstenite → tokio::net::TcpStream) can't connect. Force the HTTP
    // Responses transport (host-brokered via wasi-http) by disabling websockets on
    // every provider; codex's HTTP path is the supported transport in the VM.
    for provider in config.model_providers.values_mut() {
        provider.supports_websockets = false;
    }

    let auth_manager = AuthManager::shared(
        config.codex_home.clone(),
        /*enable_codex_api_key_env*/ true,
        config.cli_auth_credentials_store_mode,
    );

    let auth_for_resume = AuthManager::shared(
        config.codex_home.clone(),
        /*enable_codex_api_key_env*/ true,
        config.cli_auth_credentials_store_mode,
    );
    let manager = ThreadManager::new(
        &config,
        auth_manager,
        SessionSource::Exec,
        Default::default(),
    );
    // Replay prior turns when the adapter sends `history`; otherwise start a fresh thread.
    let history = history_from_start(&start);
    let new_thread = if history.is_empty() {
        manager
            .start_thread(config.clone())
            .await
            .context("start thread")?
    } else {
        manager
            .resume_thread_with_history(
                config.clone(),
                InitialHistory::Forked(history),
                auth_for_resume,
                /*persist_extended_history*/ false,
                /*parent_trace*/ None,
            )
            .await
            .context("resume thread with history")?
    };
    let thread = new_thread.thread;

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

    // Whether the current assistant message streamed any deltas. Codex streams
    // text via AgentMessageDelta when the model streams; when it does not (the
    // model returns a complete message), only the final AgentMessage event
    // carries the text. Emit the final message as a text_delta in that case so
    // the EE stream always carries the assistant text, without double-emitting
    // when deltas were already streamed.
    let mut saw_delta = false;
    loop {
        let Event { id, msg } = thread.next_event().await.context("next_event")?;
        match msg {
            EventMsg::AgentMessageDelta(d) => {
                saw_delta = true;
                emit(json!({ "type": "text_delta", "delta": d.delta }));
            }
            EventMsg::AgentMessage(m) => {
                if !saw_delta && !m.message.is_empty() {
                    emit(json!({ "type": "text_delta", "delta": m.message }));
                }
                saw_delta = false;
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
