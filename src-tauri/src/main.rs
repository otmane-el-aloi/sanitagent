// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod sanitizer;
mod window;

use sanitizer::SanitizationResult;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub struct AppState {
    pub last_result: Mutex<Option<SanitizationResult>>,
}

#[tauri::command]
async fn sanitize_text(
    app: AppHandle,
    text: String,
    state: State<'_, AppState>,
) -> Result<SanitizationResult, String> {
    let result = sanitizer::process_pipeline(text).await;

    // Automatically copy sanitized result back to OS clipboard
    let _ = app.clipboard().write_text(&result.sanitized_text);

    if let Ok(mut guard) = state.last_result.lock() {
        *guard = Some(result.clone());
    }

    Ok(result)
}

#[tauri::command]
async fn read_and_sanitize_clipboard(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SanitizationResult, String> {
    let raw_text = app
        .clipboard()
        .read_text()
        .map_err(|e| format!("Clipboard read error: {}", e))?;

    if raw_text.trim().is_empty() {
        return Err("Clipboard is empty".to_string());
    }

    let result = sanitizer::process_pipeline(raw_text).await;

    // Write sanitized text back to clipboard
    app.clipboard()
        .write_text(&result.sanitized_text)
        .map_err(|e| format!("Clipboard write error: {}", e))?;

    if let Ok(mut guard) = state.last_result.lock() {
        *guard = Some(result.clone());
    }

    // Show window top center
    if let Some(window) = app.get_webview_window("main") {
        window::position_top_center(&app, &window);
        let _ = window.show();
        let _ = app.emit("sanitization-complete", &result);
    }

    Ok(result)
}

#[tauri::command]
fn get_last_result(state: State<'_, AppState>) -> Option<SanitizationResult> {
    state.last_result.lock().ok().and_then(|g| g.clone())
}

#[tauri::command]
fn hide_hud(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn trigger_clipboard_sanitization(app: &AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<AppState>();
        let Ok(raw_text) = app_handle.clipboard().read_text() else {
            return;
        };
        if raw_text.trim().is_empty() {
            return;
        }

        let result = sanitizer::process_pipeline(raw_text).await;
        if app_handle
            .clipboard()
            .write_text(&result.sanitized_text)
            .is_err()
        {
            return;
        }

        if let Ok(mut guard) = state.last_result.lock() {
            *guard = Some(result.clone());
        }

        if let Some(window) = app_handle.get_webview_window("main") {
            window::position_top_center(&app_handle, &window);
            let _ = window.show();
            let _ = app_handle.emit("sanitization-complete", &result);
        }
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            last_result: Mutex::new(None),
        })
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // Configure floating non-activating NSPanel window
            window::configure_hud_window(&window);
            window::position_top_center(app.handle(), &window);

            // Register System Tray
            let quit_item = MenuItem::with_id(app, "quit", "Quit SanitAgent", true, None::<&str>).unwrap();
            let toggle_item = MenuItem::with_id(app, "toggle", "Trigger Sanitizer (Cmd+Shift+S)", true, None::<&str>).unwrap();
            let menu = Menu::with_items(app, &[&toggle_item, &quit_item]).unwrap();

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "toggle" => trigger_clipboard_sanitization(app),
                    _ => {}
                })
                .build(app)?;

            // Register Global Shortcut Cmd + Shift + S
            let shortcut = "CommandOrControl+Shift+S".parse::<Shortcut>().unwrap();
            let app_handle = app.handle().clone();

            let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    trigger_clipboard_sanitization(&app_handle);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sanitize_text,
            read_and_sanitize_clipboard,
            get_last_result,
            hide_hud
        ])
        .run(tauri::generate_context!())
        .expect("error while running SanitAgent application");
}
