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

    // Look for the agentpulse marker. Antigravity nests its named hooks one
    // level deeper (root → "agentpulse" → event), everyone else uses "hooks".
    let hooks_obj = if provider_id == "antigravity" {
        json.get("agentpulse").unwrap_or(&json)
    } else {
        json.get("hooks").unwrap_or(&json)
    };
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
                        if cmd.contains(MARKER) {
                            return false;
                        }
                    }
                }
            }
        }
    }

    true
}

// Substring that uniquely identifies AgentPulse-installed hooks. Matches
// the sidecar binary filename across all shells + OSes (agent-pulse-hook
// on unix, agent-pulse-hook.exe on windows). Previously "agentpulse" —
// which never matched anything, because the binary name is hyphenated.
const MARKER: &str = "agent-pulse-hook";

/// Absolute path to the sidecar binary, expected next to the main exe.
/// Shipping a binary (not a shell one-liner) keeps hook commands
/// shell-agnostic across bash / PowerShell / cmd.exe.
fn sidecar_path() -> PathBuf {
    let exe_name = if cfg!(windows) {
        "agent-pulse-hook.exe"
    } else {
        "agent-pulse-hook"
    };
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(exe_name)))
        .unwrap_or_else(|| PathBuf::from(exe_name))
}

/// Build the hook command string. The sidecar reads stdin + port file
/// itself, so no shell substitution is needed.
fn hook_cmd(provider_id: &str) -> String {
    format!("\"{}\" {provider_id}", sidecar_path().display())
}

