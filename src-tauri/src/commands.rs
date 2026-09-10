use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{ContactInput, CoworkerInput, Database};
use crate::graph::GraphClient;
use crate::hotkey::HotkeyManager;
use crate::models::{Category, Contact, Coworker, Note, Reminder, SearchResult, Tag};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub open: String,
    pub save_close: String,
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
        .map_err(|e| e.to_string())
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
        .map_err(|e| e.to_string())
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

// --- Tag commands ---

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

// --- Category commands ---

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

// --- Reminder commands ---

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

// --- Graph commands ---

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
    db.list_contacts().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_contacts(
    db: State<'_, Arc<Database>>,
    query: String,
) -> Result<Vec<Contact>, String> {
    db.search_contacts(&query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_contact(
    db: State<'_, Arc<Database>>,
    contact: ContactPayload,
) -> Result<Contact, String> {
    db.create_contact(&ContactInput::from(&contact))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_contact(
    db: State<'_, Arc<Database>>,
    id: i64,
    contact: ContactPayload,
) -> Result<Contact, String> {
    db.update_contact(id, &ContactInput::from(&contact))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_contact(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_contact(id).map_err(|e| e.to_string())
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
    db.list_coworkers().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_coworkers(
    db: State<'_, Arc<Database>>,
    query: String,
) -> Result<Vec<Coworker>, String> {
    db.search_coworkers(&query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_coworker(
    db: State<'_, Arc<Database>>,
    coworker: CoworkerPayload,
) -> Result<Coworker, String> {
    db.create_coworker(&CoworkerInput::from(&coworker))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_coworker(
    db: State<'_, Arc<Database>>,
    id: i64,
    coworker: CoworkerPayload,
) -> Result<Coworker, String> {
    db.update_coworker(id, &CoworkerInput::from(&coworker))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_coworker(db: State<'_, Arc<Database>>, id: i64) -> Result<(), String> {
    db.delete_coworker(id).map_err(|e| e.to_string())
}

// --- CSV import/export for contacts ---

#[tauri::command]
pub fn export_contacts_csv(db: State<'_, Arc<Database>>) -> Result<String, String> {
    let contacts = db.list_contacts().map_err(|e| e.to_string())?;
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
        db.create_contact(&input).map_err(|e| e.to_string())?;
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
    to_email: String,
) -> Result<(), String> {
    use tauri_plugin_shell::ShellExt;

    let note = db.get_note(note_id).map_err(|e| e.to_string())?;
    let contact = note.contact.as_ref().ok_or("No contact linked to this note")?;

    let subject = format!(
        "Telephone rapport — {} {}",
        contact.first_name, contact.last_name
    );

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

    let mut body = format!("Contact: {} {}", contact.first_name, contact.last_name);
    if let Some(ref company) = contact.company_name {
        body.push_str(&format!("\nCompany: {}", company));
    }
    if !info_parts.is_empty() {
        body.push_str(&format!("\n{}", info_parts.join(", ")));
    }
    if let Some(ref email) = contact.email {
        body.push_str(&format!("\n{}", email));
    }
    body.push_str(&format!("\n\nPlease Recall!\n{}", note.content));

    // Build mailto URL — opens Outlook (or default mail client) with pre-filled email
    let subject_enc = urlencoding::encode(&subject);
    let body_enc = urlencoding::encode(&body);
    let mailto = format!("mailto:{}?subject={}&body={}", to_email, subject_enc, body_enc);

    app.shell()
        .open(mailto, None)
        .map_err(|e| format!("Failed to open mail client: {}", e))
}

#[tauri::command]
pub async fn create_calendar_with_contact(
    db: State<'_, Arc<Database>>,
    graph: State<'_, GraphClient>,
    note_id: i64,
    due_at: String,
) -> Result<String, String> {
    let note = db.get_note(note_id).map_err(|e| e.to_string())?;

    // Build title from contact name + identifier
    let mut title = note.title.clone();
    if let Some(ref contact) = note.contact {
        let contact_name = if contact.kind == "company" && contact.company_name.is_some() {
            contact.company_name.as_ref().unwrap().clone()
        } else {
            format!("{} {}", contact.first_name, contact.last_name)
        };
        title = if contact.customer_identifier.is_some() {
            format!("{} ({})", contact_name, contact.customer_identifier.as_ref().unwrap())
        } else {
            contact_name
        };
    }

    // Create calendar event with 1h duration
    let event_id = graph
        .create_event_with_duration(&title, &due_at, &note.content, 60)
        .map_err(|e| e.to_string())?;

    // Also set a reminder in the DB and link the calendar event
    let reminder = db.set_reminder(note_id, &due_at).map_err(|e| e.to_string())?;
    db.update_reminder_calendar_event(reminder.id, &event_id)
        .map_err(|e| e.to_string())?;

    Ok(event_id)
}
