use masternote_lib::db::{Database, TemplateInput, SavedSearchInput};

fn setup() -> Database {
    let path = masternote_lib::db::test_db_path();
    let db = Database::open(&path).expect("Failed to open db");
    db.run_migrations().expect("Failed to run migrations");
    db
}

#[test]
fn test_create_and_get_note() {
    let db = setup();
    let note = db
        .create_note("Test title", "Test content", None)
        .expect("create_note failed");
    assert_eq!(note.title, "Test title");
    assert_eq!(note.content, "Test content");
    assert!(note.id > 0);

    let fetched = db.get_note(note.id).expect("get_note failed");
    assert_eq!(fetched.title, "Test title");
    assert_eq!(fetched.content, "Test content");
}

#[test]
fn test_update_note() {
    let db = setup();
    let note = db
        .create_note("Original", "Original content", None)
        .expect("create_note failed");
    let updated = db
        .update_note(note.id, "Updated", "Updated content", None)
        .expect("update_note failed");
    assert_eq!(updated.title, "Updated");
    assert_eq!(updated.content, "Updated content");
}

#[test]
fn test_delete_note() {
    let db = setup();
    let note = db
        .create_note("To delete", "Delete me", None)
        .expect("create_note failed");
    db.delete_note(note.id).expect("delete_note failed");
    let result = db.get_note(note.id);
    assert!(result.is_err());
}

#[test]
fn test_list_notes() {
    let db = setup();
    db.create_note("Note 1", "Content 1", None).expect("create failed");
    db.create_note("Note 2", "Content 2", None).expect("create failed");
    db.create_note("Note 3", "Content 3", None).expect("create failed");

    let notes = db.list_notes(100, 0).expect("list_notes failed");
    assert_eq!(notes.len(), 3);
}

#[test]
fn test_search_notes() {
    let db = setup();
    db.create_note("Meeting notes", "Discussed quarterly budget", None)
        .expect("create failed");
    db.create_note("Grocery list", "Milk, eggs, bread", None)
        .expect("create failed");

    let results = db.search_notes("budget", 50).expect("search failed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Meeting notes");

    let results = db.search_notes("milk", 50).expect("search failed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Grocery list");
}

#[test]
fn test_tags() {
    let db = setup();
    let note = db
        .create_note("Tagged note", "Content", None)
        .expect("create failed");

    let tag = db
        .add_tag_to_note(note.id, "important")
        .expect("add_tag failed");
    assert_eq!(tag.name, "important");

    let tags = db.list_tags().expect("list_tags failed");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "important");

    let note_tags = db.get_tags_for_note(note.id).expect("get_tags failed");
    assert_eq!(note_tags.len(), 1);
    assert_eq!(note_tags[0].name, "important");

    db.remove_tag_from_note(note.id, tag.id)
        .expect("remove_tag failed");
    let note_tags = db.get_tags_for_note(note.id).expect("get_tags failed");
    assert_eq!(note_tags.len(), 0);
}

#[test]
fn test_categories() {
    let db = setup();
    let cat = db
        .create_category("Work", "#ff0000")
        .expect("create_category failed");
    assert_eq!(cat.name, "Work");
    assert_eq!(cat.color, "#ff0000");

    let cats = db.list_categories().expect("list_categories failed");
    // 4 default + 1 new
    assert_eq!(cats.len(), 5);

    let note = db
        .create_note("Work note", "Content", Some(cat.id))
        .expect("create failed");
    assert_eq!(note.category_id, Some(cat.id));
    assert_eq!(note.category_name.as_deref(), Some("Work"));
}

#[test]
fn test_reminders() {
    let db = setup();
    let note = db
        .create_note("Reminded note", "Content", None)
        .expect("create failed");

    let due_at = "2099-01-01T12:00:00.000Z";
    let reminder = db
        .set_reminder(note.id, due_at, None, None)
        .expect("set_reminder failed");
    assert_eq!(reminder.note_id, note.id);
    assert_eq!(reminder.due_at, due_at);
    assert!(!reminder.fired);

    let fetched = db
        .get_reminder_for_note(note.id)
        .expect("get_reminder failed")
        .expect("no reminder");
    assert_eq!(fetched.due_at, due_at);

    db.mark_reminder_fired(reminder.id)
        .expect("mark_fired failed");
    let fetched = db
        .get_reminder_for_note(note.id)
        .expect("get_reminder failed")
        .expect("no reminder");
    assert!(fetched.fired);

    db.delete_reminder(note.id).expect("delete_reminder failed");
    let fetched = db.get_reminder_for_note(note.id).expect("get_reminder failed");
    assert!(fetched.is_none());
}

