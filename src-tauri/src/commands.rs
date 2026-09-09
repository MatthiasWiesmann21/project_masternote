use std::sync::Arc;

use tauri::State;

use crate::db::Database;
use crate::graph::GraphClient;
use crate::models::{Category, Note, Reminder, SearchResult, Tag};

#[tauri::command]
pub fn create_note(
    db: State<'_, Arc<Database>>,
    title: String,
    content: String,
    category_id: Option<i64>,
) -> Result<Note, String> {
    db.create_note(&title, &content, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note(
    db: State<'_, Arc<Database>>,
    id: i64,
    title: String,
    content: String,
    category_id: Option<i64>,
) -> Result<Note, String> {
    db.update_note(id, &title, &content, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_note(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_note(db: State<'_, Arc<Database>>, id: i64) -> Result<Note, String> {
    db.get_note(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_notes(
    db: State<'_, Arc<Database>>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Note>, String> {
    db.list_notes(limit.unwrap_or(100), offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_notes(
    db: State<'_, Arc<Database>>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    db.search_notes(&query, limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tags(db: State<'_, Arc<Database>>) -> Result<Vec<Tag>, String> {
    db.list_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag_to_note(
    db: State<'_, Arc<Database>>,
    note_id: i64,
    tag_name: String,
) -> Result<Tag, String> {
    db.add_tag_to_note(note_id, &tag_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_tag_from_note(
    db: State<'_, Arc<Database>>,
    note_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    db.remove_tag_from_note(note_id, tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_categories(db: State<'_, Arc<Database>>) -> Result<Vec<Category>, String> {
    db.list_categories().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_category(
    db: State<'_, Arc<Database>>,
    name: String,
    color: String,
) -> Result<Category, String> {
    db.create_category(&name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_reminder(
    db: State<'_, Arc<Database>>,
    graph: State<'_, GraphClient>,
    note_id: i64,
    due_at: String,
    create_calendar_event: bool,
) -> Result<Reminder, String> {
    let reminder = db
        .set_reminder(note_id, &due_at)
        .map_err(|e| e.to_string())?;

    if create_calendar_event {
        let note = db.get_note(note_id).map_err(|e| e.to_string())?;
        match graph.create_event(&note.title, &reminder.due_at, &note.content) {
            Ok(event_id) => {
                db.update_reminder_calendar_event(reminder.id, &event_id)
                    .map_err(|e| e.to_string())?;
            }
            Err(e) => {
                log::warn!("Failed to create calendar event: {}", e);
            }
        }
    }

    db.get_reminder_for_note(note_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Reminder not found after insert".to_string())
}

#[tauri::command]
pub fn delete_reminder(db: State<'_, Arc<Database>>, note_id: i64) -> Result<(), String> {
    db.delete_reminder(note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn graph_sign_in(graph: State<'_, GraphClient>) -> Result<bool, String> {
    graph.sign_in().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn graph_sign_out(graph: State<'_, GraphClient>) -> Result<(), String> {
    graph.sign_out().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn graph_is_signed_in(graph: State<'_, GraphClient>) -> Result<bool, String> {
    Ok(graph.is_signed_in())
}

#[tauri::command]
pub fn hide_widget(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|e| format!("Failed to hide window: {}", e))?;
    }
    Ok(())
}
