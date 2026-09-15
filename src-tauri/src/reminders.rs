use std::sync::Arc;
use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::db::Database;

pub fn start_reminder_scheduler(app: AppHandle, db: Arc<Database>) {
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Failed to create reminder scheduler runtime: {}", e);
                return;
            }
        };

        loop {
            rt.block_on(async {
                tokio::time::sleep(Duration::from_secs(30)).await;
                check_and_fire_reminders(&app, &db);
            });
        }
    });
}

fn check_and_fire_reminders(app: &AppHandle, db: &Database) {
    match db.list_due_reminders() {
        Ok(reminders) => {
            for (reminder, note) in reminders {
                log::info!(
                    "Firing reminder {} for note {}",
                    reminder.id,
                    note.id
                );

                // Fire system notification
                let lang = crate::i18n::ui_language(app);
                let title = if note.title.is_empty() {
                    crate::i18n::tr(&lang, "notif.title")
                } else {
                    &note.title
                };
                let body = if note.content.is_empty() {
                    crate::i18n::tr(&lang, "notif.due").to_string()
                } else {
                    note.content.chars().take(120).collect()
                };

                if let Err(e) = app.notification().builder()
                    .title(title)
                    .body(&body)
                    .show()
                {
                    log::warn!("Failed to show notification: {}", e);
                }

                // If recurring, re-arm with next due time; otherwise mark as fired
                if reminder.recur_interval.is_some() && reminder.recur_unit.is_some() {
                    if let Err(e) = db.rearm_recurring_reminder(reminder.id) {
                        log::warn!("Failed to rearm recurring reminder: {}", e);
                    }
                } else if let Err(e) = db.mark_reminder_fired(reminder.id) {
                    log::warn!("Failed to mark reminder as fired: {}", e);
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to list due reminders: {}", e);
        }
    }
}
