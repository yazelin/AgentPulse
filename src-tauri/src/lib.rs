mod config;
mod hook_event;
mod hook_server;
mod hooks_configurator;
mod session;

use config::{AppConfig, load_config, save_config, detect_providers};
use hook_server::HookServer;
use log::info;
use session::{AppState, SessionManager};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

struct AppSessionManager(Mutex<SessionManager>);
struct AppConfigState(Mutex<AppConfig>);

#[tauri::command]
fn get_state(manager: tauri::State<AppSessionManager>) -> AppState {
    manager.0.lock().unwrap().get_state()
}

#[tauri::command]
fn select_session(manager: tauri::State<AppSessionManager>, id: String) {
    manager.0.lock().unwrap().select_session(id);
}

#[tauri::command]
fn get_config(config_state: tauri::State<AppConfigState>) -> AppConfig {
    config_state.0.lock().unwrap().clone()
}

#[tauri::command]
fn save_app_config(config_state: tauri::State<AppConfigState>, new_config: AppConfig) -> Result<(), String> {
    save_config(&new_config)?;
    *config_state.0.lock().unwrap() = new_config;
    Ok(())
}

#[tauri::command]
fn detect_installed_providers() -> std::collections::HashMap<String, bool> {
    detect_providers()
}

#[tauri::command]
fn check_provider_setup(provider_id: String, config_state: tauri::State<AppConfigState>) -> bool {
    let config = config_state.0.lock().unwrap();
    if let Some(provider) = config.providers.get(&provider_id) {
        hooks_configurator::provider_needs_setup(&provider_id, provider)
    } else {
        true
    }
}

#[tauri::command]
fn install_provider_hooks(
    provider_id: String,
    config_state: tauri::State<AppConfigState>,
    port_state: tauri::State<ServerPort>,
) -> Result<(), String> {
    let mut config = config_state.0.lock().unwrap();
    if let Some(provider) = config.providers.get(&provider_id) {
        hooks_configurator::install_provider(&provider_id, provider, port_state.0)?;
        // Mark as enabled
        if let Some(p) = config.providers.get_mut(&provider_id) {
            p.enabled = true;
        }
        save_config(&config).ok();
        Ok(())
    } else {
        Err(format!("Unknown provider: {provider_id}"))
    }
}

#[tauri::command]
fn get_server_port(port_state: tauri::State<ServerPort>) -> u16 {
    port_state.0
}

#[tauri::command]
fn focus_session_window(project_name: String, cwd: Option<String>) {
    let searches = vec![project_name, cwd.unwrap_or_default()];
    for term in &searches {
        if term.is_empty() { continue; }
        if let Ok(output) = std::process::Command::new("xdotool")
            .args(["search", "--name", term])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Ok(wid) = line.trim().parse::<u64>() {
                    let _ = std::process::Command::new("xdotool")
                        .args(["windowactivate", &wid.to_string()])
                        .spawn();
                    return;
                }
            }
        }
    }
}

#[tauri::command]
fn bounce_window(window: tauri::WebviewWindow) {
    let win = window.clone();
    std::thread::spawn(move || {
        if let Ok(pos) = win.outer_position() {
            let scale = win.scale_factor().unwrap_or(1.0);
            let orig_y = pos.y as f64 / scale;
            let orig_x = pos.x as f64 / scale;

            let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(orig_x, orig_y + 8.0)));
            std::thread::sleep(std::time::Duration::from_millis(50));
            let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(orig_x, orig_y - 3.0)));
            std::thread::sleep(std::time::Duration::from_millis(40));
            let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(orig_x, orig_y + 2.0)));
            std::thread::sleep(std::time::Duration::from_millis(30));
            let _ = win.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(orig_x, orig_y)));
        }
    });
}

#[tauri::command]
fn resize_window(window: tauri::WebviewWindow, width: f64, height: f64) {
    let _ = window.set_resizable(true);
    let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)));
    let _ = window.set_always_on_top(true);
}

