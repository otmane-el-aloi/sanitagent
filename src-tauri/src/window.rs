use tauri::{AppHandle, WebviewWindow};

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSFloatingWindowLevel, NSWindow, NSWindowCollectionBehavior};

/// Configures the macOS HUD window as a non-activating floating NSPanel top-center.
pub fn configure_hud_window(window: &WebviewWindow) {
    let _ = window.set_always_on_top(true);

    #[cfg(target_os = "macos")]
    unsafe {
        let ptr = window.ns_window().unwrap() as *mut NSWindow;
        if let Some(ns_window) = ptr.as_ref() {
            // Set floating window level (NSFloatingWindowLevel = 5)
            ns_window.setLevel(NSFloatingWindowLevel);

            // Non-activating panel collection behavior
            let collection_behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::IgnoresCycle;
            ns_window.setCollectionBehavior(collection_behavior);

            ns_window.setHasShadow(true);
        }
    }
}

/// Positions the HUD window top-center of the primary monitor
pub fn position_top_center(_app: &AppHandle, window: &WebviewWindow) {
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let monitor_size = monitor.size();
        let scale_factor = monitor.scale_factor();

        let window_size = window.outer_size().unwrap_or(tauri::PhysicalSize::new(560, 480));

        let monitor_width = monitor_size.width as f64 / scale_factor;
        let window_width = window_size.width as f64 / scale_factor;

        let x = ((monitor_width - window_width) / 2.0) as i32;
        let y = 50; // Top offset (50px from top)

        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
            x as f64, y as f64,
        )));
    }
}