#[test]
fn test_due_reminders() {
    let db = setup();
    let note = db
        .create_note("Due note", "Content", None)
        .expect("create failed");

    let past = "2000-01-01T12:00:00.000Z";
    db.set_reminder(note.id, past, None, None).expect("set_reminder failed");

    let due = db.list_due_reminders().expect("list_due failed");
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].1.id, note.id);
}

#[test]
fn test_default_categories_exist() {
    let db = setup();
    let cats = db.list_categories().expect("list_categories failed");
    let names: Vec<&str> = cats.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"General"));
    assert!(names.contains(&"Call notes"));
    assert!(names.contains(&"Ideas"));
    assert!(names.contains(&"Tasks"));
}

#[test]
fn test_fts_search_snippet() {
    let db = setup();
    db.create_note(
        "Project plan",
        "We need to ship the new feature by next week and test it thoroughly",
        None,
    )
    .expect("create failed");

    let results = db.search_notes("feature", 50).expect("search failed");
    assert_eq!(results.len(), 1);
    assert!(!results[0].snippet.is_empty());
}

#[test]
fn test_archive_note() {
    let db = setup();
    let note = db
        .create_note("To archive", "Content", None)
        .expect("create failed");

    db.archive_note(note.id).expect("archive failed");
    let fetched = db.get_note(note.id).expect("get_note failed");
    assert!(fetched.archived);

    db.unarchive_note(note.id).expect("unarchive failed");
    let fetched = db.get_note(note.id).expect("get_note failed");
    assert!(!fetched.archived);
}

#[test]
fn test_reorder_note() {
    let db = setup();
    let n1 = db.create_note("Note 1", "c1", None).expect("create failed");
    let n2 = db.create_note("Note 2", "c2", None).expect("create failed");

    db.reorder_note(n2.id, 100).expect("reorder failed");
    db.reorder_note(n1.id, 50).expect("reorder failed");

    let fetched1 = db.get_note(n1.id).expect("get failed");
    let fetched2 = db.get_note(n2.id).expect("get failed");
    assert_eq!(fetched1.sort_order, 50);
    assert_eq!(fetched2.sort_order, 100);
}

#[test]
fn test_recurring_reminder() {
    let db = setup();
    let note = db
        .create_note("Recurring note", "Content", None)
        .expect("create failed");

    let due_at = "2099-01-01T12:00:00.000Z";
    let reminder = db
        .set_reminder(note.id, due_at, Some(7), Some("days"))
        .expect("set_reminder failed");

    assert_eq!(reminder.recur_interval, Some(7));
    assert_eq!(reminder.recur_unit.as_deref(), Some("days"));
}

#[test]
fn test_note_links() {
    let db = setup();
    let n1 = db.create_note("Source", "content 1", None).expect("create failed");
    let n2 = db.create_note("Target", "content 2", None).expect("create failed");

    db.link_notes(n1.id, n2.id).expect("link failed");
    db.unlink_notes(n1.id, n2.id).expect("unlink failed");
}

#[test]
fn test_templates() {
    let db = setup();
    let template = db
        .create_template(&TemplateInput {
            name: "Meeting".to_string(),
            title: "Meeting notes".to_string(),
            content: "Attendees:\n\nAgenda:\n".to_string(),
            category_id: None,
            tags: "meeting,notes".to_string(),
        })
        .expect("create_template failed");

    assert_eq!(template.name, "Meeting");
    let templates = db.list_templates().expect("list_templates failed");
    assert_eq!(templates.len(), 1);

    db.delete_template(template.id).expect("delete_template failed");
    let templates = db.list_templates().expect("list_templates failed");
    assert_eq!(templates.len(), 0);
}

#[test]
fn test_saved_searches() {
    let db = setup();
    let search = db
        .create_saved_search(&SavedSearchInput {
            name: "My tasks".to_string(),
            query: "task".to_string(),
            category_id: None,
            tag_name: None,
            time_range: None,
        })
        .expect("create_saved_search failed");

    assert_eq!(search.name, "My tasks");
    let searches = db.list_saved_searches().expect("list_saved_searches failed");
    assert_eq!(searches.len(), 1);

    db.delete_saved_search(search.id).expect("delete_saved_search failed");
    let searches = db.list_saved_searches().expect("list_saved_searches failed");
    assert_eq!(searches.len(), 0);
}

#[test]
fn test_statistics() {
    let db = setup();
    db.create_note("Note 1", "content", None).expect("create failed");
    db.create_note("Note 2", "content", None).expect("create failed");

    let stats = db.get_statistics().expect("get_statistics failed");
    assert!(stats.total_notes >= 2);
    assert_eq!(stats.archived_notes, 0);
}

#[test]
fn test_migration_versioning() {
    let db = setup();
    let version = db.get_schema_version().expect("get_schema_version failed");
    assert!(version >= 9, "schema version should be at least 9, got {}", version);
}