#[tauri::command]
fn is_cursor_inside(window: tauri::WebviewWindow) -> bool {
    let cursor = match window.cursor_position() { Ok(c) => c, Err(_) => return false };
    let pos = match window.outer_position() { Ok(p) => p, Err(_) => return false };
    let size = match window.outer_size() { Ok(s) => s, Err(_) => return false };
    let margin = 2.0;
    cursor.x >= (pos.x as f64 - margin)
        && cursor.x <= (pos.x as f64 + size.width as f64 + margin)
        && cursor.y >= (pos.y as f64 - margin)
        && cursor.y <= (pos.y as f64 + size.height as f64 + margin)
}

struct ServerPort(u16);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Load config
            let config = load_config();
            save_config(&config).ok(); // Ensure file exists with defaults
            app.manage(AppConfigState(Mutex::new(config)));

            // Window setup
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(300.0, 46.0)));

                // Cursor position polling
                let win = window.clone();
                let was_inside = Arc::new(AtomicBool::new(false));
                let was_inside_clone = was_inside.clone();

                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        let inside = (|| {
                            let cursor = win.cursor_position().ok()?;
                            let pos = win.outer_position().ok()?;
                            let size = win.outer_size().ok()?;
                            let margin = 2.0;
                            Some(
                                cursor.x >= (pos.x as f64 - margin)
                                && cursor.x <= (pos.x as f64 + size.width as f64 + margin)
                                && cursor.y >= (pos.y as f64 - margin)
                                && cursor.y <= (pos.y as f64 + size.height as f64 + margin)
                            )
                        })().unwrap_or(false);

                        let was = was_inside_clone.load(Ordering::Relaxed);
                        if inside != was {
                            was_inside_clone.store(inside, Ordering::Relaxed);
                            if inside {
                                let _ = win.emit("cursor-entered", ());
                            } else {
                                let _ = win.emit("cursor-left", ());
                            }
                        }
                    }
                });
            }

            // Session manager
            app.manage(AppSessionManager(Mutex::new(SessionManager::new())));

            // Hook server
            let handle = app.handle().clone();
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            let port = rt.block_on(async {
                let mut server = HookServer::new();
                match server.start().await {
                    Ok(mut rx) => {
                        let port = server.port();
                        let h = handle.clone();
                        tokio::spawn(async move {
                            while let Some(event) = rx.recv().await {
                                let mgr = h.state::<AppSessionManager>();
                                let completed = {
                                    let mut m = mgr.0.lock().unwrap();
                                    m.handle_event(&event)
                                };
                                let _ = h.emit("session-update", ());
                                if completed {
                                    let _ = h.emit("task-completed", ());
                                }
                            }
                        });
                        port
                    }
                    Err(e) => {
                        log::error!("Failed to start server: {e}");
                        0
                    }
                }
            });

            std::mem::forget(rt);
            app.manage(ServerPort(port));

            // Staleness checker
            let handle2 = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(10));
                let mgr = handle2.state::<AppSessionManager>();
                mgr.0.lock().unwrap().check_staleness();
                let _ = handle2.emit("session-update", ());
            });

            // System tray
            let show = MenuItemBuilder::with_id("show", "Show/Hide").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit AgentPulse").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show)
                .separator()
                .item(&quit)
                .build()?;

            let icon_bytes = include_bytes!("../icons/32x32.png");
            let icon = Image::from_bytes(icon_bytes)?;

            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("AgentPulse")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                if let Ok(Some(monitor)) = w.current_monitor() {
                                    let scale = monitor.scale_factor();
                                    let pos = monitor.position();
                                    let sw = monitor.size().width as f64 / scale;
                                    let x = pos.x as f64 / scale + (sw / 2.0 - 145.0);
                                    let y = pos.y as f64 / scale + 8.0;
                                    let _ = w.set_position(tauri::Position::Logical(
                                        tauri::LogicalPosition::new(x, y),
                                    ));
                                }
                                let _ = w.show();
                                let _ = w.set_always_on_top(true);
                                let _ = w.set_focus();
                            }
                        }
                    }
                    "quit" => {
                        hook_server::remove_port_file();
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            info!("AgentPulse ready on port {port}");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            select_session,
            get_config,
            save_app_config,
            detect_installed_providers,
            check_provider_setup,
            install_provider_hooks,
            get_server_port,
            resize_window,
            bounce_window,
            focus_session_window,
            is_cursor_inside,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AgentPulse");
}
