pub mod commands;
pub mod db;
pub mod graph;
pub mod models;
pub mod reminders;

use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use db::Database;
use graph::GraphClient;

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            // Open database in user data dir
            let data_dir = dirs::data_dir()
                .unwrap_or_else(|| std::env::temp_dir())
                .join("masternote");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("masternote.db");
            log::info!("Database path: {}", db_path.display());

            let db = Database::open(db_path.to_str().unwrap())?;
            db.run_migrations()?;
            let db = Arc::new(db);

            // Graph client
            let graph = GraphClient::new(&app.handle())?;

            // Start reminder scheduler
            reminders::start_reminder_scheduler(app.handle().clone(), Arc::clone(&db));

            app.manage(db);
            app.manage(graph);

            // System tray
            let toggle_item = MenuItem::with_id(app, "toggle", "Show/Hide", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("MasterNote")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => toggle_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Global hotkey: Ctrl+Shift+Space toggles widget
            let app_handle = app.handle().clone();
            app.global_shortcut()
                .on_shortcut("Ctrl+Shift+Space", move |_app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        toggle_window(&app_handle);
                    }
                })?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Focused(false) = event {
                let _ = window;
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::get_note,
            commands::list_notes,
            commands::search_notes,
            commands::list_tags,
            commands::add_tag_to_note,
            commands::remove_tag_from_note,
            commands::list_categories,
            commands::create_category,
            commands::set_reminder,
            commands::delete_reminder,
            commands::graph_sign_in,
            commands::graph_sign_out,
            commands::graph_is_signed_in,
            commands::hide_widget,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
