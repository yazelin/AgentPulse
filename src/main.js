function invoke(cmd, args = {}) {
  return window.__TAURI_INTERNALS__.invoke(cmd, args);
}

// ─── Provider Icons (inline SVG) ───
const PROVIDER_ICONS = {
  claude: `<svg viewBox="0 0 1200 1200" fill="currentColor"><path d="M233.96 800.21L468.64 668.54l3.95-11.44-3.95-6.36-11.44 0-39.22-2.42-134.09-3.62-116.3-4.83-112.67-6.04-28.35-6.04L0 592.75l2.74-17.48 23.84-16.03 34.15 2.98 75.46 5.15 113.23 7.81 82.15 4.83 121.69 12.65 19.33 0 2.74-7.81-6.6-4.83-5.15-4.83L346.39 495.79 219.54 411.87l-66.44-48.32-35.92-24.48-18.12-22.95-7.81-50.09 32.62-35.92 43.81 2.98 11.19 2.98 44.38 34.15 94.79 73.37 123.79 91.17 18.12 15.06 7.25-5.15.89-3.63-8.13-13.61-67.46-121.69-71.84-123.79-31.97-51.3-8.46-30.76c-2.98-12.64-5.15-23.27-5.15-36.24l37.13-50.42 20.54-6.6 49.53 6.6 20.86 18.12 30.76 70.39 49.85 110.82 77.32 150.68 22.63 44.7 12.08 41.4 4.51 12.64h7.81v-7.25l6.36-84.89 11.76-104.21 11.44-134.09 3.94-37.77 18.68-45.26 37.13-24.48 28.99 13.85 23.84 34.15-3.3 22.07-14.17 92.13-27.79 144.32-18.12 96.64 10.55 0 12.08-12.08 48.89-64.91 82.15-102.68 36.24-40.76 42.28-45.02 27.14-21.42 51.3 0 37.77 56.13-16.91 57.99-52.83 67.01-43.81 56.78-62.82 84.56-39.22 67.65 3.63 5.4 9.34-.89 141.91-30.2 76.67-13.85 91.49-15.7 41.4 19.33 4.51 19.65-16.27 40.19-97.85 24.16-114.77 22.95-170.9 40.43-2.09 1.53 2.42 2.98 76.99 7.25 32.94 1.77 80.62 0 150.12 11.19 39.22 25.93 23.52 31.73-3.95 24.16-60.4 30.76-81.5-19.33-190.23-45.26-65.46-16.27-9.02 0v5.4l54.36 53.15 99.62 89.96 124.75 115.97 6.36 28.67-16.03 22.63-16.91-2.42-109.61-82.47-42.28-37.13-95.76-80.62-6.36 0v8.46l22.07 32.29 116.54 175.17 6.04 53.72-8.46 17.48-30.2 10.55-33.18-6.04-68.21-95.76-70.39-107.84-56.78-96.64-6.93 3.95-33.5 360.89-15.7 18.44-36.24 13.85-30.2-22.95-16.03-37.13 16.03-73.37 19.33-95.76 15.7-76.11 14.17-94.55 8.46-31.41-.56-2.09-6.93.89-71.28 97.85-108.4 146.5-85.77 91.81-20.54 8.13-35.6-18.44 3.3-32.94 19.89-29.31 118.74-151.01 71.6-93.58 46.23-53.72-.32-7.81-2.74 0L205.29 929.4l-56.13 7.25-24.16-22.63 2.98-37.13 11.44-12.08 94.79-65.23z"/></svg>`,
  gemini: `<svg viewBox="0 0 65 65" fill="currentColor"><path d="M32.447 0c.68 0 1.273.465 1.439 1.125a38.904 38.904 0 001.999 5.905c2.152 5 5.105 9.376 8.854 13.125 3.751 3.75 8.126 6.703 13.125 8.855a38.98 38.98 0 005.906 1.999c.66.166 1.124.758 1.124 1.438 0 .68-.464 1.273-1.125 1.439a38.902 38.902 0 00-5.905 1.999c-5 2.152-9.375 5.105-13.125 8.854-3.749 3.751-6.702 8.126-8.854 13.125a38.973 38.973 0 00-2 5.906 1.485 1.485 0 01-1.438 1.124c-.68 0-1.272-.464-1.438-1.125a38.913 38.913 0 00-2-5.905c-2.151-5-5.103-9.375-8.854-13.125-3.75-3.749-8.125-6.702-13.125-8.854a38.973 38.973 0 00-5.905-2A1.485 1.485 0 010 32.448c0-.68.465-1.272 1.125-1.438a38.903 38.903 0 005.905-2c5-2.151 9.376-5.104 13.125-8.854 3.75-3.749 6.703-8.125 8.855-13.125a38.972 38.972 0 001.999-5.905A1.485 1.485 0 0132.447 0z"/></svg>`,
  copilot: `<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C8 2 4.5 4.5 4 8v3c0 1 .5 2 1.5 2.5V15c0 1.5 1 2.5 2.5 2.5h8c1.5 0 2.5-1 2.5-2.5v-1.5c1-.5 1.5-1.5 1.5-2.5V8c-.5-3.5-4-6-8-6zM8.5 11a1.5 1.5 0 110-3 1.5 1.5 0 010 3zm7 0a1.5 1.5 0 110-3 1.5 1.5 0 010 3zM9 15.5a1 1 0 112 0 1 1 0 01-2 0zm4 0a1 1 0 112 0 1 1 0 01-2 0z"/><path d="M6 9c0-1.5.5-3 2-4s3-1.5 4-1.5 2.5.5 4 1.5 2 2.5 2 4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>`,
  codex: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M5 16l5-4-5-4"/><line x1="13" y1="18" x2="19" y2="18"/></svg>`,
};

