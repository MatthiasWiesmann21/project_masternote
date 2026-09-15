use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub category_color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub source: Option<String>,
    pub tags: Vec<Tag>,
    pub reminder: Option<Reminder>,
    pub contact_id: Option<i64>,
    pub coworker_id: Option<i64>,
    pub contact: Option<Contact>,
    pub coworker: Option<Coworker>,
    pub archived: bool,
    pub pinned: bool,
    pub sort_order: i64,
    pub links: Vec<NoteLink>,
    pub backlinks: Vec<NoteLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteLink {
    pub id: i64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: i64,
    pub note_id: i64,
    pub due_at: String,
    pub fired: bool,
    pub calendar_event_id: Option<String>,
    pub recur_interval: Option<i64>,
    pub recur_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub snippet: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: i64,
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
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coworker {
    pub id: i64,
    pub last_name: String,
    pub first_name: String,
    pub email: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteTemplate {
    pub id: i64,
    pub name: String,
    pub title: String,
    pub content: String,
    pub category_id: Option<i64>,
    pub tags: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSearch {
    pub id: i64,
    pub name: String,
    pub query: String,
    pub category_id: Option<i64>,
    pub tag_name: Option<String>,
    pub time_range: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteStatistics {
    pub total_notes: i64,
    pub archived_notes: i64,
    pub notes_with_reminders: i64,
    pub total_contacts: i64,
    pub total_coworkers: i64,
    pub notes_this_week: i64,
    pub notes_per_category: Vec<CategoryCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryCount {
    pub name: String,
    pub count: i64,
    pub color: String,
}
