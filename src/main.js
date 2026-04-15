function invoke(cmd, args = {}) {
  return window.__TAURI_INTERNALS__.invoke(cmd, args);
}

// ─── Settings ───
const S = {
  accentColor: localStorage.getItem("accentColor") || "purple",
  soundEnabled: localStorage.getItem("soundEnabled") === "true",
  soundName: localStorage.getItem("soundName") || "glass",
  pinExpanded: localStorage.getItem("pinExpanded") === "true",
  textSize: localStorage.getItem("textSize") || "medium",
  hooksDismissed: localStorage.getItem("hooksDismissed") === "true",
};
function save(k, v) { S[k] = v; localStorage.setItem(k, String(v)); }

const COLORS = {
  purple: "rgb(217,128,255)", cyan: "rgb(77,217,255)",
  green: "rgb(77,242,153)", orange: "rgb(255,153,51)", pink: "rgb(255,102,153)",
};
const SCALES = { small: 0.85, medium: 1, large: 1.15 };
const W = 300;

let currentView = "capsule";
let serverPort = 0;
let collapsedAt = 0; // timestamp of last collapse, for cooldown

const $ = (id) => document.getElementById(id);

// ─── Force repaint after DOM change ───
async function repaint() {
  try {
    const el = document.getElementById("app");
    const h = Math.max(Math.ceil(el.scrollHeight) + 2, 46);
    await invoke("resize_window", { width: W + 1, height: h });
    await invoke("resize_window", { width: W, height: h });
  } catch (e) {}
}

// ─── Window resize — instant, no animation ───
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
  $("capsule").classList.toggle("has-panel", view !== "capsule");
  fitWindow();
  // Bounce when collapsing back to capsule
  if (view === "capsule" && wasExpanded) {
    collapsedAt = Date.now();
    setTimeout(() => invoke("bounce_window").catch(() => {}), 80);
  }
}

// ─── Custom dropdown ───
function initCustomDropdown() {
  const wrapper = $("sound-dropdown");
  const selected = wrapper.querySelector(".dropdown-selected");
  const options = wrapper.querySelector(".dropdown-options");

  selected.addEventListener("click", (e) => {
    e.stopPropagation();
    options.classList.toggle("hidden");
  });

  wrapper.querySelectorAll(".dropdown-option").forEach(opt => {
    opt.addEventListener("click", (e) => {
      e.stopPropagation();
      const val = opt.dataset.value;
      selected.textContent = opt.textContent;
      save("soundName", val);
      playSound(val);
      options.classList.add("hidden");
      // Update active state
      wrapper.querySelectorAll(".dropdown-option").forEach(o => o.classList.toggle("active", o.dataset.value === val));
    });
  });

  // Close on click outside
  document.addEventListener("click", () => options.classList.add("hidden"));
}