const PROVIDER_COLORS = {
  claude: "#d97757",
  gemini: "#4285f4",
  copilot: "#6e40c9",
  codex: "#10a37f",
};

// ─── State ───
const COLORS = {
  purple: "rgb(217,128,255)", cyan: "rgb(77,217,255)",
  green: "rgb(77,242,153)", orange: "rgb(255,153,51)", pink: "rgb(255,102,153)",
};
const SCALES = { small: 0.85, medium: 1, large: 1.15 };
const W = 300;

let currentView = "capsule";
let serverPort = 0;
let appConfig = null;
let collapsedAt = 0;

const $ = (id) => document.getElementById(id);

// ─── Window resize ───
async function fitWindow() {
  await new Promise(r => requestAnimationFrame(r));
  const h = Math.max(Math.ceil(document.getElementById("app").scrollHeight) + 2, 46);
  await invoke("resize_window", { width: W, height: h });
}

// ─── View switching ───
function showView(view) {
  const wasExpanded = currentView !== "capsule";
  currentView = view;
  $("view-expanded").classList.toggle("hidden", view !== "expanded");
  $("view-settings").classList.toggle("hidden", view !== "settings");
  $("capsule").classList.toggle("has-panel-below", view === "expanded" || view === "settings");
  fitWindow();
  if (view === "capsule" && wasExpanded) {
    collapsedAt = Date.now();
    setTimeout(() => invoke("bounce_window").catch(() => {}), 80);
  }
}

// ─── Provider icon HTML ───
function providerIconHtml(providerId, size = 14) {
  const svg = PROVIDER_ICONS[providerId] || PROVIDER_ICONS.claude;
  const color = PROVIDER_COLORS[providerId] || "#888";
  return `<span class="provider-icon" style="width:${size}px;height:${size}px;color:${color}">${svg}</span>`;
}

