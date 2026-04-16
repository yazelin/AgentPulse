// Mock Tauri IPC + event system for the landing-page demo.
//
// The production src/main.js talks to Rust via window.__TAURI_INTERNALS__.
// This shim creates an in-memory stand-in that:
//   - routes invoke() calls to JS handlers
//   - simulates an event bus (task-completed, task-waiting, cursor-left, ...)
//   - runs a lightweight state machine in the background so the demo shows
//     sessions moving through Working → Idle → Waiting without any backend
//
// Nothing here ever reaches out to a real server.

(function () {
  const PROVIDERS = ["claude", "gemini", "codex", "copilot"];

  // ── in-memory state ──────────────────────────────────────────────────
  const now = () => new Date().toISOString();

  const state = {
    port: 19280,
    config: {
      setup_done: true,
      appearance: {
        accent_color: "purple",
        text_size: "medium",
        theme: "dark",
        pin_expanded: false,
        sound_enabled: false,
        provider_sounds: {
          claude: "claude.mp3",
          gemini: "gemini.mp3",
          codex: "codex.mp3",
          copilot: "copilot.mp3",
        },
        provider_waiting_sounds: {
          claude: "claude-waiting.mp3",
          gemini: "gemini-waiting.mp3",
          codex: "codex-waiting.mp3",
          copilot: "copilot-waiting.mp3",
        },
        sound_name: "",
      },
      providers: {
        claude:  { enabled: true,  name: "Claude Code",   settings_path: "~/.claude/settings.json" },
        gemini:  { enabled: true,  name: "Gemini CLI",    settings_path: "~/.gemini/settings.json" },
        codex:   { enabled: true,  name: "Codex CLI",     settings_path: "~/.codex/hooks.json" },
        copilot: { enabled: false, name: "GitHub Copilot", settings_path: "~/.copilot/config.json" },
      },
    },
    sessions: {
      "claude-demo": {
        id: "claude-demo", provider: "claude", state: "working",
        start_time: now(), last_event_time: now(),
        cwd: "~/SDD/AgentPulse", last_tool_name: "Edit", last_prompt: "add landing page",
      },
      "gemini-demo": {
        id: "gemini-demo", provider: "gemini", state: "idle",
        start_time: now(), last_event_time: now(),
        cwd: "~/projects/ai-experiments", last_tool_name: null, last_prompt: null,
      },
      "codex-demo": {
        id: "codex-demo", provider: "codex", state: "waiting_for_user",
        start_time: now(), last_event_time: now(),
        cwd: "~/work/backend", last_tool_name: "shell", last_prompt: "refactor auth middleware",
      },
    },
    activeId: "claude-demo",
    sessionStart: Date.now(),
  };

  // ── event bus ────────────────────────────────────────────────────────
  const listeners = new Map();
  function emit(event, payload) {
    const fns = listeners.get(event) || [];
    fns.forEach((fn) => {
      try { fn({ payload }); } catch (e) { console.warn(e); }
    });
  }

  // ── session snapshot helpers ─────────────────────────────────────────
  function sessionInfo(s) {
    const totalSec = Math.floor((Date.now() - new Date(s.start_time).getTime()) / 1000);
    const h = Math.floor(totalSec / 3600);
    const m = Math.floor((totalSec % 3600) / 60);
    const sec = totalSec % 60;
    const formatted = h > 0
      ? `${h}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`
      : `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
    const projectName = s.cwd ? s.cwd.split("/").pop() : s.id.slice(0, 8);
    const isActive = s.state === "working" || s.state === "waiting_for_user";
    return {
      id: s.id,
      provider: s.provider,
      state: s.state,
      project_name: projectName,
      cwd: s.cwd,
      is_active: isActive,
      formatted_time: formatted,
      last_tool_name: s.last_tool_name,
      last_prompt: s.last_prompt,
    };
  }

  function buildAppState() {
    const all = Object.values(state.sessions);
    const sessions = all.map(sessionInfo);
    // sort: active first, then by last event
    sessions.sort((a, b) => {
      if (a.is_active !== b.is_active) return a.is_active ? -1 : 1;
      return 0;
    });

    const active = state.sessions[state.activeId];
    const activeInfo = active ? sessionInfo(active) :
      (sessions.find((s) => s.is_active) || sessions[0] || null);

    const activeProviders = [...new Set(all.filter(s => s.state === "working" || s.state === "waiting_for_user").map(s => s.provider))].sort();

    return {
      active_session: activeInfo,
      sessions,
      session_count: all.length,
      active_count: all.filter((s) => s.state === "working" || s.state === "waiting_for_user").length,
      active_providers: activeProviders,
    };
  }

  // ── invoke handlers ──────────────────────────────────────────────────
  const handlers = {
    get_config: () => JSON.parse(JSON.stringify(state.config)),
    save_app_config: ({ newConfig }) => { state.config = newConfig; return null; },
    get_server_port: () => state.port,
    get_state: () => buildAppState(),

    // window/tray ops — no-ops in demo
    resize_window: () => null,
    bounce_window: () => null,
    "plugin:window|start_dragging": () => null,

    // sound system
    list_sounds: () => [
      "claude.mp3", "gemini.mp3", "codex.mp3", "copilot.mp3",
      "claude-waiting.mp3", "gemini-waiting.mp3", "codex-waiting.mp3", "copilot-waiting.mp3",
    ],
    play_sound_file: ({ name }) => {
      try {
        const url = `https://raw.githubusercontent.com/yazelin/AgentPulse/main/sounds/${name}`;
        const audio = new Audio(url);
        audio.volume = 0.6;
        audio.play().catch(() => {});
      } catch (e) {}
      return null;
    },
    open_sounds_folder: () => { alert("Opening the sounds folder isn't available in the web demo.\nTry downloading AgentPulse to explore this feature."); return null; },

    // providers — main.js invokes "detect_installed_providers"
    detect_installed_providers: () => ({ claude: true, gemini: true, codex: true, copilot: true }),
    check_provider_setup: () => false,
    install_provider_hooks: ({ providerId }) => {
      if (state.config.providers[providerId]) state.config.providers[providerId].enabled = true;
      return null;
    },
    remove_provider_hooks: ({ providerId }) => {
      if (state.config.providers[providerId]) state.config.providers[providerId].enabled = false;
      return null;
    },
    open_provider_settings: () => { alert("Opening the provider's settings file isn't available in the web demo."); return null; },
    open_url: ({ url }) => { if (url) window.open(url, "_blank", "noopener"); return null; },

    // sessions — main.js sends { id }; accept both shapes defensively
    select_session: (args) => {
      const id = args.id || args.sessionId;
      if (id && state.sessions[id]) state.activeId = id;
      return null;
    },
    remove_session: (args) => {
      const id = args.id || args.sessionId;
      delete state.sessions[id];
      if (state.activeId === id) {
        const remaining = Object.keys(state.sessions);
        state.activeId = remaining[0] || null;
      }
      emit("session-update");
      return null;
    },
    check_staleness: () => null,

    // event plugin
    "plugin:event|listen": ({ event, handler }) => {
      if (!listeners.has(event)) listeners.set(event, []);
      listeners.get(event).push(handler);
      return Promise.resolve(Date.now());
    },
    "plugin:event|unlisten": () => null,
  };

  // ── public shim ──────────────────────────────────────────────────────
  window.__TAURI_INTERNALS__ = {
    invoke: (cmd, args = {}) => {
      const fn = handlers[cmd];
      if (!fn) {
        // Unknown command — return null so main.js doesn't crash.
        return Promise.resolve(null);
      }
      try {
        return Promise.resolve(fn(args));
      } catch (e) {
        return Promise.reject(e);
      }
    },
    // main.js wraps each listener fn via transformCallback and passes the
    // result as a "handler" field. In real Tauri this becomes a numeric id
    // routed through Rust. Here we just keep the function itself.
    transformCallback: (fn) => fn,
  };

  // ── demo state machine ───────────────────────────────────────────────
  // Cycles sessions through states every few seconds so the UI has life.
  const cycle = [
    // (dt in ms, session id, new state)
    [0,     "claude-demo",  "working"],
    [8000,  "claude-demo",  "idle"],
    [10000, "gemini-demo",  "working"],
    [12000, "codex-demo",   "working"],
    [14000, "claude-demo",  "working"],
    [18000, "codex-demo",   "waiting_for_user"],  // triggers task-waiting sound
    [23000, "gemini-demo",  "idle"],
    [28000, "claude-demo",  "idle"],                 // triggers task-completed sound
    [30000, "codex-demo",   "working"],
    [34000, "copilot-demo", "working"],              // new session appears
  ];

  function ensureSession(id, provider) {
    if (!state.sessions[id]) {
      state.sessions[id] = {
        id, provider, state: "idle",
        start_time: now(), last_event_time: now(),
        cwd: `~/projects/${provider}-demo`,
        last_tool_name: null, last_prompt: null,
      };
    }
  }

  function applyTransition(sessionId, newState) {
    const s = state.sessions[sessionId];
    if (!s) {
      const provider = sessionId.replace("-demo", "");
      ensureSession(sessionId, provider);
      return applyTransition(sessionId, newState);
    }
    const prev = s.state;
    s.state = newState;
    s.last_event_time = now();
    if (prev === "working" && newState === "idle") {
      emit("task-completed", s.provider);
    }
    if (prev !== "waiting_for_user" && newState === "waiting_for_user") {
      emit("task-waiting", s.provider);
    }
    emit("session-update");
  }

  function startCycle() {
    let idx = 0;
    const run = () => {
      const entry = cycle[idx];
      if (!entry) {
        // loop back so the demo keeps going forever
        idx = 0;
        state.sessionStart = Date.now();
        setTimeout(run, 4000);
        return;
      }
      const [, id, newState] = entry;
      applyTransition(id, newState);
      idx++;
      const next = cycle[idx];
      const delay = next ? (next[0] - entry[0]) : 4000;
      setTimeout(run, Math.max(delay, 1000));
    };
    // kick off after a short delay so the UI has time to render first
    setTimeout(run, 1500);
  }

  // expose a tiny helper for the landing page to know the demo is ready
  window.__AP_DEMO__ = { state, emit, startCycle };

  // Real Tauri emits "cursor-left" from a Rust polling thread. In the demo
  // we approximate by watching pointer leave/enter on the app container.
  function wireCursorLeave() {
    const app = document.getElementById("app");
    if (!app) { setTimeout(wireCursorLeave, 50); return; }
    app.addEventListener("mouseleave", () => emit("cursor-left"));
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", wireCursorLeave);
  } else {
    wireCursorLeave();
  }

  // auto-start the state cycling
  startCycle();
})();
