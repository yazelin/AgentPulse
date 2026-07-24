# Changelog

All notable changes land here. Each tagged release on
[GitHub Releases](https://github.com/yazelin/AgentPulse/releases) pulls its
description from the matching `## [vX.Y.Z]` section via `release.yml`.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.4.0] — 2026-07-24

Replace Gemini CLI with Antigravity CLI (`agy`).

Google's Gemini CLI is no longer supported, so its provider is removed and
replaced with **Antigravity CLI** (`agy`, Google's gemini_coder). agy uses a
fundamentally different hook model — not a drop-in rename.

### Added

- **Antigravity CLI (`agy`) provider.** Hooks live in `~/.gemini/config/hooks.json`
  as a named-hook schema (`{name: {Event: [handlers]}}`), distinct from every
  other provider. AgentPulse wires `PreInvocation` → `UserPromptSubmit`,
  `PostToolUse`, and `Stop`. Detection matches the `agy` binary or the
  `~/.gemini/antigravity-cli/` config dir. New completion/waiting sounds
  (`antigravity.mp3` / `antigravity-waiting.mp3`, zh-TW 曉臻 voice) and the
  official Antigravity icon (from @lobehub/icons).
- **Sidecar event-name injection.** agy's stdin payload carries no event-name
  field (the event is implied by the hooks.json key) and its hooks are
  synchronous (must print a JSON result on stdout). The sidecar now takes an
  optional 2nd arg — the event name — folds it into the POST body as
  `hook_event_name`, and echoes `{}`. Only emitted when the 2nd arg is present,
  so fire-and-forget providers stay silent. `conversationId` added as a
  `session_id` alias.
- **Config migration for existing installs.** `load_config` now reconciles the
  persisted provider set against the known defaults — dropping the retired
  `gemini` entry and adding `antigravity` — so upgraders see the new provider
  in the UI (and can install its hooks) instead of a dead Gemini toggle. Stale
  per-provider sound entries are pruned; the JS auto-match repopulates the new
  provider's sounds on next launch.

### Removed

- **Gemini CLI provider** — config entry, `install_gemini_hooks`, its
  `BeforeAgent`/`AfterAgent`/`BeforeModel`/… event mappings, the
  `hook_cmd_powershell` helper (Gemini was its only user), and the
  `gemini.mp3` / `gemini-waiting.mp3` sounds. Existing dead Gemini hooks in
  `~/.gemini/settings.json` are harmless (Gemini CLI is deprecated) but can be
  removed by hand.

## [v0.3.0] — 2026-05-28

Mori body-interface integration + Linux launch polish.

### Added

- **Body-interface bridge for [Mori](https://github.com/yazelin/mori-desktop)** —
  the existing hook server now exposes GET `/health`, `/manifest`, `/sessions`,
  and an SSE `/events` stream that re-broadcasts session transitions
  (`StartedWaiting` → `cue.waiting_input`, `Completed` → `cue.done`) in the
  Mori event envelope. On startup AgentPulse writes a body manifest to
  `~/.mori/body-parts/mori.agent-pulse/manifest.json` (with the live port) so
  Mori Desktop's body registry discovers it automatically. Detection pipeline
  (per-CLI hooks → `SessionManager`) is unchanged; only an outward read
  surface is added. Zero new dependencies.

### Changed

- **Linux scripts default to native Wayland** — `dev.sh` and `watch.sh` no
  longer hard-force `GDK_BACKEND=x11`. Native Wayland avoids XWayland's
  transparent-window ghosting on hybrid GPUs. Opt back into XWayland with
  `AGENTPULSE_GDK_X11=1 ./dev.sh` if always-on-top sinks on your compositor.
- **Tauri CLI fallback in scripts** — `./dev.sh` / `./watch.sh` now resolve a
  Tauri CLI before building: prefer the local `node_modules/.bin/tauri`
  (`@tauri-apps/cli`), else fall back to the installed `cargo-tauri`. Scripts
  work whether or not `npm install` has been run.
- **Collapse bounce reverted to real window movement** — the v0.2.3 → v0.2.4
  CSS-transform bounce ghosted transparent windows on some compositors (the
  translated content's vacated area wasn't repainted). Restored the original
  Rust `bounce_window` (`set_position`), which goes through the WM's damage
  path cleanly. The CSS keyframe remains in `styles.css` as a dormant
  fallback.

## [v0.2.4] — 2026-05-11

Compat fixes for the Codex v0.129+ feature-flag rename and for
always-on-top under GNOME Wayland.

### Fixed

- **Codex `[features].codex_hooks` deprecation warning** — Codex v0.129
  renamed the flag to `[features].hooks` and prints a deprecation banner
  on every launch under the old name. `install_codex_hooks` now writes
  the new key, and `ensure_codex_hooks_feature` migrates existing
  configs in place: line-by-line patching that preserves user
  comments / indent / formatting, dedupes if both flags coexist, and
  ignores commented or unrelated keys. Covered by 6 unit tests.
- **Always-on-top under GNOME Wayland** — GNOME / Mutter resets
  `_NET_WM_STATE_ABOVE` whenever the window loses focus, so the capsule
  drifted behind other windows after every focus change. Now re-asserted
  on `Focused(false)` on Linux. `dev.sh` and `watch.sh` also force
  `GDK_BACKEND=x11` so the XWayland path (which respects the hint
  reliably) is used during development.

### Changed

- **Switched to npm-installed Tauri CLI** — `@tauri-apps/cli` is now a
  devDependency; no global `cargo install tauri-cli` required for
  contributors. `default-run = "agent-pulse"` added to `Cargo.toml` so
  bare `cargo run` no longer errors on the two-binary workspace.

### Note

Codex v0.129+ also added per-hook trust review. After enabling Codex in
AgentPulse on a fresh machine, run `codex` once and approve the listed
hooks via `/hooks` — they only need to be approved once per machine.

[v0.2.4]: https://github.com/yazelin/AgentPulse/releases/tag/v0.2.4

## [v0.2.3] — 2026-04-17

Fixes spurious mid-turn completion sounds and swallowed end-of-turn
signals on Gemini CLI and GitHub Copilot.

### Fixed

- **Gemini: completion sound firing mid-turn** — `AfterModel` was mapped
  to `Stop`, but Gemini's `BeforeModel`/`AfterModel` pair fires once per
  model call, and a single turn typically runs 2+ model calls when tools
  are used. The mapping caused a `Completed` transition after every
  model call (playing the task-completed sound mid-turn), and then the
  real `AfterAgent` at end-of-turn was swallowed because state was
  already `Idle`. `AfterModel` now falls through to the state machine's
  no-op arm; `AfterAgent` alone carries the per-turn completion signal.
- **Copilot: `subagentStop` spuriously emitting completion** — same shape
  as the Gemini bug. A user prompt that spawns a subagent would fire
  `subagentStop` when the child finished (mid-turn), flipping state to
  `Idle` and firing the completion sound; the parent's `agentStop` would
  then be swallowed. `subagentStop` no longer maps to `Stop` — only
  `agentStop` does.

[v0.2.3]: https://github.com/yazelin/AgentPulse/releases/tag/v0.2.3

## [v0.2.2] — 2026-04-17

Hotfix for Gemini CLI hook execution on Windows.

### Fixed

- **Gemini CLI hooks on Windows** — Gemini hardcodes
  `powershell.exe -NoProfile -Command` for hook execution, and PowerShell
  parses `"path\to\exe.exe" arg` as a bare string expression (ParserError:
  UnexpectedToken at the argument) rather than a call. The Gemini hook
  command is now prefixed with PowerShell's `&` call operator on Windows
  only; cmd.exe and bash on other platforms still see the unchanged form.
- **Hook dedup marker** — `provider_needs_setup` / `remove_provider` looked
  for the substring `"agentpulse"` to identify previously installed
  AgentPulse hooks, but the v0.2 sidecar command contains
  `agent-pulse-hook` (hyphenated). The marker never matched, so toggling a
  provider on/off accumulated duplicate hook entries. Marker now matches
  the sidecar filename across shells and OSes.

[v0.2.2]: https://github.com/yazelin/AgentPulse/releases/tag/v0.2.2

## [v0.2.1] — 2026-04-16

First public release. Multi-provider hook monitoring with a sidecar-binary
architecture that works across Linux, macOS (Apple Silicon + Intel), and
Windows.

### Supported platforms

Each tag produces four zip archives: `linux`, `macos-arm64`, `macos-x64`,
`windows`. The two macOS zips are cross-compiled on the same Apple-Silicon
runner (GitHub retired the standalone Intel runner).

### Added

- **Sidecar hook binary** — CLI hooks invoke a standalone executable
  (`agent-pulse-hook`) instead of an inline bash one-liner. The sidecar reads
  the event JSON from stdin, resolves the server port from
  `~/.agentpulse/port`, and POSTs to the AgentPulse HTTP server. One command
  string works identically across bash, PowerShell, and cmd.exe, so
  Claude Code, Gemini CLI, Codex, and GitHub Copilot all share the same
  hook invocation.
- **Per-provider waiting sounds** — new `SessionTransition::StartedWaiting`
  fires a `task-waiting` event whenever a session first hits
  `WaitingForUser`; the frontend plays a dedicated per-provider clip
  independent of the completion clip. Ships eight bundled TTS sounds
  (`{provider}.mp3` + `{provider}-waiting.mp3`, voiced with
  `zh-TW-HsiaoChenNeural` / 曉臻).
- **Single-instance plugin** — `tauri-plugin-single-instance` forwards a
  second launch to the running window (show + focus) and exits. Fixes the
  ghost tray icon that appeared when a duplicate launch's HTTP server
  refused the port but the UI still spawned.
- **Landing page + live demo** — `docs/` is a GitHub Pages site with a
  full interactive AgentPulse embedded in an iframe (powered by a mocked
  Tauri IPC shim so the real `src/main.js` boots without a backend).
  Download buttons auto-fill from the GitHub releases API.
- **CHANGELOG + auto-populated release notes** — this file. The release
  workflow extracts the section matching the tag and passes it to
  `softprops/action-gh-release` as the draft body.
- **Project docs** — `CLAUDE.md` committed, covering the v0.2 architecture
  for anyone cloning the repo.

### Changed

- **Providers default to disabled** — toggling a provider on in Settings
  now immediately installs its hooks. Claude previously defaulted to
  `enabled: true` but the install flow didn't fire at startup, leaving the
  checkbox "on" with no actual hook installed.
- **Notification Sounds toggle label** — renamed from "Sound on Complete"
  to "Notification Sounds" since it now gates both completion and waiting
  clips.
- **Capsule bounce animation** — the collapse-to-capsule bounce runs as a
  pure CSS `@keyframes` animation (`capsuleCollapseBounce`) instead of the
  Rust-side `bounce_window` shim. Transform-only + 260 ms keeps it on the
  GPU and avoids X11 ghosting on transparent windows.
- **`seed_default_sounds` runs on every launch** — idempotent write, so
  existing installs pick up newly bundled defaults automatically.
- **CI speedups** — `cargo-binstall` for the Tauri CLI (dropped Windows
  install from ~10 min to ~45 s). Upgraded `actions/checkout@v5` and
  `actions/upload-artifact@v6` to clear Node 20 deprecation warnings.

### Fixed

- **Windows DWM border** — `"shadow": false` in `tauri.conf.json` removes
  the halo/ghost border visible on Windows 10/11.
- **Capsule count `3/3`** — when every session was active, the `/`
  separator was dropped and the counts jammed together as `33`. Always
  renders the slash now when at least one session is active.

### Known limitations

- **No installer** (`.msi` / `.deb` / `.dmg`) yet — bundle packaging of the
  sidecar binary across installer formats is still unresolved. Ships as
  zip only; keep the two binaries in the same folder.
- **Codex CLI on Windows** — upstream disables hook execution. AgentPulse
  still writes the hook config, so when OpenAI re-enables it the
  integration lights up automatically.
- **No code signing** — macOS users need `xattr -cr` to bypass Gatekeeper;
  Windows users see SmartScreen's "More info → Run anyway". Apple
  Developer cert ($99/yr) is the real fix — intentionally deferred.

[v0.2.1]: https://github.com/yazelin/AgentPulse/releases/tag/v0.2.1