// ─── Init ───
async function init() {
  if (!window.__TAURI_INTERNALS__) { setTimeout(init, 200); return; }

  try {
    serverPort = await invoke("get_server_port");
    appConfig = await invoke("get_config");
  } catch (e) { return; }

  applyAccentColor(appConfig.appearance.accent_color);
  applyTextSize(appConfig.appearance.text_size);
  $("toggle-sound").checked = appConfig.appearance.sound_enabled;
  $("toggle-pin").checked = appConfig.appearance.pin_expanded;
  if (appConfig.appearance.sound_enabled) $("sound-picker").classList.remove("hidden");

  // First launch → open settings automatically
  const firstLaunch = !appConfig.setup_done;
  if (firstLaunch) {
    await renderProviders();
    showView("settings");
  } else {
    await fitWindow();
    if (appConfig.appearance.pin_expanded) {
      $("btn-pin").classList.add("active");
      showView("expanded");
    }
  }

  // Drag
  $("capsule").addEventListener("mousedown", (e) => {
    if (e.buttons === 1) invoke("plugin:window|start_dragging", { label: "main" }).catch(() => {});
  });

  // Hover expand
  $("capsule").addEventListener("mouseenter", () => {
    if (currentView === "capsule" && !appConfig.appearance.pin_expanded && (Date.now() - collapsedAt > 500)) {
      showView("expanded");
    }
  });

  // Collapse via cursor-left event + :hover fallback
  const collapseCallbackId = window.__TAURI_INTERNALS__.transformCallback(() => {
    if (currentView === "expanded" && !appConfig.appearance.pin_expanded) showView("capsule");
  });
  invoke("plugin:event|listen", { event: "cursor-left", target: { kind: "Any" }, handler: collapseCallbackId }).catch(() => {});

  setInterval(() => {
    if (currentView !== "expanded" || appConfig.appearance.pin_expanded) return;
    if (!document.getElementById("app").matches(":hover")) showView("capsule");
  }, 200);

  // Re-register after delay
  setTimeout(() => {
    const cb2 = window.__TAURI_INTERNALS__.transformCallback(() => {
      if (currentView === "expanded" && !appConfig.appearance.pin_expanded) showView("capsule");
    });
    invoke("plugin:event|listen", { event: "cursor-left", target: { kind: "Any" }, handler: cb2 }).catch(() => {});
  }, 2000);

  // Pin
  $("btn-pin").addEventListener("click", () => {
    appConfig.appearance.pin_expanded = !appConfig.appearance.pin_expanded;
    $("toggle-pin").checked = appConfig.appearance.pin_expanded;
    $("btn-pin").classList.toggle("active", appConfig.appearance.pin_expanded);
    if (!appConfig.appearance.pin_expanded && currentView === "expanded") showView("capsule");
    saveConfig();
  });

  // Settings
  $("btn-settings").addEventListener("click", () => { renderProviders(); showView("settings"); });
  $("btn-close-settings").addEventListener("click", () => {
    appConfig.setup_done = true; saveConfig();
    showView(appConfig.appearance.pin_expanded ? "expanded" : "capsule");
  });

  $("toggle-pin").addEventListener("change", (e) => {
    appConfig.appearance.pin_expanded = e.target.checked;
    $("btn-pin").classList.toggle("active", appConfig.appearance.pin_expanded);
    if (!appConfig.appearance.pin_expanded) showView("capsule"); else showView("expanded");
    saveConfig();
  });

  $("toggle-sound").addEventListener("change", (e) => {
    appConfig.appearance.sound_enabled = e.target.checked;
    $("sound-picker").classList.toggle("hidden", !e.target.checked);
    fitWindow();
    saveConfig();
  });

  initCustomDropdown();

  document.querySelectorAll(".color-dot").forEach(d => d.addEventListener("click", () => {
    appConfig.appearance.accent_color = d.dataset.color;
    applyAccentColor(d.dataset.color);
    saveConfig();
  }));

  document.querySelectorAll(".size-btn").forEach(b => b.addEventListener("click", () => {
    appConfig.appearance.text_size = b.dataset.size;
    applyTextSize(b.dataset.size);
    fitWindow();
    saveConfig();
  }));

  $("btn-quit").addEventListener("click", () => { try { window.close(); } catch (e) {} });

  refreshState();
  setInterval(refreshState, 1000);
}

// ─── Providers in settings ───
const PROVIDER_ORDER = ["claude", "gemini", "codex", "copilot"];

