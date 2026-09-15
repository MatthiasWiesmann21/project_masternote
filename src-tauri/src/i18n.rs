use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "settings.json";
const KEY_LANGUAGE: &str = "language";

/// Read the UI language from the settings store ("en" | "de" | "fr" | "es").
pub fn ui_language(app: &AppHandle) -> String {
    app.store(STORE_PATH)
        .ok()
        .and_then(|s| s.get(KEY_LANGUAGE))
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "en".to_string())
}

/// Backend-facing translations (email templates, notifications, errors).
pub fn tr<'a>(lang: &str, key: &'a str) -> &'a str {
    match (lang, key) {
        // Telephone rapport email
        (_, "rapport.subject") => match lang {
            "de" => "Telefonrapport — {name}",
            "fr" => "Rapport téléphonique — {name}",
            "es" => "Informe telefónico — {name}",
            _ => "Telephone rapport — {name}",
        },
        (_, "rapport.contact") => match lang {
            "de" => "Kontakt",
            "fr" => "Contact",
            "es" => "Contacto",
            _ => "Contact",
        },
        (_, "rapport.company") => match lang {
            "de" => "Firma",
            "fr" => "Société",
            "es" => "Empresa",
            _ => "Company",
        },
        (_, "rapport.noContact") => match lang {
            "de" => "Kein Kontakt verknüpft",
            "fr" => "Aucun contact associé",
            "es" => "Sin contacto vinculado",
            _ => "No contact linked",
        },
        (_, "rapport.recall") => match lang {
            "de" => "Bitte zurückrufen!",
            "fr" => "Merci de rappeler !",
            "es" => "¡Por favor, devuelva la llamada!",
            _ => "Please Recall!",
        },
        (_, "rapport.noRecipient") => match lang {
            "de" => "Kein Empfänger — der verknüpfte Kollege hat keine E-Mail-Adresse.",
            "fr" => "Aucun destinataire — le collègue associé n'a pas d'adresse e-mail.",
            "es" => "Sin destinatario — el compañero vinculado no tiene correo electrónico.",
            _ => "No recipient — the linked coworker has no email address.",
        },
        // Reminder notifications
        (_, "notif.title") => match lang {
            "de" => "MasterNote-Erinnerung",
            "fr" => "Rappel MasterNote",
            "es" => "Recordatorio de MasterNote",
            _ => "MasterNote reminder",
        },
        (_, "notif.due") => match lang {
            "de" => "Eine Notiz ist fällig.",
            "fr" => "Vous avez une note due.",
            "es" => "Tiene una nota pendiente.",
            _ => "You have a note due.",
        },
        _ => key,
    }
}
