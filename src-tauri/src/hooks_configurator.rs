use log::info;
use serde_json::{json, Value};
use std::path::PathBuf;

fn settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
}

pub fn needs_setup() -> bool {
    let path = match settings_path() {
        Some(p) => p,
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
                            if cmd.contains("ccani") || cmd.contains("1928") {
                                return false;
                            }
                        }
                        if let Some(url) = hook.get("url").and_then(|v| v.as_str()) {
                            if url.contains("1928") {
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

pub fn install(port: u16) -> Result<(), String> {
    let path = settings_path().ok_or("Cannot determine home directory")?;

    let mut root: Value = if path.exists() {
        let data = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|_| {
            "~/.claude/settings.json contains malformed JSON. Please fix it manually.".to_string()
        })?
    } else {
        json!({})
    };

    let hooks = root
        .as_object_mut()
        .ok_or("settings.json root is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));

    let curl_cmd = format!(
        "curl -sf -m 2 -X POST -H 'Content-Type: application/json' \
         -d \"$(cat)\" http://localhost:$(cat ~/.ccani/port 2>/dev/null || echo {port})/hook || true"
    );

    let all_events = [
        "SessionStart",
        "SessionEnd",
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "PostToolUseFailure",
        "PermissionRequest",
        "Stop",
    ];

    for event in all_events {
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

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let formatted = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    std::fs::write(&path, formatted).map_err(|e| e.to_string())?;

    info!("Claude Code hooks configured on port {port}");
    Ok(())
}
