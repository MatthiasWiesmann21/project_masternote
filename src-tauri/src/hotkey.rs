use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "settings.json";
const KEY_OPEN_HOTKEY: &str = "open_hotkey";
const KEY_SAVE_CLOSE_HOTKEY: &str = "save_close_hotkey";
const KEY_QUICK_CAPTURE_HOTKEY: &str = "quick_capture_hotkey";

const DEFAULT_OPEN_HOTKEY: &str = "Ctrl+Shift+M";
const DEFAULT_SAVE_CLOSE_HOTKEY: &str = "Ctrl+Shift+N";
const DEFAULT_QUICK_CAPTURE_HOTKEY: &str = "Ctrl+Shift+Q";

pub struct HotkeyManager {
    app: AppHandle,
}

impl HotkeyManager {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn store(&self) -> Arc<tauri_plugin_store::Store<tauri::Wry>> {
        self.app
            .store(STORE_PATH)
            .expect("Failed to open settings store")
    }

    pub fn get_open_hotkey(&self) -> String {
        self.store()
            .get(KEY_OPEN_HOTKEY)
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| DEFAULT_OPEN_HOTKEY.to_string())
    }

    pub fn get_save_close_hotkey(&self) -> String {
        self.store()
            .get(KEY_SAVE_CLOSE_HOTKEY)
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| DEFAULT_SAVE_CLOSE_HOTKEY.to_string())
    }

    pub fn get_quick_capture_hotkey(&self) -> String {
        self.store()
            .get(KEY_QUICK_CAPTURE_HOTKEY)
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| DEFAULT_QUICK_CAPTURE_HOTKEY.to_string())
    }

    pub fn register_all(&self) -> Result<(), String> {
        let open_hk = self.get_open_hotkey();
        let save_close_hk = self.get_save_close_hotkey();
        let quick_capture_hk = self.get_quick_capture_hotkey();

        self.register_open(&open_hk)?;
        self.register_save_close(&save_close_hk)?;
        self.register_quick_capture(&quick_capture_hk)?;
        Ok(())
    }

    fn register_open(&self, hotkey: &str) -> Result<(), String> {
        let app = self.app.clone();
        self.app
            .global_shortcut()
            .on_shortcut(hotkey, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    toggle_window(&app);
                }
            })
            .map_err(|e| format!("Failed to register open hotkey '{}': {}", hotkey, e))
    }

    fn register_save_close(&self, hotkey: &str) -> Result<(), String> {
        let app = self.app.clone();
        self.app
            .global_shortcut()
            .on_shortcut(hotkey, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    // Emit event to frontend to save and close
                    let _ = app.emit("save-and-close", ());
                }
            })
            .map_err(|e| format!("Failed to register save&close hotkey '{}': {}", hotkey, e))
    }

    fn register_quick_capture(&self, hotkey: &str) -> Result<(), String> {
        let app = self.app.clone();
        self.app
            .global_shortcut()
            .on_shortcut(hotkey, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let _ = app.emit("quick-capture", ());
                }
            })
            .map_err(|e| format!("Failed to register quick-capture hotkey '{}': {}", hotkey, e))
    }

    pub fn set_open_hotkey(&self, new_hotkey: &str) -> Result<(), String> {
        let old = self.get_open_hotkey();
        // Unregister old
        let _ = self.app.global_shortcut().unregister(old.as_str());
        // Register new
        self.register_open(new_hotkey)?;
        // Persist
        let store = self.store();
        store.set(
            KEY_OPEN_HOTKEY,
            serde_json::Value::String(new_hotkey.to_string()),
        );
        store.save().ok();
        Ok(())
    }

    pub fn set_save_close_hotkey(&self, new_hotkey: &str) -> Result<(), String> {
        let old = self.get_save_close_hotkey();
        // Unregister old
        let _ = self.app.global_shortcut().unregister(old.as_str());
        // Register new
        self.register_save_close(new_hotkey)?;
        // Persist
        let store = self.store();
        store.set(
            KEY_SAVE_CLOSE_HOTKEY,
            serde_json::Value::String(new_hotkey.to_string()),
        );
        store.save().ok();
        Ok(())
    }

    pub fn set_quick_capture_hotkey(&self, new_hotkey: &str) -> Result<(), String> {
        let old = self.get_quick_capture_hotkey();
        // Unregister old
        let _ = self.app.global_shortcut().unregister(old.as_str());
        // Register new
        self.register_quick_capture(new_hotkey)?;
        // Persist
        let store = self.store();
        store.set(
            KEY_QUICK_CAPTURE_HOTKEY,
            serde_json::Value::String(new_hotkey.to_string()),
        );
        store.save().ok();
        Ok(())
    }
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