/// Like `hook_cmd` but appends the event name as a 2nd arg. Antigravity's
/// stdin payload carries no event-name field (the event is implied by the
/// hooks.json key), so we pass it explicitly and the sidecar injects it before
/// POSTing. agy runs hook commands via `sh -c` / `cmd /c`, so no PowerShell
/// call-operator dance is needed here.
fn hook_cmd_ev(provider_id: &str, event: &str) -> String {
    format!("\"{}\" {provider_id} {event}", sidecar_path().display())
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

    // Antigravity CLI stores named hooks at the root of hooks.json (no "hooks"
    // wrapper). AgentPulse owns the "agentpulse" named hook — drop it wholesale.
    if provider_id == "antigravity" {
        if let Some(obj) = root.as_object_mut() {
            obj.remove("agentpulse");
        }
    }

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
                        cmd_str.contains(MARKER) || bash_str.contains(MARKER)
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
pub fn install_provider(provider_id: &str, config: &ProviderConfig) -> Result<(), String> {
    // Clean up any existing AgentPulse hooks first
    let _ = remove_provider(provider_id, config);

    let path = match &config.settings_path {
        Some(p) => expand_path(p),
        None => return Err(format!("No settings path for provider {provider_id}")),
    };

    match provider_id {
        "claude" => install_claude_hooks(&path),
        "antigravity" => install_antigravity_hooks(&path),
        "codex" => install_codex_hooks(&path),
        "copilot" => install_copilot_hooks(&path),
        _ => Err(format!("Unknown provider: {provider_id}")),
    }
}

/// Claude Code: hooks in ~/.claude/settings.json
fn install_claude_hooks(path: &PathBuf) -> Result<(), String> {
    let mut root = load_or_create_json(path)?;

    let hooks = root
        .as_object_mut()
        .ok_or("settings.json root is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));

    let cmd = hook_cmd("claude");

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
    info!("Claude Code hooks configured");
    Ok(())
}

/// Antigravity CLI (agy): named hooks in ~/.gemini/config/hooks.json.
///
/// agy's hook model differs from every other provider:
///   1. Config is a separate `hooks.json` (not settings.json), keyed by a
///      hook *name* → event → handlers (one extra level of nesting).
///   2. Only 5 events exist: PreToolUse / PostToolUse / PreInvocation /
///      PostInvocation / Stop. No SessionStart/SessionEnd/UserPromptSubmit.
///   3. Hooks are SYNCHRONOUS — each must print a JSON result on stdout, and
///      the stdin payload has no event-name field (event = the config key).
///
/// So we register under a single AgentPulse-owned name, pass the event to the
/// sidecar as a 2nd arg (it injects `hook_event_name` before POSTing), and only
/// wire events whose stdout contract is satisfied by an empty `{}`:
///   - PreInvocation → fires before each model call (→ SessionStart / Working)
///   - PostToolUse   → keeps the session Working
///   - Stop          → turn end (→ Completed)
/// PreToolUse is skipped on purpose: it demands a `decision` field, and a bare
/// `{}` would misgate tool execution. No `async` flag — agy blocks on hooks.
fn install_antigravity_hooks(path: &PathBuf) -> Result<(), String> {
    let mut root = load_or_create_json(path)?;

    // Tool-scoped events use the grouped matcher+hooks shape; lifecycle events
    // use the flat handler-list shape (per agy's hooks.json spec).
    let group = |event: &str| json!([{
        "matcher": "",
        "hooks": [{ "type": "command", "command": hook_cmd_ev("antigravity", event) }]
    }]);
    let flat = |event: &str| json!([{ "type": "command", "command": hook_cmd_ev("antigravity", event) }]);

    let spec = json!({
        "PreInvocation": flat("PreInvocation"),
        "PostToolUse": group("PostToolUse"),
        "Stop": flat("Stop"),
    });

    // Overwrite the whole AgentPulse-owned named hook — idempotent, no dup
    // accumulation on repeated installs.
    root.as_object_mut()
        .ok_or("hooks.json root is not an object")?
        .insert("agentpulse".into(), spec);

    save_json(path, &root)?;
    info!("Antigravity CLI hooks configured");
    Ok(())
}

/// Codex CLI: hooks in ~/.codex/hooks.json + enable feature flag in config.toml
fn install_codex_hooks(path: &PathBuf) -> Result<(), String> {
    // 1. Enable hooks feature flag in config.toml. Codex v0.129 renamed
    //    `codex_hooks` to `hooks`; the old name still works but prints a
    //    deprecation warning on every launch, so migrate eagerly.
    let config_toml = path.parent()
        .ok_or("Invalid hooks.json path")?
        .join("config.toml");

    if config_toml.exists() {
        let original = std::fs::read_to_string(&config_toml).map_err(|e| e.to_string())?;
        let updated = ensure_codex_hooks_feature(&original);
        if updated != original {
            std::fs::write(&config_toml, updated).map_err(|e| e.to_string())?;
            info!("Updated [features].hooks in Codex config.toml");
        }
    }

    // 2. Write hooks.json
    let cmd = hook_cmd("codex");

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
    info!("Codex CLI hooks configured");
    Ok(())
}

/// GitHub Copilot CLI: hooks in ~/.copilot/config.json
fn install_copilot_hooks(path: &PathBuf) -> Result<(), String> {
    let mut root = load_or_create_json(path)?;

    let hooks = root
        .as_object_mut()
        .ok_or("config.json root is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));

    let cmd = hook_cmd("copilot");

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
    info!("GitHub Copilot CLI hooks configured");
    Ok(())
}

/// Ensure `[features].hooks = true` exists in Codex config.toml, migrating
/// any pre-v0.129 `codex_hooks = true` line in place. We avoid pulling in
/// a TOML serializer so user comments and formatting survive.
fn ensure_codex_hooks_feature(original: &str) -> String {
    let key_of = |line: &str| -> Option<String> {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            return None;
        }
        let key = trimmed.split('=').next()?.trim();
        if key == "codex_hooks" || key == "hooks" {
            Some(key.to_string())
        } else {
            None
        }
    };

    let mut seen = false;
    let mut lines: Vec<String> = original
        .lines()
        .filter_map(|line| {
            match key_of(line) {
                Some(_) if seen => None, // drop duplicate flag lines
                Some(key) => {
                    seen = true;
                    if key == "codex_hooks" {
                        let lead_len = line.len() - line.trim_start().len();
                        Some(format!("{}hooks = true", &line[..lead_len]))
                    } else {
                        Some(line.to_string())
                    }
                }
                None => Some(line.to_string()),
            }
        })
        .collect();

    if !seen {
        // Insert into existing [features] table or append a new one
        if let Some(idx) = lines.iter().position(|l| l.trim() == "[features]") {
            lines.insert(idx + 1, "hooks = true".to_string());
        } else {
            if !lines.last().map_or(true, |l| l.is_empty()) {
                lines.push(String::new());
            }
            lines.push("[features]".to_string());
            lines.push("hooks = true".to_string());
        }
    }

    let mut out = lines.join("\n");
    if original.ends_with('\n') && !out.ends_with('\n') {
        out.push('\n');
    }
    out
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

#[cfg(test)]
mod tests {
    use super::ensure_codex_hooks_feature;

    #[test]
    fn migrates_deprecated_codex_hooks_in_place() {
        let input = "model_provider = \"azure\"\n\n[features]\ncodex_hooks = true\n";
        let out = ensure_codex_hooks_feature(input);
        assert!(out.contains("hooks = true"));
        assert!(!out.contains("codex_hooks"));
        assert!(out.starts_with("model_provider"));
    }

    #[test]
    fn leaves_existing_hooks_flag_alone() {
        let input = "[features]\nhooks = true\n";
        assert_eq!(ensure_codex_hooks_feature(input), input);
    }

    #[test]
    fn appends_features_table_when_missing() {
        let input = "model_provider = \"azure\"\n";
        let out = ensure_codex_hooks_feature(input);
        assert!(out.contains("[features]\nhooks = true"));
    }

    #[test]
    fn inserts_into_existing_features_table() {
        let input = "[features]\nweb_search = true\n";
        let out = ensure_codex_hooks_feature(input);
        assert!(out.contains("[features]\nhooks = true\nweb_search = true"));
    }

    #[test]
    fn ignores_commented_lines_and_unrelated_keys() {
        let input = "# codex_hooks = true\n[features]\nhooks_dir = \"x\"\n";
        let out = ensure_codex_hooks_feature(input);
        assert!(out.contains("[features]\nhooks = true\nhooks_dir = \"x\""));
        assert!(out.contains("# codex_hooks = true"));
    }

    #[test]
    fn dedupes_when_both_flags_present() {
        let input = "[features]\ncodex_hooks = true\nhooks = true\n";
        let out = ensure_codex_hooks_feature(input);
        let count = out.matches("hooks = true").count();
        assert_eq!(count, 1, "expected one hooks line, got: {out}");
        assert!(!out.contains("codex_hooks"));
    }
}
