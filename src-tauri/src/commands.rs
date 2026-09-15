use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{ContactInput, CoworkerInput, Database, TemplateInput, SavedSearchInput};
use crate::graph::GraphClient;
use crate::hotkey::HotkeyManager;
use crate::models::{Category, Contact, Coworker, Note, NoteStatistics, NoteTemplate, Reminder, SavedSearch, SearchResult, Tag};

/// Maps internal errors to user-friendly messages.
fn user_error(e: impl std::fmt::Display) -> String {
    let msg = e.to_string();
    if msg.contains("UNIQUE constraint") {
        "An item with that name already exists.".to_string()
    } else if msg.contains("FOREIGN KEY constraint") {
        "Referenced item does not exist.".to_string()
    } else if msg.contains("timed out") || msg.contains("timeout") {
        "Network request timed out. Check your connection and try again.".to_string()
    } else if msg.contains("401") || msg.contains("Unauthorized") {
        "Outlook session expired — please sign in again in Settings.".to_string()
    } else if msg.contains("duplicate column name") {
        "Database column already exists.".to_string()
    } else {
        msg
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub open: String,
    pub save_close: String,
    pub quick_capture: String,
}

// --- Note commands ---

#[tauri::command]
pub fn create_note(
    db: State<'_, Arc<Database>>,
    title: String,
    content: String,
    category_id: Option<i64>,
    contact_id: Option<i64>,
    coworker_id: Option<i64>,
) -> Result<Note, String> {
    db.create_note_full(&title, &content, category_id, contact_id, coworker_id)
        .map_err(user_error)
}

#[tauri::command]
pub fn update_note(
    db: State<'_, Arc<Database>>,
    id: i64,
    title: String,
    content: String,
    category_id: Option<i64>,
    contact_id: Option<i64>,
    coworker_id: Option<i64>,
) -> Result<Note, String> {
    db.update_note_full(id, &title, &content, category_id, contact_id, coworker_id)
        .map_err(user_error)
}

#[tauri::command]
pub fn delete_note(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_note(id).map_err(user_error)
}

#[tauri::command]
pub fn get_note(db: State<'_, Arc<Database>>, id: i64) -> Result<Note, String> {
    db.get_note(id).map_err(user_error)
}

#[tauri::command]
pub fn list_notes(
    db: State<'_, Arc<Database>>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Note>, String> {
    // Always include archived notes; the frontend filters them based on user preference
    db.list_notes_filtered(limit.unwrap_or(500), offset.unwrap_or(0), true)
        .map_err(user_error)
}

#[tauri::command]
pub fn search_notes(
    db: State<'_, Arc<Database>>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<SearchResult>, String> {
    db.search_notes(&query, limit.unwrap_or(50))
        .map_err(user_error)
}

// --- Tag commands ---

#[tauri::command]
pub fn list_tags(db: State<'_, Arc<Database>>) -> Result<Vec<Tag>, String> {
    db.list_tags().map_err(user_error)
}

#[tauri::command]
pub fn add_tag_to_note(
    db: State<'_, Arc<Database>>,
    note_id: i64,
    tag_name: String,
) -> Result<Tag, String> {
    db.add_tag_to_note(note_id, &tag_name)
        .map_err(user_error)
}

#[tauri::command]
pub fn remove_tag_from_note(
    db: State<'_, Arc<Database>>,
    note_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    db.remove_tag_from_note(note_id, tag_id)
        .map_err(user_error)
}

// --- Category commands ---

#[tauri::command]
pub fn list_categories(db: State<'_, Arc<Database>>) -> Result<Vec<Category>, String> {
    db.list_categories().map_err(user_error)
}

#[tauri::command]
pub fn create_category(
    db: State<'_, Arc<Database>>,
    name: String,
    color: String,
) -> Result<Category, String> {
    db.create_category(&name, &color).map_err(user_error)
}

// --- Reminder commands ---

#[tauri::command]
pub fn set_reminder(
    db: State<'_, Arc<Database>>,
    graph: State<'_, GraphClient>,
    note_id: i64,
    due_at: String,
    create_calendar_event: bool,
    recur_interval: Option<i64>,
    recur_unit: Option<String>,
) -> Result<Reminder, String> {
    let reminder = db
        .set_reminder(note_id, &due_at, recur_interval, recur_unit.as_deref())
        .map_err(user_error)?;

    if create_calendar_event {
        let note = db.get_note(note_id).map_err(user_error)?;
        match graph.create_event(&note.title, &reminder.due_at, &note.content) {
            Ok(event_id) => {
                db.update_reminder_calendar_event(reminder.id, &event_id)
                    .map_err(user_error)?;
            }
            Err(e) => {
                log::warn!("Failed to create calendar event: {}", e);
            }
        }
    }

    db.get_reminder_for_note(note_id)
        .map_err(user_error)?
        .ok_or_else(|| "Reminder not found after insert".to_string())
}

#[tauri::command]
pub fn delete_reminder(db: State<'_, Arc<Database>>, note_id: i64) -> Result<(), String> {
    db.delete_reminder(note_id).map_err(user_error)
}

// --- Graph commands ---

#[tauri::command]
pub async fn graph_sign_in(graph: State<'_, GraphClient>) -> Result<bool, String> {
    graph.sign_in().await.map_err(user_error)
}

#[tauri::command]
pub async fn graph_sign_out(graph: State<'_, GraphClient>) -> Result<(), String> {
    graph.sign_out().await.map_err(user_error)
}

#[tauri::command]
pub fn graph_is_signed_in(graph: State<'_, GraphClient>) -> Result<bool, String> {
    Ok(graph.is_signed_in())
}

#[tauri::command]
pub fn get_graph_client_id(graph: State<'_, GraphClient>) -> Result<String, String> {
    graph.get_client_id()
}

#[tauri::command]
pub fn set_graph_client_id(graph: State<'_, GraphClient>, client_id: String) -> Result<(), String> {
    graph.set_client_id(client_id)
}

// --- Widget commands ---

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

// --- Hotkey commands ---

#[tauri::command]
pub fn get_hotkeys(hotkey_manager: State<'_, HotkeyManager>) -> Result<HotkeyConfig, String> {
    Ok(HotkeyConfig {
        open: hotkey_manager.get_open_hotkey(),
        save_close: hotkey_manager.get_save_close_hotkey(),
        quick_capture: hotkey_manager.get_quick_capture_hotkey(),
    })
}

#[tauri::command]
pub fn set_open_hotkey(
    hotkey_manager: State<'_, HotkeyManager>,
    hotkey: String,
) -> Result<(), String> {
    hotkey_manager.set_open_hotkey(&hotkey)
}

#[tauri::command]
pub fn set_save_close_hotkey(
    hotkey_manager: State<'_, HotkeyManager>,
    hotkey: String,
) -> Result<(), String> {
    hotkey_manager.set_save_close_hotkey(&hotkey)
}

#[tauri::command]
pub fn set_quick_capture_hotkey(
    hotkey_manager: State<'_, HotkeyManager>,
    hotkey: String,
) -> Result<(), String> {
    hotkey_manager.set_quick_capture_hotkey(&hotkey)
}

// --- Contact commands ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactPayload {
    pub last_name: String,
    pub first_name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub kind: String,
    pub company_name: Option<String>,
    pub customer_identifier: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
}