async function renderProviders() {
  const detected = await invoke("detect_installed_providers");
  const list = $("provider-list");

  // Fixed order instead of HashMap random order
  const entries = PROVIDER_ORDER
    .filter(id => appConfig.providers[id])
    .map(id => [id, appConfig.providers[id]]);

  list.innerHTML = entries.map(([id, p]) => {
    const found = detected[id] || false;
    const canEnable = !!p.settings_path;
    const checked = p.enabled ? "checked" : "";
    const statusText = !canEnable ? "coming soon"
                     : found ? "detected"
                     : "";
    const statusClass = !canEnable ? "provider-pending"
                      : found ? "provider-found"
                      : "";

    return `<label class="provider-item ${!canEnable ? 'disabled' : ''}">
      <input type="checkbox" class="provider-check" data-provider="${id}" ${checked} ${!canEnable ? 'disabled' : ''}>
      ${providerIconHtml(id, 18)}
      <span class="provider-name">${esc(p.name)}</span>
      ${statusText ? `<span class="${statusClass}">${statusText}</span>` : ""}
    </label>`;
  }).join("");

  // Listen for toggle changes
  list.querySelectorAll(".provider-check").forEach(cb => {
    cb.addEventListener("change", async () => {
      const pid = cb.dataset.provider;
      if (cb.checked) {
        // Enable and install hooks
        try {
          await invoke("install_provider_hooks", { providerId: pid });
        } catch (e) {}
      } else {
        // Just disable in config
        appConfig.providers[pid].enabled = false;
        await saveConfig();
      }
      appConfig = await invoke("get_config");
      appConfig.setup_done = true; saveConfig();
    });
  });
}

// ─── Dropdown ───
function initCustomDropdown() {
  const wrapper = $("sound-dropdown");
  const selected = wrapper.querySelector(".dropdown-selected");
  const options = wrapper.querySelector(".dropdown-options");

  selected.addEventListener("click", (e) => { e.stopPropagation(); options.classList.toggle("hidden"); });
  wrapper.querySelectorAll(".dropdown-option").forEach(opt => {
    opt.addEventListener("click", (e) => {
      e.stopPropagation();
      selected.textContent = opt.textContent;
      appConfig.appearance.sound_name = opt.dataset.value;
      playSound(opt.dataset.value);
      options.classList.add("hidden");
      wrapper.querySelectorAll(".dropdown-option").forEach(o => o.classList.toggle("active", o.dataset.value === opt.dataset.value));
      saveConfig();
    });
  });
  document.addEventListener("click", () => options.classList.add("hidden"));
}

// ─── Config save ───
async function saveConfig() {
  try { await invoke("save_app_config", { newConfig: appConfig }); } catch (e) {}
}

// ─── State ───
let lastJson = "";
async function refreshState() {
  try {
    const st = await invoke("get_state");
    const j = JSON.stringify(st);
    if (j === lastJson) return;
    lastJson = j;
    renderCapsule(st);
    renderSessions(st);
    if (currentView === "expanded") fitWindow();
  } catch (e) {}
}

function renderCapsule(st) {
  const s = st.active_session;

  // Capsule icons: show active provider icons
  const providers = st.active_providers.length > 0 ? st.active_providers : (s ? [s.provider] : ["claude"]);
  $("capsule-icons").innerHTML = providers.map(p => providerIconHtml(p, 16)).join('<span class="icon-sep">|</span>');

  if (s) {
    $("capsule-project").textContent = s.project_name;
    const stMap = { working: "Working...", waiting_for_user: "Waiting", stale: "Stale" };
    $("capsule-status").textContent = stMap[s.state] || "Idle";
    $("capsule-time").textContent = s.is_active ? s.formatted_time : "";
    $("capsule-time").style.display = s.is_active ? "" : "none";
  } else {
    $("capsule-project").textContent = "AgentPulse";
    $("capsule-status").textContent = "";
    $("capsule-time").style.display = "none";
  }

  if (st.session_count > 1) {
    $("capsule-count").classList.remove("hidden");
    let h = "";
    if (st.active_count > 0) {
      h += `<span class="count-active">${st.active_count}</span>`;
      if (st.active_count < st.session_count) h += `<span class="count-sep">/</span>`;
    }
    h += `<span class="count-total">${st.session_count}</span>`;
    $("capsule-count").innerHTML = h;
  } else $("capsule-count").classList.add("hidden");
}

