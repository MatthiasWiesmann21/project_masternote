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