impl From<&ContactPayload> for ContactInput {
    fn from(c: &ContactPayload) -> Self {
        ContactInput {
            last_name: c.last_name.clone(),
            first_name: c.first_name.clone(),
            address: c.address.clone(),
            email: c.email.clone(),
            gender: c.gender.clone(),
            kind: c.kind.clone(),
            company_name: c.company_name.clone(),
            customer_identifier: c.customer_identifier.clone(),
            phone: c.phone.clone(),
            mobile: c.mobile.clone(),
        }
    }
}

#[tauri::command]
pub fn list_contacts(db: State<'_, Arc<Database>>) -> Result<Vec<Contact>, String> {
    db.list_contacts().map_err(user_error)
}

#[tauri::command]
pub fn search_contacts(
    db: State<'_, Arc<Database>>,
    query: String,
) -> Result<Vec<Contact>, String> {
    db.search_contacts(&query).map_err(user_error)
}

#[tauri::command]
pub fn create_contact(
    db: State<'_, Arc<Database>>,
    contact: ContactPayload,
) -> Result<Contact, String> {
    db.create_contact(&ContactInput::from(&contact))
        .map_err(user_error)
}

#[tauri::command]
pub fn update_contact(
    db: State<'_, Arc<Database>>,
    id: i64,
    contact: ContactPayload,
) -> Result<Contact, String> {
    db.update_contact(id, &ContactInput::from(&contact))
        .map_err(user_error)
}

