mod hook_event;
mod hook_server;
mod hooks_configurator;
mod session;

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

#[tauri::command]
fn get_state(manager: tauri::State<AppSessionManager>) -> AppState {
    manager.0.lock().unwrap().get_state()
}

#[tauri::command]
fn select_session(manager: tauri::State<AppSessionManager>, id: String) {
    manager.0.lock().unwrap().select_session(id);
}

#[tauri::command]
fn check_hooks_setup() -> bool {
    hooks_configurator::needs_setup()
}

#[tauri::command]
fn install_hooks(port: u16) -> Result<(), String> {
    hooks_configurator::install(port)
}

#[tauri::command]
fn get_server_port(port_state: tauri::State<ServerPort>) -> u16 {
    port_state.0
}

#[tauri::command]
fn reposition_window(window: tauri::WebviewWindow, position: String) {
    let monitor = match window.primary_monitor() {
        Ok(Some(m)) => m,
        _ => return,
    };
    let scale = monitor.scale_factor();
    let screen_w = monitor.size().width as f64 / scale;
    let screen_h = monitor.size().height as f64 / scale;

    let (x, y) = match position.as_str() {
        "bottom-left" => (12.0, screen_h - 300.0),
        "bottom-right" => (screen_w - 290.0 - 12.0, screen_h - 300.0),
        _ => ((screen_w - 290.0) / 2.0, 8.0), // top-center
    };

    let _ = window.set_position(tauri::Position::Logical(
        tauri::LogicalPosition::new(x, y),
    ));
}

#[tauri::command]
fn focus_session_window(project_name: String, cwd: Option<String>) {
    // Try to find and activate a terminal window matching the session
    // Search by project name first, then by cwd path
    let searches = vec![
        project_name.clone(),
        cwd.clone().unwrap_or_default(),
    ];

    for term in &searches {
        if term.is_empty() { continue; }
        // xdotool search --name returns window IDs matching the title
        if let Ok(output) = std::process::Command::new("xdotool")
            .args(["search", "--name", term])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Ok(wid) = line.trim().parse::<u64>() {
                    // Activate the first matching window
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

            // Down
            let _ = win.set_position(tauri::Position::Logical(
                tauri::LogicalPosition::new(orig_x, orig_y + 8.0),
            ));
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Overshoot up
            let _ = win.set_position(tauri::Position::Logical(
                tauri::LogicalPosition::new(orig_x, orig_y - 3.0),
            ));
            std::thread::sleep(std::time::Duration::from_millis(40));

            // Small bounce
            let _ = win.set_position(tauri::Position::Logical(
                tauri::LogicalPosition::new(orig_x, orig_y + 2.0),
            ));
            std::thread::sleep(std::time::Duration::from_millis(30));

            // Settle
            let _ = win.set_position(tauri::Position::Logical(
                tauri::LogicalPosition::new(orig_x, orig_y),
            ));
        }
    });
}

#[tauri::command]
fn resize_window(window: tauri::WebviewWindow, width: f64, height: f64) {
    let _ = window.set_resizable(true);
    let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)));
    // Keep resizable so user can adjust width if they want
    // Always on top
    let _ = window.set_always_on_top(true);
}

#[tauri::command]
fn is_cursor_inside(window: tauri::WebviewWindow) -> bool {
    let cursor = match window.cursor_position() {
        Ok(c) => c,
        Err(_) => return false,
    };
    let pos = match window.outer_position() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let size = match window.outer_size() {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Add small margin so it doesn't flicker at edges
    let margin = 2.0;
    let in_x = cursor.x >= (pos.x as f64 - margin)
        && cursor.x <= (pos.x as f64 + size.width as f64 + margin);
    let in_y = cursor.y >= (pos.y as f64 - margin)
        && cursor.y <= (pos.y as f64 + size.height as f64 + margin);

    in_x && in_y
}

struct ServerPort(u16);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Logging
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // ── Window setup ──
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(300.0, 46.0)));

                // ── Cursor position polling for hover detection ──
                let win = window.clone();
                let was_inside = Arc::new(AtomicBool::new(false));
                let was_inside_clone = was_inside.clone();

                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(50));

                        // Cursor tracking
                        let inside = (|| {
                            let cursor = win.cursor_position().ok()?;
                            let pos = win.outer_position().ok()?;
                            let size = win.outer_size().ok()?;
                            let margin = 2.0;
                            let in_x = cursor.x >= (pos.x as f64 - margin)
                                && cursor.x <= (pos.x as f64 + size.width as f64 + margin);
                            let in_y = cursor.y >= (pos.y as f64 - margin)
                                && cursor.y <= (pos.y as f64 + size.height as f64 + margin);
                            Some(in_x && in_y)
                        })()
                        .unwrap_or(false);

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
            let session_manager = SessionManager::new();
            app.manage(AppSessionManager(Mutex::new(session_manager)));

            // Start hook server
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
            let quit = MenuItemBuilder::with_id("quit", "Quit ClaudePulse").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show)
                .separator()
                .item(&quit)
                .build()?;

            let icon_bytes = include_bytes!("../icons/32x32.png");
            let icon = Image::from_bytes(icon_bytes)?;

            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("ClaudePulse")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                // Position at top-center of the current monitor
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

            info!("ClaudePulse ready on port {port}");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            select_session,
            check_hooks_setup,
            install_hooks,
            get_server_port,
            reposition_window,
            resize_window,
            bounce_window,
            focus_session_window,
            is_cursor_inside,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ClaudePulse");
}