function renderSessions(st) {
  const aid = st.active_session?.id;
  $("session-list").innerHTML = st.sessions.map(s => {
    const sel = s.id === aid ? " selected" : "";
    const sc = ({ working: "working", waiting_for_user: "waiting_for_user", stale: "stale" })[s.state] || "idle";
    const sl = ({ working: "working", waiting_for_user: "waiting", stale: "stale" })[s.state] || "";
    const cwdShort = s.cwd ? s.cwd.replace(/^\/home\/[^/]+/, "~") : "";
    return `<div class="session-row${sel}" data-id="${s.id}">
      <div class="session-provider-icon">${providerIconHtml(s.provider, 14)}</div>
      <div class="session-info">
        <div class="session-header">
          <span class="session-name">${esc(s.project_name)}</span>
          ${sl ? `<span class="session-state-label ${sc}">${sl}</span>` : ""}
        </div>
        ${cwdShort ? `<div class="session-cwd">${esc(cwdShort)}</div>` : ""}
        ${s.last_prompt ? `<div class="session-prompt">${esc(s.last_prompt)}</div>` : ""}
      </div>
      ${s.is_active ? `<span class="session-time">${s.formatted_time}</span>` : ""}
    </div>`;
  }).join("");

  $("session-list").querySelectorAll(".session-row").forEach(r => {
    r.addEventListener("click", () => {
      const sid = r.dataset.id;
      invoke("select_session", { id: sid });
      const session = st.sessions.find(s => s.id === sid);
      if (session) invoke("focus_session_window", { projectName: session.project_name, cwd: session.cwd || null }).catch(() => {});
      refreshState();
    });
  });
}

// ─── Apply ───
function applyAccentColor(n) {
  document.documentElement.style.setProperty("--accent", COLORS[n] || COLORS.purple);
  document.querySelectorAll(".color-dot").forEach(d => d.classList.toggle("active", d.dataset.color === n));
}
function applyTextSize(s) {
  document.documentElement.style.setProperty("--scale", SCALES[s] || 1);
  document.querySelectorAll(".size-btn").forEach(b => b.classList.toggle("active", b.dataset.size === s));
}

// ─── Sounds ───
function playSound(name) {
  try {
    const ctx = new AudioContext(), osc = ctx.createOscillator(), g = ctx.createGain();
    osc.connect(g); g.connect(ctx.destination); g.gain.setValueAtTime(0.2, ctx.currentTime); const t = ctx.currentTime;
    switch (name) {
      case "ping": osc.frequency.setValueAtTime(1200, t); g.gain.exponentialRampToValueAtTime(0.01, t + 0.15); osc.stop(t + 0.15); break;
      case "pop": osc.frequency.setValueAtTime(600, t); osc.frequency.exponentialRampToValueAtTime(200, t + 0.08); g.gain.exponentialRampToValueAtTime(0.01, t + 0.1); osc.stop(t + 0.1); break;
      case "chime": osc.type = "sine"; osc.frequency.setValueAtTime(523, t); osc.frequency.setValueAtTime(659, t + 0.15); osc.frequency.setValueAtTime(784, t + 0.3); g.gain.exponentialRampToValueAtTime(0.01, t + 0.5); osc.stop(t + 0.5); break;
      case "bell": osc.type = "sine"; osc.frequency.setValueAtTime(880, t); g.gain.exponentialRampToValueAtTime(0.01, t + 0.8); osc.stop(t + 0.8); break;
      default: osc.type = "sine"; osc.frequency.setValueAtTime(880, t); osc.frequency.setValueAtTime(1100, t + 0.1); g.gain.exponentialRampToValueAtTime(0.01, t + 0.3); osc.stop(t + 0.3);
    }
    osc.start(t);
  } catch (e) {}
}

function esc(s) { const d = document.createElement("div"); d.textContent = s; return d.innerHTML; }

if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
else init();
