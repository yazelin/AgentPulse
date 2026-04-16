use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Raw event as received from any CLI — fields vary by provider
#[derive(Debug, Clone, Deserialize)]
pub struct RawHookEvent {
    #[serde(flatten)]
    pub fields: std::collections::HashMap<String, Value>,
}

/// Normalized event used internally
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookEvent {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub hook_event_name: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub notification_type: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
}

impl RawHookEvent {
    /// Normalize raw fields from any CLI into a standard HookEvent
    pub fn normalize(self, provider: &str) -> HookEvent {
        let f = &self.fields;

        // session_id: try multiple field names
        let session_id = get_str(f, "session_id")
            .or_else(|| get_str(f, "sessionId"))
            .or_else(|| get_str(f, "session"))
            .unwrap_or_default();

        // hook_event_name: try multiple field names
        let hook_event_name = get_str(f, "hook_event_name")
            .or_else(|| get_str(f, "hookEventName"))
            .or_else(|| get_str(f, "event"))
            .or_else(|| get_str(f, "type"))
            .unwrap_or_default();

        // cwd: try multiple field names
        let cwd = get_str(f, "cwd")
            .or_else(|| get_str(f, "workingDirectory"))
            .or_else(|| get_str(f, "projectDir"));

        // prompt: try multiple field names
        let prompt = get_str(f, "prompt")
            .or_else(|| get_str(f, "initialPrompt"))
            .or_else(|| get_str(f, "input"))
            .or_else(|| get_str(f, "message"))
            .or_else(|| get_str(f, "userPrompt"));

        // tool_name
        let tool_name = get_str(f, "tool_name")
            .or_else(|| get_str(f, "toolName"));

        let notification_type = get_str(f, "notification_type")
            .or_else(|| get_str(f, "notificationType"));

        HookEvent {
            provider: provider.to_string(),
            session_id,
            hook_event_name,
            cwd,
            tool_name,
            notification_type,
            prompt,
        }
    }
}

fn get_str(map: &std::collections::HashMap<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}
