use crate::config::{expand_path, ProviderConfig};
use log::info;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Check if a provider's hooks are already configured
pub fn provider_needs_setup(provider_id: &str, config: &ProviderConfig) -> bool {
    let path = match &config.settings_path {
        Some(p) => expand_path(p),
        None => return true,
    };

    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return true,
    };

    let json: Value = match serde_json::from_str(&data) {
        Ok(v) => v,
        Err(_) => return true,
    };

    // Look for agentpulse marker in hooks
    let hooks = match json.get("hooks") {
        Some(Value::Object(h)) => h,
        _ => return true,
    };

    for (_event, entries) in hooks {
        if let Value::Array(entries) = entries {
            for entry in entries {
                if let Some(Value::Array(hook_list)) = entry.get("hooks") {
                    for hook in hook_list {
                        if let Some(cmd) = hook.get("command").and_then(|v| v.as_str()) {
                            if cmd.contains("agentpulse") && cmd.contains(provider_id) {
                                return false;
                            }
                        }
                    }
                }
            }
        }
    }

    true
}

/// Install hooks for a provider
pub fn install_provider(provider_id: &str, config: &ProviderConfig, port: u16) -> Result<(), String> {
    let path = match &config.settings_path {
        Some(p) => expand_path(p),
        None => return Err(format!("No settings path for provider {provider_id}")),
    };

    match provider_id {
        "claude" => install_claude_hooks(&path, port),
        "gemini" => install_gemini_hooks(&path, port),
        "codex" => install_codex_hooks(&path, port),
        _ => Err(format!("Unknown provider: {provider_id}")),
    }
}

/// Claude Code: hooks in ~/.claude/settings.json
fn install_claude_hooks(path: &PathBuf, port: u16) -> Result<(), String> {
    let mut root = load_or_create_json(path)?;

    let hooks = root
        .as_object_mut()
        .ok_or("settings.json root is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));

    let curl_cmd = format!(
        "curl -sf -m 2 -X POST -H 'Content-Type: application/json' \
         -d \"$(cat)\" http://localhost:$(cat ~/.agentpulse/port 2>/dev/null || echo {port})/hook/claude || true"
    );

    let events = [
        "SessionStart", "SessionEnd", "UserPromptSubmit",
        "PreToolUse", "PostToolUse", "PostToolUseFailure",
        "PermissionRequest", "Stop",
    ];

    for event in events {
        let entry = json!({
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": curl_cmd,
                "async": true
            }]
        });

        let event_hooks = hooks
            .as_object_mut()
            .ok_or("hooks is not an object")?
            .entry(event)
            .or_insert_with(|| json!([]));

        if let Value::Array(ref mut arr) = event_hooks {
            arr.push(entry);
        }
    }

    save_json(path, &root)?;
    info!("Claude Code hooks configured on port {port}");
    Ok(())
}

/// Gemini CLI: hooks in ~/.gemini/settings.json
fn install_gemini_hooks(path: &PathBuf, port: u16) -> Result<(), String> {
    let mut root = load_or_create_json(path)?;

    let hooks = root
        .as_object_mut()
        .ok_or("settings.json root is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));

    let curl_cmd = format!(
        "curl -sf -m 2 -X POST -H 'Content-Type: application/json' \
         -d \"$(cat)\" http://localhost:$(cat ~/.agentpulse/port 2>/dev/null || echo {port})/hook/gemini || true"
    );

    // Gemini CLI hook events
    let events = [
        "SessionStart", "SessionEnd",
        "BeforeAgent", "AfterAgent",
        "BeforeModel", "AfterModel",
        "BeforeTool", "AfterTool",
        "Notification",
    ];

    for event in events {
        let entry = json!({
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": curl_cmd,
                "async": true
            }]
        });

        let event_hooks = hooks
            .as_object_mut()
            .ok_or("hooks is not an object")?
            .entry(event)
            .or_insert_with(|| json!([]));

        if let Value::Array(ref mut arr) = event_hooks {
            arr.push(entry);
        }
    }

    save_json(path, &root)?;
    info!("Gemini CLI hooks configured on port {port}");
    Ok(())
}

/// Codex CLI: hooks in project-level hooks.json
fn install_codex_hooks(_path: &PathBuf, _port: u16) -> Result<(), String> {
    // TODO: Codex uses hooks.json files, implementation pending
    Err("Codex CLI hook installation not yet implemented".into())
}

fn load_or_create_json(path: &PathBuf) -> Result<Value, String> {
    if path.exists() {
        let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| format!("{} contains malformed JSON: {e}", path.display()))
    } else {
        Ok(json!({}))
    }
}

fn save_json(path: &PathBuf, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let formatted = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, formatted).map_err(|e| e.to_string())?;
    Ok(())
}
