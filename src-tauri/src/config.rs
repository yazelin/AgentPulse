use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub setup_done: bool,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default = "default_providers")]
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_accent")]
    pub accent_color: String,
    #[serde(default = "default_text_size")]
    pub text_size: String,
    #[serde(default)]
    pub pin_expanded: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub sound_enabled: bool,
    /// Per-provider sound played on task completion.
    #[serde(default)]
    pub provider_sounds: std::collections::HashMap<String, String>,
    /// Per-provider sound played when a session transitions to WaitingForUser.
    #[serde(default)]
    pub provider_waiting_sounds: std::collections::HashMap<String, String>,
    /// Legacy field, kept for backward compat
    #[serde(default)]
    pub sound_name: String,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            accent_color: default_accent(),
            text_size: default_text_size(),
            theme: default_theme(),
            pin_expanded: false,
            sound_enabled: false,
            provider_sounds: std::collections::HashMap::new(),
            provider_waiting_sounds: std::collections::HashMap::new(),
            sound_name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub name: String,
    #[serde(default)]
    pub settings_path: Option<String>,
}

fn default_accent() -> String { "purple".into() }
fn default_text_size() -> String { "medium".into() }
fn default_theme() -> String { "dark".into() }

fn default_providers() -> HashMap<String, ProviderConfig> {
    let mut m = HashMap::new();
    m.insert("claude".into(), ProviderConfig {
        enabled: false,
        name: "Claude Code".into(),
        settings_path: Some("~/.claude/settings.json".into()),
    });
    m.insert("antigravity".into(), ProviderConfig {
        enabled: false,
        name: "Antigravity CLI".into(),
        settings_path: Some("~/.gemini/config/hooks.json".into()),
    });
    m.insert("copilot".into(), ProviderConfig {
        enabled: false,
        name: "GitHub Copilot".into(),
        settings_path: Some("~/.copilot/config.json".into()),
    });
    m.insert("codex".into(), ProviderConfig {
        enabled: false,
        name: "Codex CLI".into(),
        settings_path: Some("~/.codex/hooks.json".into()),
    });
    m
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            setup_done: false,
            appearance: AppearanceConfig::default(),
            providers: default_providers(),
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
        .join("agentpulse")
        .join("config.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    let mut config = if let Ok(data) = std::fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AppConfig::default()
    };
    reconcile_providers(&mut config);
    config
}

/// Migrate a persisted config to the current provider set: drop retired
/// providers (e.g. Gemini → Antigravity) and add newly introduced ones,
/// preserving the user's `enabled` state for providers that persist.
///
/// `#[serde(default)]` on `providers` only fires when the key is entirely
/// absent, so without this an existing install keeps a dead `gemini` entry
/// (whose toggle now errors) and never sees `antigravity` in the UI — so the
/// user could never enable it or install its hooks.
fn reconcile_providers(config: &mut AppConfig) {
    let known = default_providers();
    config.providers.retain(|id, _| known.contains_key(id));
    for (id, def) in known {
        config.providers.entry(id).or_insert(def);
    }
    // Drop sound-map entries for removed providers, then seed a default sound
    // for any provider that lacks one. Without the seed, a newly added/migrated
    // provider (e.g. antigravity) stays silent on completion until the user
    // opens the Sounds tab — the JS auto-match only runs when that UI renders.
    // play_sound_file no-ops if the file is missing, so seeding `{id}.mp3` is
    // safe even when a custom provider has no bundled clip.
    let ids: Vec<String> = config.providers.keys().cloned().collect();
    config.appearance.provider_sounds.retain(|id, _| ids.contains(id));
    config.appearance.provider_waiting_sounds.retain(|id, _| ids.contains(id));
    for id in &ids {
        config.appearance.provider_sounds
            .entry(id.clone())
            .or_insert_with(|| format!("{id}.mp3"));
        config.appearance.provider_waiting_sounds
            .entry(id.clone())
            .or_insert_with(|| format!("{id}-waiting.mp3"));
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, data).map_err(|e| e.to_string())?;
    Ok(())
}

/// Expand ~ to home dir
pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Detect which providers are installed on this system
pub fn detect_providers() -> HashMap<String, bool> {
    let mut detected = HashMap::new();

    // Claude: check if ~/.claude/ exists
    detected.insert("claude".into(), dirs::home_dir()
        .map(|h| h.join(".claude").exists())
        .unwrap_or(false));

    // Antigravity CLI (agy): check if the `agy` binary exists or its config
    // dir ~/.gemini/antigravity-cli/ is present.
    detected.insert("antigravity".into(),
        which_exists("agy") ||
        dirs::home_dir().map(|h| h.join(".gemini").join("antigravity-cli").exists()).unwrap_or(false));

    // Copilot: check if gh copilot or copilot binary exists
    detected.insert("copilot".into(), which_exists("copilot"));

    // Codex: check if codex binary exists
    detected.insert("codex".into(), which_exists("codex"));

    detected
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconcile_drops_gemini_and_adds_antigravity_preserving_enabled() {
        // Simulate a persisted pre-0.4.0 config: gemini present + enabled,
        // antigravity absent, claude enabled.
        let mut cfg = AppConfig::default();
        cfg.providers.clear();
        cfg.providers.insert("claude".into(), ProviderConfig {
            enabled: true, name: "Claude Code".into(),
            settings_path: Some("~/.claude/settings.json".into()),
        });
        cfg.providers.insert("gemini".into(), ProviderConfig {
            enabled: true, name: "Gemini CLI".into(),
            settings_path: Some("~/.gemini/settings.json".into()),
        });
        cfg.appearance.provider_sounds.insert("gemini".into(), "gemini.mp3".into());
        // A user who explicitly muted claude's completion sound.
        cfg.appearance.provider_sounds.insert("claude".into(), "__none__".into());

        reconcile_providers(&mut cfg);

        assert!(!cfg.providers.contains_key("gemini"), "retired gemini should be dropped");
        assert!(cfg.providers.contains_key("antigravity"), "antigravity should be added");
        assert_eq!(cfg.providers["antigravity"].settings_path.as_deref(),
                   Some("~/.gemini/config/hooks.json"));
        assert!(cfg.providers["claude"].enabled, "existing enabled state preserved");
        assert!(!cfg.appearance.provider_sounds.contains_key("gemini"),
                "stale gemini sound key dropped");
        // The migration gap this fix closes: antigravity gets an audible
        // default without the user opening Settings first.
        assert_eq!(cfg.appearance.provider_sounds.get("antigravity").map(String::as_str),
                   Some("antigravity.mp3"), "antigravity completion sound seeded");
        assert_eq!(cfg.appearance.provider_waiting_sounds.get("antigravity").map(String::as_str),
                   Some("antigravity-waiting.mp3"), "antigravity waiting sound seeded");
        assert_eq!(cfg.appearance.provider_sounds.get("claude").map(String::as_str),
                   Some("__none__"), "explicit user sound choice must not be overwritten");
    }
}