#[tauri::command]
pub fn delete_contact(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_contact(id).map_err(user_error)
}

// --- Coworker commands ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoworkerPayload {
    pub last_name: String,
    pub first_name: String,
    pub email: Option<String>,
}

impl From<&CoworkerPayload> for CoworkerInput {
    fn from(c: &CoworkerPayload) -> Self {
        CoworkerInput {
            last_name: c.last_name.clone(),
            first_name: c.first_name.clone(),
            email: c.email.clone(),
        }
    }
}

#[tauri::command]
pub fn list_coworkers(db: State<'_, Arc<Database>>) -> Result<Vec<Coworker>, String> {
    db.list_coworkers().map_err(user_error)
}

#[tauri::command]
pub fn search_coworkers(
    db: State<'_, Arc<Database>>,
    query: String,
) -> Result<Vec<Coworker>, String> {
    db.search_coworkers(&query).map_err(user_error)
}

#[tauri::command]
pub fn create_coworker(
    db: State<'_, Arc<Database>>,
    coworker: CoworkerPayload,
) -> Result<Coworker, String> {
    db.create_coworker(&CoworkerInput::from(&coworker))
        .map_err(user_error)
}

#[tauri::command]
pub fn update_coworker(
    db: State<'_, Arc<Database>>,
    id: i64,
    coworker: CoworkerPayload,
) -> Result<Coworker, String> {
    db.update_coworker(id, &CoworkerInput::from(&coworker))
        .map_err(user_error)
}

#[tauri::command]
pub fn delete_coworker(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_coworker(id).map_err(user_error)
}

// --- CSV import/export for contacts ---

#[tauri::command]
pub fn export_contacts_csv(db: State<'_, Arc<Database>>) -> Result<String, String> {
    let contacts = db.list_contacts().map_err(user_error)?;
    let mut csv = String::new();
    // Header
    csv.push_str("first_name,last_name,address,email,gender,kind,company_name,customer_identifier,phone,mobile\n");
    for c in &contacts {
        let esc = |s: &Option<String>| -> String {
            match s {
                Some(v) => {
                    if v.contains(',') || v.contains('"') || v.contains('\n') {
                        format!("\"{}\"", v.replace('"', "\"\""))
                    } else {
                        v.clone()
                    }
                }
                None => String::new(),
            }
        };
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            esc(&Some(c.first_name.clone())),
            esc(&Some(c.last_name.clone())),
            esc(&c.address),
            esc(&c.email),
            esc(&c.gender),
            esc(&Some(c.kind.clone())),
            esc(&c.company_name),
            esc(&c.customer_identifier),
            esc(&c.phone),
            esc(&c.mobile),
        ));
    }
    Ok(csv)
}

