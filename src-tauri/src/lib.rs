pub mod commands;
pub mod db;
pub mod graph;
pub mod hotkey;
pub mod models;
pub mod reminders;

use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

use db::Database;
use graph::GraphClient;
use hotkey::HotkeyManager;

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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Open database in user data dir
            let data_dir = dirs::data_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("masternote");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("masternote.db");
            log::info!("Database path: {}", db_path.display());

            let db = Database::open(db_path.to_str().unwrap())?;
            db.run_migrations()?;
            let db = Arc::new(db);

            // Graph client
            let graph = GraphClient::new(app.handle())?;

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

            // Register configurable global hotkeys
            let hotkey_manager = HotkeyManager::new(app.handle().clone());
            if let Err(e) = hotkey_manager.register_all() {
                log::error!("Failed to register hotkeys: {}", e);
            }
            app.manage(hotkey_manager);

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
            commands::get_graph_client_id,
            commands::set_graph_client_id,
            commands::hide_widget,
            commands::get_hotkeys,
            commands::set_open_hotkey,
            commands::set_save_close_hotkey,
            commands::set_quick_capture_hotkey,
            commands::list_contacts,
            commands::search_contacts,
            commands::create_contact,
            commands::update_contact,
            commands::delete_contact,
            commands::list_coworkers,
            commands::search_coworkers,
            commands::create_coworker,
            commands::update_coworker,
            commands::delete_coworker,
            commands::export_contacts_csv,
            commands::export_contacts_to_file,
            commands::import_contacts_csv,
            commands::import_contacts_from_file,
            commands::open_telephone_rapport,
            commands::create_calendar_with_contact,
            commands::delete_calendar_event,
            commands::update_calendar_event,
            commands::archive_note,
            commands::unarchive_note,
            commands::reorder_note,
            commands::link_notes,
            commands::unlink_notes,
            commands::recent_contacts,
            commands::recent_coworkers,
            commands::list_notes_for_contact,
            commands::list_templates,
            commands::create_template,
            commands::delete_template,
            commands::list_saved_searches,
            commands::create_saved_search,
            commands::delete_saved_search,
            commands::get_statistics,
            commands::quick_capture,
            commands::export_note_to_file,
            commands::export_notes_to_file,
            commands::export_contacts_vcard,
            commands::backup_database,
            commands::restore_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
