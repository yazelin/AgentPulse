use crate::config::TelegramConfig;

/// Blocking send — call from a spawned thread or spawn_blocking.
pub fn send(bot_token: &str, chat_id: &str, text: &str) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
    let resp = ureq::post(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send_json(serde_json::json!({ "chat_id": chat_id, "text": text }));
    match resp {
        Ok(_) => Ok(()),
        // Telegram returns the human-readable reason in the JSON body
        // (`description`) — pass it through so the UI test button shows it.
        Err(ureq::Error::Status(code, r)) => {
            let body = r.into_string().unwrap_or_default();
            Err(format!("HTTP {code}: {body}"))
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Fire-and-forget notification for session transitions. No retry: a missed
/// notification is not worth a queue — the capsule still shows the state.
pub fn notify(cfg: &TelegramConfig, provider: &str, event_label: &str, cwd: Option<&str>) {
    if cfg.bot_token.is_empty() || cfg.chat_id.is_empty() {
        return;
    }
    let token = cfg.bot_token.clone();
    let chat_id = cfg.chat_id.clone();
    let text = match cwd {
        Some(dir) => format!("[AgentPulse] {provider} {event_label} — {dir}"),
        None => format!("[AgentPulse] {provider} {event_label}"),
    };
    std::thread::spawn(move || {
        if let Err(e) = send(&token, &chat_id, &text) {
            log::warn!("telegram notify failed: {e}");
        }
    });
}
