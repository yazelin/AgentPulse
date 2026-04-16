use crate::config::{expand_path, ProviderConfig};
use log::info;
use serde_json::{json, Value};
use std::path::PathBuf;

/// Check if a provider's hooks are already configured
pub fn provider_needs_setup(_provider_id: &str, config: &ProviderConfig) -> bool {
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
    let hooks_obj = json.get("hooks").unwrap_or(&json);
    let hooks = match hooks_obj {
        Value::Object(h) => h,
        _ => return true,
    };

    for (_event, entries) in hooks {
        if let Value::Array(entries) = entries {
            for entry in entries {
                // Check both nested hooks array and direct hook objects
                let hook_list = if let Some(Value::Array(hl)) = entry.get("hooks") {
                    hl.clone()
                } else {
                    vec![entry.clone()]
                };

                for hook in &hook_list {
                    if let Some(cmd) = hook.get("command").and_then(|v| v.as_str()) {
                        if cmd.contains("agentpulse") {
                            return false;
                        }
                    }
                }
            }
        }
    }

    true
}

/// Generate curl command that captures the terminal window ID via process tree
/// Walks up the parent process tree to find a PID with an associated X11 window
fn curl_cmd(provider_id: &str, port: u16) -> String {
    // Shell snippet: walk up $PPID chain, find first PID with xdotool-searchable window
    let find_window = r#"$(p=$PPID; w=""; while [ "$p" -gt 1 ]; do w=$(xdotool search --pid $p 2>/dev/null | head -1); [ -n "$w" ] && break; p=$(awk '{print $4}' /proc/$p/stat 2>/dev/null); [ -z "$p" ] && break; done; echo "$w")"#;

    format!(
        "curl -sf -m 2 -X POST \
         -H 'Content-Type: application/json' \
         -H \"X-Window-Id: {find_window}\" \
         -d \"$(cat)\" \
         http://localhost:$(cat ~/.agentpulse/port 2>/dev/null || echo {port})/hook/{provider_id} || true"
    )
}

/// Remove only AgentPulse hooks (those containing "agentpulse" string) from a provider's config
pub fn remove_provider(provider_id: &str, config: &ProviderConfig) -> Result<(), String> {
    let path = match &config.settings_path {
        Some(p) => expand_path(p),
        None => return Ok(()),
    };

    if !path.exists() {
        return Ok(());
    }

    let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut root: Value = serde_json::from_str(&data)
        .map_err(|e| format!("malformed JSON in {}: {e}", path.display()))?;

    if let Some(Value::Object(hooks)) = root.get_mut("hooks") {
        for (_event, entries) in hooks.iter_mut() {
            if let Value::Array(arr) = entries {
                arr.retain(|entry| {
                    // Check if this entry contains an agentpulse hook
                    let hook_list = if let Some(Value::Array(hl)) = entry.get("hooks") {
                        hl.clone()
                    } else {
                        vec![entry.clone()]
                    };
                    !hook_list.iter().any(|h| {
                        let cmd_str = h.get("command").and_then(|v| v.as_str()).unwrap_or("");
                        let bash_str = h.get("bash").and_then(|v| v.as_str()).unwrap_or("");
                        cmd_str.contains("agentpulse") || bash_str.contains("agentpulse")
                    })
                });
            }
        }
    }

    let formatted = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    std::fs::write(&path, formatted).map_err(|e| e.to_string())?;
    info!("Removed AgentPulse hooks for {provider_id}");
    Ok(())
}

/// Install hooks for a provider (removes existing AgentPulse hooks first to avoid duplicates)
pub fn install_provider(provider_id: &str, config: &ProviderConfig, port: u16) -> Result<(), String> {
    // Clean up any existing AgentPulse hooks first
    let _ = remove_provider(provider_id, config);

    let path = match &config.settings_path {
        Some(p) => expand_path(p),
        None => return Err(format!("No settings path for provider {provider_id}")),
    };

    match provider_id {
        "claude" => install_claude_hooks(&path, port),
        "gemini" => install_gemini_hooks(&path, port),
        "codex" => install_codex_hooks(&path, port),
        "copilot" => install_copilot_hooks(&path, port),
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

    let cmd = curl_cmd("claude", port);

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
                "command": cmd,
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

    let cmd = curl_cmd("gemini", port);

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
                "command": cmd,
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

/// Codex CLI: hooks in ~/.codex/hooks.json + enable feature flag in config.toml
fn install_codex_hooks(path: &PathBuf, port: u16) -> Result<(), String> {
    // 1. Enable codex_hooks feature flag in config.toml
    let config_toml = path.parent()
        .ok_or("Invalid hooks.json path")?
        .join("config.toml");

    if config_toml.exists() {
        let mut content = std::fs::read_to_string(&config_toml).map_err(|e| e.to_string())?;
        if !content.contains("codex_hooks") {
            // Add [features] section with codex_hooks = true
            if content.contains("[features]") {
                content = content.replace("[features]", "[features]\ncodex_hooks = true");
            } else {
                content.push_str("\n[features]\ncodex_hooks = true\n");
            }
            std::fs::write(&config_toml, content).map_err(|e| e.to_string())?;
            info!("Enabled codex_hooks feature flag in config.toml");
        }
    }

    // 2. Write hooks.json
    let cmd = curl_cmd("codex", port);

    let hooks_json = json!({
        "hooks": {
            "SessionStart": [{
                "hooks": [{ "type": "command", "command": cmd }]
            }],
            "UserPromptSubmit": [{
                "hooks": [{ "type": "command", "command": cmd }]
            }],
            "PreToolUse": [{
                "matcher": "",
                "hooks": [{ "type": "command", "command": cmd }]
            }],
            "PostToolUse": [{
                "matcher": "",
                "hooks": [{ "type": "command", "command": cmd }]
            }],
            "Stop": [{
                "hooks": [{ "type": "command", "command": cmd }]
            }]
        }
    });

    save_json(path, &hooks_json)?;
    info!("Codex CLI hooks configured on port {port}");
    Ok(())
}

/// GitHub Copilot CLI: hooks in ~/.copilot/config.json
fn install_copilot_hooks(path: &PathBuf, port: u16) -> Result<(), String> {
    let mut root = load_or_create_json(path)?;

    let hooks = root
        .as_object_mut()
        .ok_or("config.json root is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));

    let cmd = curl_cmd("copilot", port);

    let events = [
        "sessionStart", "sessionEnd", "userPromptSubmitted",
        "preToolUse", "postToolUse", "agentStop",
    ];

    for event in events {
        let hook_entry = json!({
            "type": "command",
            "bash": cmd
        });

        let event_hooks = hooks
            .as_object_mut()
            .ok_or("hooks is not an object")?
            .entry(event)
            .or_insert_with(|| json!([]));

        if let Value::Array(ref mut arr) = event_hooks {
            arr.push(hook_entry);
        }
    }

    save_json(path, &root)?;
    info!("GitHub Copilot CLI hooks configured on port {port}");
    Ok(())
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