// ─── Init ───
async function init() {
  if (!window.__TAURI_INTERNALS__) { setTimeout(init, 200); return; }

  applyAccentColor(S.accentColor);
  applyTextSize(S.textSize);
  $("toggle-sound").checked = S.soundEnabled;
  $("toggle-pin").checked = S.pinExpanded;
  if (S.soundEnabled) $("sound-picker").classList.remove("hidden");

  // Set dropdown selected text
  const soundLabel = { glass: "Glass", ping: "Ping", pop: "Pop", chime: "Chime", bell: "Bell" };
  $("sound-dropdown").querySelector(".dropdown-selected").textContent = soundLabel[S.soundName] || "Glass";
  $("sound-dropdown").querySelectorAll(".dropdown-option").forEach(o =>
    o.classList.toggle("active", o.dataset.value === S.soundName)
  );
  initCustomDropdown();

  await fitWindow();

  if (S.pinExpanded) {
    $("btn-pin").classList.add("active");
    showView("expanded");
  }

  // ── Hooks ──
  try {
    serverPort = await invoke("get_server_port");
    if (!S.hooksDismissed) {
      if (await invoke("check_hooks_setup")) {
        $("setup-banner").classList.remove("hidden");
        fitWindow();
      }
    }
  } catch (e) {}

  $("btn-setup").addEventListener("click", async () => {
    try { await invoke("install_hooks", { port: serverPort }); } catch (e) {}
    save("hooksDismissed", true);
    $("setup-banner").classList.add("hidden");
    fitWindow();
  });
  $("btn-skip").addEventListener("click", () => {
    save("hooksDismissed", true);
    $("setup-banner").classList.add("hidden");
    fitWindow();
  });

  // ── Drag ──
  $("capsule").addEventListener("mousedown", (e) => {
    if (e.buttons === 1) invoke("plugin:window|start_dragging", { label: "main" }).catch(() => {});
  });

  // ── Hover expand (with cooldown to prevent collapse→re-expand loop) ──
  $("capsule").addEventListener("mouseenter", () => {
    if (currentView === "capsule" && !S.pinExpanded && (Date.now() - collapsedAt > 500)) {
      showView("expanded");
    }
  });

  // ── Collapse: listen to Rust cursor-left event ──
  // Register a callback that Tauri's event system will call
  const callbackId = window.__TAURI_INTERNALS__.transformCallback((event) => {
    if (currentView === "expanded" && !S.pinExpanded) {
      showView("capsule");
    }
  });
  // Tell Tauri to route "cursor-left" events to our callback
  invoke("plugin:event|listen", {
    event: "cursor-left",
    target: { kind: "Any" },
    handler: callbackId,
  }).catch(() => {});

  // Fallback: also check :hover every 200ms
  setInterval(() => {
    if (currentView !== "expanded" || S.pinExpanded) return;
    if (!document.getElementById("app").matches(":hover")) {
      showView("capsule");
    }
  }, 200);

  // ── Pin ──
  $("btn-pin").addEventListener("click", () => {
    save("pinExpanded", !S.pinExpanded);
    $("toggle-pin").checked = S.pinExpanded;
    $("btn-pin").classList.toggle("active", S.pinExpanded);
    if (!S.pinExpanded && currentView === "expanded") showView("capsule");
  });

  // ── Settings ──
  $("btn-settings").addEventListener("click", () => showView("settings"));
  $("btn-close-settings").addEventListener("click", () => showView(S.pinExpanded ? "expanded" : "capsule"));

  $("toggle-pin").addEventListener("change", (e) => {
    save("pinExpanded", e.target.checked);
    $("btn-pin").classList.toggle("active", S.pinExpanded);
    if (!S.pinExpanded) showView("capsule");
    else showView("expanded");
  });

  $("toggle-sound").addEventListener("change", (e) => {
    save("soundEnabled", e.target.checked);
    $("sound-picker").classList.toggle("hidden", !e.target.checked);
    fitWindow();
  });

  document.querySelectorAll(".color-dot").forEach(d => d.addEventListener("click", () => {
    save("accentColor", d.dataset.color); applyAccentColor(d.dataset.color);
  }));
  document.querySelectorAll(".size-btn").forEach(b => b.addEventListener("click", () => {
    save("textSize", b.dataset.size); applyTextSize(b.dataset.size); fitWindow();
  }));

  $("btn-quit").addEventListener("click", () => { try { window.close(); } catch (e) {} });

  refreshState();
  setInterval(refreshState, 1000);

  // Re-register cursor-left listener after a delay to ensure it's active
  setTimeout(() => {
    const cb2 = window.__TAURI_INTERNALS__.transformCallback((event) => {
      if (currentView === "expanded" && !S.pinExpanded) showView("capsule");
    });
    invoke("plugin:event|listen", { event: "cursor-left", target: { kind: "Any" }, handler: cb2 }).catch(() => {});
  }, 2000);
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
  if (s) {
    $("capsule-project").textContent = s.project_name;
    $("capsule-status").textContent = ({ working: "Working...", waiting_for_user: "Waiting", stale: "Stale" })[s.state] || "Idle";
    document.querySelector(".spark-icon").className = "spark-icon " + (({ working: "working", waiting_for_user: "waiting_for_user", stale: "stale" })[s.state] || "idle");
    $("capsule-time").textContent = s.is_active ? s.formatted_time : "";
    $("capsule-time").style.display = s.is_active ? "" : "none";
  } else {
    $("capsule-project").textContent = "Pulse"; $("capsule-status").textContent = "";
    document.querySelector(".spark-icon").className = "spark-icon idle";
    $("capsule-time").style.display = "none";
  }
  if (st.session_count > 1) {
    $("capsule-count").classList.remove("hidden");
    let h = "";
    if (st.active_count > 0) { h += `<span class="count-active">${st.active_count}</span>`; if (st.active_count < st.session_count) h += `<span class="count-sep">/</span>`; }
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
      <div class="session-dot ${sc}"></div>
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

function applyAccentColor(n) {
  const c = COLORS[n] || COLORS.purple;
  document.documentElement.style.setProperty("--accent", c);
  document.documentElement.style.setProperty("--accent-dim", c.replace("rgb(", "rgba(").replace(")", ",0.6)"));
  document.querySelectorAll(".color-dot").forEach(d => d.classList.toggle("active", d.dataset.color === n));
}
function applyTextSize(s) {
  document.documentElement.style.setProperty("--scale", SCALES[s] || 1);
  document.querySelectorAll(".size-btn").forEach(b => b.classList.toggle("active", b.dataset.size === s));
}

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
