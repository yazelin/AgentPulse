use crate::hook_event::HookEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Working,
    WaitingForUser,
    Stale,
}

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: String,
    pub provider: String,
    pub state: SessionState,
    pub start_time: DateTime<Utc>,
    pub last_event_time: DateTime<Utc>,
    pub cwd: Option<String>,
    pub last_tool_name: Option<String>,
    pub last_prompt: Option<String>,
}

impl Session {
    pub fn new(id: String, provider: String, cwd: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            provider,
            state: SessionState::Idle,
            start_time: now,
            last_event_time: now,
            cwd,
            last_tool_name: None,
            last_prompt: None,
        }
    }

    pub fn handle_event(&mut self, event: &HookEvent) {
        self.last_event_time = Utc::now();

        match event.hook_event_name.as_str() {
            "SessionStart" => {
                self.state = SessionState::Idle;
                if let Some(ref cwd) = event.cwd {
                    self.cwd = Some(cwd.clone());
                }
            }
            "UserPromptSubmit" => {
                self.state = SessionState::Working;
                if let Some(ref prompt) = event.prompt {
                    self.last_prompt = Some(prompt.clone());
                }
            }
            "PreToolUse" | "PostToolUse" | "PostToolUseFailure" => {
                self.state = SessionState::Working;
                if let Some(ref tool_name) = event.tool_name {
                    self.last_tool_name = Some(tool_name.clone());
                }
            }
            "PermissionRequest" => {
                self.state = SessionState::WaitingForUser;
            }
            "Stop" => {
                self.state = SessionState::Idle;
            }
            _ => {}
        }
    }

    pub fn project_name(&self) -> String {
        if let Some(ref cwd) = self.cwd {
            if let Some(name) = std::path::Path::new(cwd).file_name() {
                return name.to_string_lossy().to_string();
            }
        }
        self.id.chars().take(8).collect()
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Working | SessionState::WaitingForUser)
    }

    pub fn elapsed_seconds(&self) -> i64 {
        Utc::now().signed_duration_since(self.start_time).num_seconds()
    }

    pub fn formatted_time(&self) -> String {
        let total = self.elapsed_seconds();
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        let seconds = total % 60;
        if hours > 0 {
            format!("{hours}:{minutes:02}:{seconds:02}")
        } else {
            format!("{minutes:02}:{seconds:02}")
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub provider: String,
    pub state: SessionState,
    pub project_name: String,
    pub cwd: Option<String>,
    pub is_active: bool,
    pub formatted_time: String,
    pub last_tool_name: Option<String>,
    pub last_prompt: Option<String>,
}

impl From<&Session> for SessionInfo {
    fn from(s: &Session) -> Self {
        Self {
            id: s.id.clone(),
            provider: s.provider.clone(),
            state: s.state,
            project_name: s.project_name(),
            cwd: s.cwd.clone(),
            is_active: s.is_active(),
            formatted_time: s.formatted_time(),
            last_tool_name: s.last_tool_name.clone(),
            last_prompt: s.last_prompt.clone(),
        }
    }
}

pub struct SessionManager {
    pub sessions: HashMap<String, Session>,
    pub active_session_id: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session_id: None,
        }
    }

    pub fn handle_event(&mut self, event: &HookEvent) -> bool {
        if event.hook_event_name == "SessionEnd" {
            self.sessions.remove(&event.session_id);
            if self.active_session_id.as_deref() == Some(&event.session_id) {
                self.active_session_id = self.sessions.keys().next().cloned();
            }
            return true;
        }

        let provider = event.provider.clone();
        let session = self
            .sessions
            .entry(event.session_id.clone())
            .or_insert_with(|| {
                if self.active_session_id.is_none() {
                    self.active_session_id = Some(event.session_id.clone());
                }
                Session::new(event.session_id.clone(), provider, event.cwd.clone())
            });

        let was_working = session.state == SessionState::Working;
        session.handle_event(event);
        let now_idle = session.state == SessionState::Idle;

        was_working && now_idle
    }

    pub fn check_staleness(&mut self) {
        let now = Utc::now();
        let mut to_remove = Vec::new();

        for (id, session) in &mut self.sessions {
            let elapsed = now.signed_duration_since(session.last_event_time).num_seconds();
            if elapsed > 1800 {
                to_remove.push(id.clone());
            } else if elapsed > 600 {
                session.state = SessionState::Stale;
            } else if session.is_active() && elapsed > 30 {
                session.state = SessionState::Idle;
            }
        }

        for id in to_remove {
            self.sessions.remove(&id);
            if self.active_session_id.as_deref() == Some(&id) {
                self.active_session_id = self.sessions.keys().next().cloned();
            }
        }
    }

    pub fn select_session(&mut self, id: String) {
        if self.sessions.contains_key(&id) {
            self.active_session_id = Some(id);
        }
    }

    pub fn active_session(&self) -> Option<&Session> {
        if let Some(ref id) = self.active_session_id {
            if let Some(s) = self.sessions.get(id) {
                if s.is_active() {
                    return Some(s);
                }
            }
        }
        if let Some(s) = self.sessions.values().find(|s| s.is_active()) {
            return Some(s);
        }
        if let Some(ref id) = self.active_session_id {
            if let Some(s) = self.sessions.get(id) {
                return Some(s);
            }
        }
        self.sessions.values().next()
    }

    pub fn sorted_sessions(&self) -> Vec<SessionInfo> {
        let mut sessions: Vec<&Session> = self.sessions.values().collect();
        sessions.sort_by(|a, b| {
            b.is_active()
                .cmp(&a.is_active())
                .then(b.last_event_time.cmp(&a.last_event_time))
        });
        sessions.iter().map(|s| SessionInfo::from(*s)).collect()
    }

    pub fn active_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_active()).count()
    }

    /// Get unique active provider IDs
    pub fn active_providers(&self) -> Vec<String> {
        let mut providers: Vec<String> = self.sessions.values()
            .filter(|s| s.is_active())
            .map(|s| s.provider.clone())
            .collect();
        providers.sort();
        providers.dedup();
        providers
    }

    pub fn get_state(&self) -> AppState {
        let active = self.active_session().map(SessionInfo::from);
        let sessions = self.sorted_sessions();
        let session_count = self.sessions.len();
        let active_count = self.active_count();
        let active_providers = self.active_providers();

        AppState {
            active_session: active,
            sessions,
            session_count,
            active_count,
            active_providers,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AppState {
    pub active_session: Option<SessionInfo>,
    pub sessions: Vec<SessionInfo>,
    pub session_count: usize,
    pub active_count: usize,
    pub active_providers: Vec<String>,
}