#[tauri::command]
pub fn export_contacts_to_file(
    db: State<'_, Arc<Database>>,
    path: String,
) -> Result<(), String> {
    let csv = export_contacts_csv(db)?;
    std::fs::write(&path, csv).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
pub fn import_contacts_from_file(
    db: State<'_, Arc<Database>>,
    path: String,
) -> Result<i32, String> {
    let csv_content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    import_contacts_csv(db, csv_content)
}

#[tauri::command]
pub fn import_contacts_csv(
    db: State<'_, Arc<Database>>,
    csv_content: String,
) -> Result<i32, String> {
    let mut lines = csv_content.lines();
    let header = lines.next().ok_or("Empty CSV")?;
    let header_cols: Vec<String> = header.split(',').map(|c| c.trim().to_lowercase()).collect();

    let col_idx = |name: &str| -> Option<usize> {
        header_cols.iter().position(|c| c == name)
    };

    let mut count = 0i32;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<String> = parse_csv_line(line);
        let get = |name: &str| -> Option<String> {
            col_idx(name).and_then(|i| fields.get(i).map(|s| s.trim().to_string())).filter(|s| !s.is_empty())
        };

        let input = ContactInput {
            last_name: get("last_name").unwrap_or_default(),
            first_name: get("first_name").unwrap_or_default(),
            address: get("address"),
            email: get("email"),
            gender: get("gender"),
            kind: get("kind").unwrap_or_else(|| "private".to_string()),
            company_name: get("company_name"),
            customer_identifier: get("customer_identifier"),
            phone: get("phone"),
            mobile: get("mobile"),
        };
        if input.last_name.is_empty() && input.first_name.is_empty() && input.company_name.is_none() {
            continue;
        }
        db.create_contact(&input).map_err(user_error)?;
        count += 1;
    }
    Ok(count)
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            '"' if !in_quotes => {
                in_quotes = true;
            }
            ',' if !in_quotes => {
                result.push(current.clone());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    result.push(current);
    result
}

// --- Telephone rapport + calendar with contact ---

#[tauri::command]
pub fn open_telephone_rapport(
    db: State<'_, Arc<Database>>,
    app: tauri::AppHandle,
    note_id: i64,
    to_email: Option<String>,
) -> Result<(), String> {
    let note = db.get_note(note_id).map_err(user_error)?;
    let lang = crate::i18n::ui_language(&app);

    // Resolve recipient: explicit email, or the coworker linked to the note
    let to_email = match to_email {
        Some(e) if !e.is_empty() => e,
        _ => note
            .coworker
            .as_ref()
            .and_then(|c| c.email.clone())
            .ok_or_else(|| crate::i18n::tr(&lang, "rapport.noRecipient").to_string())?,
    };

    // Build subject — use contact name if available, otherwise note title
    let name = note
        .contact
        .as_ref()
        .map(|c| format!("{} {}", c.first_name, c.last_name))
        .unwrap_or_else(|| note.title.clone());
    let subject = crate::i18n::tr(&lang, "rapport.subject").replace("{name}", &name);

    // Build body — contact info is optional
    let mut body = String::new();
    if let Some(ref contact) = note.contact {
        body.push_str(&format!(
            "{}: {} {}",
            crate::i18n::tr(&lang, "rapport.contact"),
            contact.first_name,
            contact.last_name
        ));
        if let Some(ref company) = contact.company_name {
            body.push_str(&format!(
                "\n{}: {}",
                crate::i18n::tr(&lang, "rapport.company"),
                company
            ));
        }
        // Build contact info line: ID, phone, mobile comma-separated
        let mut info_parts: Vec<String> = Vec::new();
        if let Some(ref id) = contact.customer_identifier {
            info_parts.push(id.clone());
        }
        if let Some(ref phone) = contact.phone {
            info_parts.push(phone.clone());
        }
        if let Some(ref mobile) = contact.mobile {
            info_parts.push(mobile.clone());
        }
        if !info_parts.is_empty() {
            body.push_str(&format!("\n{}", info_parts.join(", ")));
        }
        if let Some(ref email) = contact.email {
            body.push_str(&format!("\n{}", email));
        }
    } else {
        body.push_str(crate::i18n::tr(&lang, "rapport.noContact"));
    }
    body.push_str(&format!(
        "\n\n{}\n{}",
        crate::i18n::tr(&lang, "rapport.recall"),
        note.content
    ));

    // Build mailto URL — opens Outlook (or default mail client) with pre-filled email
    let subject_enc = urlencoding::encode(&subject);
    let body_enc = urlencoding::encode(&body);
    let mailto = format!("mailto:{}?subject={}&body={}", to_email, subject_enc, body_enc);

    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_url(mailto, None::<&str>)
        .map_err(|e| format!("Failed to open mail client: {}", e))
}

#[tauri::command]
pub async fn create_calendar_with_contact(
    db: State<'_, Arc<Database>>,
    graph: State<'_, GraphClient>,
    note_id: i64,
    due_at: String,
) -> Result<String, String> {
    let note = db.get_note(note_id).map_err(user_error)?;

    // Build title from contact name + identifier
    let mut title = note.title.clone();
    if let Some(ref contact) = note.contact {
        let contact_name = if contact.kind == "company" {
            if let Some(company) = &contact.company_name {
                company.clone()
            } else {
                format!("{} {}", contact.first_name, contact.last_name)
            }
        } else {
            format!("{} {}", contact.first_name, contact.last_name)
        };
        title = if let Some(identifier) = &contact.customer_identifier {
            format!("{} ({})", contact_name, identifier)
        } else {
            contact_name
        };
    }

    // Create calendar event with 1h duration
    let event_id = graph
        .create_event_with_duration(&title, &due_at, &note.content, 60)
        .map_err(user_error)?;

    // Also set a reminder in the DB and link the calendar event
    let reminder = db
        .set_reminder(note_id, &due_at, None, None)
        .map_err(user_error)?;
    db.update_reminder_calendar_event(reminder.id, &event_id)
        .map_err(user_error)?;

    Ok(event_id)
}

// --- Calendar event management ---

#[tauri::command]
pub fn delete_calendar_event(
    db: State<'_, Arc<Database>>,
    graph: State<'_, GraphClient>,
    note_id: i64,
) -> Result<(), String> {
    let reminder = db
        .get_reminder_for_note(note_id)
        .map_err(user_error)?
        .ok_or_else(|| "No reminder found for this note".to_string())?;

    if let Some(event_id) = &reminder.calendar_event_id {
        graph.delete_event(event_id).map_err(user_error)?;
    }
    db.delete_reminder(note_id).map_err(user_error)
}

#[tauri::command]
pub fn update_calendar_event(
    db: State<'_, Arc<Database>>,
    graph: State<'_, GraphClient>,
    note_id: i64,
    due_at: String,
) -> Result<(), String> {
    let reminder = db
        .get_reminder_for_note(note_id)
        .map_err(user_error)?
        .ok_or_else(|| "No reminder found for this note".to_string())?;

    let note = db.get_note(note_id).map_err(user_error)?;

    if let Some(event_id) = &reminder.calendar_event_id {
        graph
            .update_event(event_id, &note.title, &due_at, &note.content, 30)
            .map_err(user_error)?;
    }

    db.set_reminder(note_id, &due_at, reminder.recur_interval, reminder.recur_unit.as_deref())
        .map_err(user_error)?;

    Ok(())
}

// --- Archive commands ---

#[tauri::command]
pub fn archive_note(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.archive_note(id).map_err(user_error)
}

#[tauri::command]
pub fn unarchive_note(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.unarchive_note(id).map_err(user_error)
}

// --- Pin commands ---

#[tauri::command]
pub fn pin_note(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.pin_note(id).map_err(user_error)
}

#[tauri::command]
pub fn unpin_note(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.unpin_note(id).map_err(user_error)
}

// --- Note reordering ---

#[tauri::command]
pub fn reorder_note(db: State<'_, Arc<Database>>, id: i64, sort_order: i64) -> Result<(), String> {
    db.reorder_note(id, sort_order).map_err(user_error)
}

// --- Note links ---

#[tauri::command]
pub fn link_notes(db: State<'_, Arc<Database>>, from_id: i64, to_id: i64) -> Result<(), String> {
    db.link_notes(from_id, to_id).map_err(user_error)
}

#[tauri::command]
pub fn unlink_notes(db: State<'_, Arc<Database>>, from_id: i64, to_id: i64) -> Result<(), String> {
    db.unlink_notes(from_id, to_id).map_err(user_error)
}

// --- Recent contacts/coworkers ---

#[tauri::command]
pub fn recent_contacts(db: State<'_, Arc<Database>>, limit: i64) -> Result<Vec<Contact>, String> {
    db.recent_contacts(limit).map_err(user_error)
}

#[tauri::command]
pub fn recent_coworkers(db: State<'_, Arc<Database>>, limit: i64) -> Result<Vec<Coworker>, String> {
    db.recent_coworkers(limit).map_err(user_error)
}

// --- Contact note history ---

#[tauri::command]
pub fn list_notes_for_contact(db: State<'_, Arc<Database>>, contact_id: i64) -> Result<Vec<Note>, String> {
    db.list_notes_for_contact(contact_id).map_err(user_error)
}

// --- Note templates ---

#[tauri::command]
pub fn list_templates(db: State<'_, Arc<Database>>) -> Result<Vec<NoteTemplate>, String> {
    db.list_templates().map_err(user_error)
}

#[tauri::command]
pub fn create_template(
    db: State<'_, Arc<Database>>,
    name: String,
    title: String,
    content: String,
    category_id: Option<i64>,
    tags: String,
) -> Result<NoteTemplate, String> {
    db.create_template(&TemplateInput {
        name,
        title,
        content,
        category_id,
        tags,
    })
    .map_err(user_error)
}

#[tauri::command]
pub fn delete_template(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_template(id).map_err(user_error)
}

// --- Saved searches ---

#[tauri::command]
pub fn list_saved_searches(db: State<'_, Arc<Database>>) -> Result<Vec<SavedSearch>, String> {
    db.list_saved_searches().map_err(user_error)
}

#[tauri::command]
pub fn create_saved_search(
    db: State<'_, Arc<Database>>,
    name: String,
    query: String,
    category_id: Option<i64>,
    tag_name: Option<String>,
    time_range: Option<String>,
) -> Result<SavedSearch, String> {
    db.create_saved_search(&SavedSearchInput {
        name,
        query,
        category_id,
        tag_name,
        time_range,
    })
    .map_err(user_error)
}

#[tauri::command]
pub fn delete_saved_search(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_saved_search(id).map_err(user_error)
}

// --- Statistics ---

#[tauri::command]
pub fn get_statistics(db: State<'_, Arc<Database>>) -> Result<NoteStatistics, String> {
    db.get_statistics().map_err(user_error)
}

// --- Quick capture ---

#[tauri::command]
pub fn quick_capture(
    db: State<'_, Arc<Database>>,
    text: String,
    category_id: Option<i64>,
) -> Result<Note, String> {
    db.create_note_full("", &text, category_id, None, None)
        .map_err(user_error)
}

// --- Single note export ---

#[tauri::command]
pub fn export_note_to_file(
    db: State<'_, Arc<Database>>,
    id: i64,
    path: String,
    format: String,
) -> Result<(), String> {
    let note = db.get_note(id).map_err(user_error)?;
    let content = match format.as_str() {
        "md" => {
            let mut md = format!("# {}\n\n", note.title);
            md.push_str(&note.content);
            if let Some(cat) = &note.category_name {
                md.push_str(&format!("\n\n*Category: {}*", cat));
            }
            if !note.tags.is_empty() {
                let tags: Vec<String> = note.tags.iter().map(|t| t.name.clone()).collect();
                md.push_str(&format!("\n*Tags: {}*", tags.join(", ")));
            }
            md
        }
        _ => {
            let mut txt = format!("{}\n\n", note.title);
            txt.push_str(&note.content);
            txt
        }
    };
    std::fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

// --- Notes CSV/JSON export ---

#[tauri::command]
pub fn export_notes_to_file(
    db: State<'_, Arc<Database>>,
    path: String,
    format: String,
) -> Result<(), String> {
    let notes = db.list_notes_filtered(10000, 0, true).map_err(user_error)?;

    let content = match format.as_str() {
        "json" => {
            serde_json::to_string_pretty(&notes)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))?
        }
        _ => {
            let mut csv = String::from("id,title,content,category,tags,contact,coworker,created_at,updated_at\n");
            for n in &notes {
                let tags: Vec<String> = n.tags.iter().map(|t| t.name.clone()).collect();
                let category = n.category_name.clone().unwrap_or_default();
                let contact = n.contact.as_ref().map(|c| {
                    format!("{} {}", c.first_name, c.last_name)
                }).unwrap_or_default();
                let coworker = n.coworker.as_ref().map(|c| {
                    format!("{} {}", c.first_name, c.last_name)
                }).unwrap_or_default();
                csv.push_str(&format!(
                    "{},\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                    n.id,
                    n.title.replace('"', "\"\""),
                    n.content.replace('"', "\"\"").replace('\n', "\\n"),
                    category,
                    tags.join(";"),
                    contact,
                    coworker,
                    n.created_at,
                    n.updated_at,
                ));
            }
            csv
        }
    };
    std::fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

// --- vCard export ---

#[tauri::command]
pub fn export_contacts_vcard(db: State<'_, Arc<Database>>, path: String) -> Result<(), String> {
    let contacts = db.list_contacts().map_err(user_error)?;
    let mut vcard = String::new();
    for c in &contacts {
        vcard.push_str("BEGIN:VCARD\nVERSION:3.0\n");
        vcard.push_str(&format!("N:{};{}\n", c.last_name, c.first_name));
        vcard.push_str(&format!("FN:{} {}\n", c.first_name, c.last_name));
        if let Some(ref email) = c.email {
            vcard.push_str(&format!("EMAIL:{}\n", email));
        }
        if let Some(ref phone) = c.phone {
            vcard.push_str(&format!("TEL;TYPE=WORK:{}\n", phone));
        }
        if let Some(ref mobile) = c.mobile {
            vcard.push_str(&format!("TEL;TYPE=CELL:{}\n", mobile));
        }
        if let Some(ref company) = c.company_name {
            vcard.push_str(&format!("ORG:{}\n", company));
        }
        if let Some(ref addr) = c.address {
            vcard.push_str(&format!("ADR:;;{};;;;\n", addr));
        }
        vcard.push_str("END:VCARD\n");
    }
    std::fs::write(&path, vcard).map_err(|e| format!("Failed to write file: {}", e))
}

// --- Database backup ---

#[tauri::command]
pub fn backup_database(path: String) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| "Cannot find data directory".to_string())?
        .join("masternote");
    let db_path = data_dir.join("masternote.db");
    std::fs::copy(&db_path, &path).map_err(|e| format!("Failed to backup database: {}", e))?;
    Ok(())
}

// --- Database restore ---

#[tauri::command]
pub fn restore_database(path: String) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| "Cannot find data directory".to_string())?
        .join("masternote");
    let db_path = data_dir.join("masternote.db");
    // Backup current DB first
    let backup_path = data_dir.join("masternote.db.bak");
    if db_path.exists() {
        std::fs::copy(&db_path, &backup_path).map_err(|e| format!("Failed to backup current DB: {}", e))?;
    }
    std::fs::copy(&path, &db_path).map_err(|e| format!("Failed to restore database: {}", e))?;
    Ok(())
}
