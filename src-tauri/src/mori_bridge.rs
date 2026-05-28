//! Mori body-interface 橋接:把 AgentPulse 既有的 session 狀態 / 轉換,用 Mori 的
//! event envelope + manifest 對外播。偵測邏輯不在這(在 session.rs);這裡只做格式轉換
//! 與 ~/.mori/body-parts 的 manifest 寫入。

use crate::session::{AppState, SessionTransition};
use serde::Serialize;

/// Mori event envelope — 對齊 mori-desktop docs/mori-body-interface.md §Events。
#[derive(Debug, Clone, Serialize)]
pub struct MoriEvent {
    pub schema_version: u32,
    pub event_id: String,
    pub source: String,
    pub r#type: String,
    pub time: String,
    pub session_id: String,
    pub severity: String,
    pub summary: String,
    pub payload: serde_json::Value,
}

impl MoriEvent {
    /// 把 AgentPulse 的 transition 轉成 Mori cue 事件。None → 不發。
    pub fn from_transition(
        t: SessionTransition,
        provider: &str,
        session_id: &str,
        now: &str,
    ) -> Option<MoriEvent> {
        let (kind, severity, summary) = match t {
            SessionTransition::StartedWaiting => (
                "cue.waiting_input",
                "attention",
                format!("{provider} is waiting for input."),
            ),
            SessionTransition::Completed => (
                "cue.done",
                "info",
                format!("{provider} session finished."),
            ),
            SessionTransition::None => return None,
        };
        Some(MoriEvent {
            schema_version: 1,
            event_id: format!("evt-{session_id}-{now}"),
            source: "mori.agent-pulse".to_string(),
            r#type: kind.to_string(),
            time: now.to_string(),
            session_id: session_id.to_string(),
            severity: severity.to_string(),
            summary,
            payload: serde_json::json!({ "provider": provider }),
        })
    }
}

/// body part manifest(Mori BI-1 BodyManifest 形狀)。帶啟動時的實際 port。
pub fn manifest_json(port: u16) -> String {
    serde_json::json!({
        "schema_version": 1,
        "id": "mori.agent-pulse",
        "name": "AgentPulse",
        "kind": "local_service",
        "description": "AI coding CLI session pulse — observes Claude / Gemini / Codex / Copilot sessions.",
        "capabilities": ["agent.session.observe"],
        "interfaces": [
            { "name": "control", "transport": "http", "base_url": format!("http://127.0.0.1:{port}") },
            { "name": "events",  "transport": "sse",  "url": format!("http://127.0.0.1:{port}/events") }
        ],
        "permissions": [],
        "data_policy": { "owns_raw_data": true, "default_ingestion": "off" }
    })
    .to_string()
}

/// 啟動時把 manifest 寫到 ~/.mori/body-parts/mori.agent-pulse/manifest.json(覆寫 — 因為 port 會變)。
pub fn write_manifest(port: u16) {
    if let Some(home) = dirs::home_dir() {
        let dir = home.join(".mori").join("body-parts").join("mori.agent-pulse");
        if std::fs::create_dir_all(&dir).is_ok() {
            let _ = std::fs::write(dir.join("manifest.json"), manifest_json(port));
        }
    }
}

/// 一個 SSE data frame。
pub fn sse_frame(json: &str) -> String {
    format!("data: {json}\n\n")
}

/// 把當前 AppState 序列化成 /sessions 的 body(AppState 已是 serde Serialize)。
pub fn sessions_json(state: &AppState) -> String {
    serde_json::to_string(state).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waiting_transition_maps_to_cue_waiting_input() {
        let ev = MoriEvent::from_transition(
            SessionTransition::StartedWaiting, "codex", "sess_1", "2026-05-28T10:00:00+08:00",
        )
        .expect("waiting → Some");
        assert_eq!(ev.r#type, "cue.waiting_input");
        assert_eq!(ev.severity, "attention");
        assert_eq!(ev.session_id, "sess_1");
        assert_eq!(ev.source, "mori.agent-pulse");
        assert!(ev.summary.contains("codex"));
    }

    #[test]
    fn completed_transition_maps_to_cue_done() {
        let ev = MoriEvent::from_transition(
            SessionTransition::Completed, "claude", "s2", "2026-05-28T10:00:00+08:00",
        )
        .expect("completed → Some");
        assert_eq!(ev.r#type, "cue.done");
        assert_eq!(ev.severity, "info");
    }

    #[test]
    fn none_transition_produces_no_event() {
        assert!(MoriEvent::from_transition(
            SessionTransition::None, "x", "s", "t").is_none());
    }

    #[test]
    fn manifest_is_valid_json_with_id_kind_and_live_port() {
        let m = manifest_json(19283);
        let v: serde_json::Value = serde_json::from_str(&m).expect("manifest valid json");
        assert_eq!(v["id"], "mori.agent-pulse");
        assert_eq!(v["kind"], "local_service");
        let ifaces = v["interfaces"].as_array().unwrap();
        let sse = ifaces.iter().find(|i| i["transport"] == "sse").unwrap();
        assert_eq!(sse["url"], "http://127.0.0.1:19283/events");
    }

    #[test]
    fn sse_frame_is_data_line_with_double_newline() {
        let f = sse_frame(r#"{"a":1}"#);
        assert_eq!(f, "data: {\"a\":1}\n\n");
    }
}
