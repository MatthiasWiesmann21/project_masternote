use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;

const GRAPH_SCOPES: &[&str] = &["Calendars.ReadWrite", "offline_access"];
const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphEvent {
    id: String,
}

pub struct GraphClient {
    client: reqwest::Client,
    client_id: Mutex<String>,
    store: Arc<tauri_plugin_store::Store<tauri::Wry>>,
    app: AppHandle,
}

impl GraphClient {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let store = app
            .store("graph.json")
            .map_err(|e| format!("Failed to open store: {}", e))?;

        // Read client ID from store first, then fall back to env var
        let client_id = store
            .get("client_id")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .filter(|s| !s.is_empty())
            .or_else(|| std::env::var("MASTERNOTE_GRAPH_CLIENT_ID").ok())
            .unwrap_or_else(String::new);

        Ok(Self {
            client: reqwest::Client::new(),
            client_id: Mutex::new(client_id),
            store,
            app: app.clone(),
        })
    }

    pub fn get_client_id(&self) -> Result<String, String> {
        let id = self.client_id.lock().unwrap().clone();
        Ok(id)
    }

    pub fn set_client_id(&self, client_id: String) -> Result<(), String> {
        let trimmed = client_id.trim().to_string();
        *self.client_id.lock().unwrap() = trimmed.clone();
        self.store
            .set("client_id", serde_json::Value::String(trimmed));
        self.store.save().ok();
        Ok(())
    }

    pub fn is_signed_in(&self) -> bool {
        self.store
            .get("access_token")
            .map(|v| v.is_string() && !v.as_str().unwrap_or("").is_empty())
            .unwrap_or(false)
    }

    pub async fn sign_in(&self) -> Result<bool, String> {
        let client_id = self.client_id.lock().unwrap().clone();
        if client_id.is_empty() {
            return Err(
                "Microsoft Graph client ID not configured. Enter it in Settings → Outlook."
                    .to_string(),
            );
        }

        // Step 1: Request device code
        let device_code: DeviceCodeResponse = self
            .client
            .post(DEVICE_CODE_URL)
            .form(&[
                ("client_id", client_id.as_str()),
                ("scope", &GRAPH_SCOPES.join(" ")),
            ])
            .send()
            .await
            .map_err(|e| format!("Device code request failed: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse device code response: {}", e))?;

        // Show the user code message via notification + frontend event
        log::info!("Graph sign-in: {}", device_code.message);

        // Emit event so frontend can show the verification URL/code
        let _ = self.app.emit(
            "graph-device-code",
            serde_json::json!({
                "message": device_code.message,
                "user_code": device_code.user_code,
                "verification_uri": device_code.verification_uri,
            }),
        );

        // Also try to open the verification URL in the browser
        if let Some(window) = self.app.get_webview_window("main") {
            let _ = window.set_focus();
        }

        // Step 2: Poll for token
        let interval = device_code.interval.max(5);
        let expires_at = std::time::Instant::now()
            + std::time::Duration::from_secs(device_code.expires_in);

        while std::time::Instant::now() < expires_at {
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

            let res = self
                .client
                .post(TOKEN_URL)
                .form(&[
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("client_id", client_id.as_str()),
                    ("device_code", &device_code.device_code),
                ])
                .send()
                .await;

            match res {
                Ok(resp) => {
                    let status = resp.status();
                    let body: serde_json::Value =
                        resp.json().await.map_err(|e| format!("Token parse error: {}", e))?;

                    if status.is_success() {
                        let token: TokenResponse = serde_json::from_value(body.clone())
                            .map_err(|e| format!("Token deserialize error: {}", e))?;
                        self.store
                            .set("access_token", serde_json::Value::String(token.access_token));
                        if let Some(rt) = token.refresh_token {
                            self.store
                                .set("refresh_token", serde_json::Value::String(rt));
                        }
                        self.store
                            .set(
                                "expires_at",
                                serde_json::Value::String(
                                    chrono::Utc::now()
                                        .timestamp()
                                        .saturating_add(token.expires_in as i64)
                                        .to_string(),
                                ),
                            );
                        self.store.save().ok();
                        return Ok(true);
                    } else {
                        let err = body
                            .get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");
                        if err == "authorization_pending" {
                            continue;
                        }
                        if err == "slow_down" {
                            tokio::time::sleep(std::time::Duration::from_secs(interval + 5)).await;
                            continue;
                        }
                        return Err(format!("Auth failed: {}", err));
                    }
                }
                Err(e) => {
                    log::warn!("Token poll error: {}", e);
                    continue;
                }
            }
        }
        Err("Device code expired".to_string())
    }

    pub async fn sign_out(&self) -> Result<(), String> {
        self.store.delete("access_token");
        self.store.delete("refresh_token");
        self.store.delete("expires_at");
        self.store.save().ok();
        Ok(())
    }

    fn is_token_expired(&self) -> bool {
        let expires_at = self
            .store
            .get("expires_at")
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        match expires_at {
            Some(ts) => {
                let exp: i64 = ts.parse().unwrap_or(0);
                let now = chrono::Utc::now().timestamp();
                // Refresh 5 minutes before expiry
                now >= exp - 300
            }
            None => true,
        }
    }

    async fn refresh_access_token(&self) -> Result<String, String> {
        let refresh_token = self
            .store
            .get("refresh_token")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| "No refresh token — please sign in again".to_string())?;

        let client_id = self.client_id.lock().unwrap().clone();

        let res = self
            .client
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", client_id.as_str()),
                ("refresh_token", &refresh_token),
                ("scope", &GRAPH_SCOPES.join(" ")),
            ])
            .send()
            .await
            .map_err(|e| format!("Token refresh request failed: {}", e))?;

        let status = res.status();
        let body: serde_json::Value =
            res.json().await.map_err(|e| format!("Token refresh parse error: {}", e))?;

        if !status.is_success() {
            let err = body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            return Err(format!("Token refresh failed: {}. Please sign in again.", err));
        }

        let token: TokenResponse = serde_json::from_value(body)
            .map_err(|e| format!("Token refresh deserialize error: {}", e))?;

        self.store
            .set("access_token", serde_json::Value::String(token.access_token.clone()));
        if let Some(rt) = token.refresh_token {
            self.store
                .set("refresh_token", serde_json::Value::String(rt));
        }
        self.store.set(
            "expires_at",
            serde_json::Value::String(
                chrono::Utc::now()
                    .timestamp()
                    .saturating_add(token.expires_in as i64)
                    .to_string(),
            ),
        );
        self.store.save().ok();

        Ok(token.access_token)
    }

    async fn get_valid_access_token(&self) -> Result<String, String> {
        if self.is_token_expired() {
            let has_refresh = self
                .store
                .get("refresh_token")
                .map(|v| v.is_string() && !v.as_str().unwrap_or("").is_empty())
                .unwrap_or(false);

            if has_refresh {
                self.refresh_access_token().await
            } else {
                self.store
                    .get("access_token")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .ok_or_else(|| "Not signed in to Microsoft Graph".to_string())
            }
        } else {
            self.store
                .get("access_token")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .ok_or_else(|| "Not signed in to Microsoft Graph".to_string())
        }
    }

    pub fn create_event(
        &self,
        title: &str,
        start_iso: &str,
        body: &str,
    ) -> Result<String, String> {
        self.create_event_with_duration(title, start_iso, body, 30)
    }

    pub fn create_event_with_duration(
        &self,
        title: &str,
        start_iso: &str,
        body: &str,
        duration_minutes: i64,
    ) -> Result<String, String> {
        let client = self.client.clone();
        let start = start_iso.to_string();
        let title = title.to_string();
        let body = body.to_string();
        let this = self;

        // Compute end time = start + duration
        let start_dt = chrono::DateTime::parse_from_rfc3339(&start)
            .map_err(|e| format!("Invalid start time: {}", e))?;
        let end_dt = start_dt + chrono::Duration::minutes(duration_minutes);
        let end = end_dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        let event_body = serde_json::json!({
            "subject": title,
            "body": {
                "contentType": "Text",
                "content": body
            },
            "start": {
                "dateTime": start,
                "timeZone": "UTC"
            },
            "end": {
                "dateTime": end,
                "timeZone": "UTC"
            },
            "reminderMinutesBeforeStart": 0,
            "isReminderOn": true
        });

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let token = this.get_valid_access_token().await?;

            let resp = client
                .post(format!("{}/me/events", GRAPH_BASE))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&event_body)
                .send()
                .await
                .map_err(|e| format!("Graph request failed: {}", e))?;

            let status = resp.status();
            let body: serde_json::Value =
                resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

            if !status.is_success() {
                return Err(format!(
                    "Graph error: {}",
                    body.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown")
                ));
            }

            let event: GraphEvent = serde_json::from_value(body)
                .map_err(|e| format!("Event parse error: {}", e))?;
            Ok(event.id)
        })
    }

    pub fn delete_event(&self, event_id: &str) -> Result<(), String> {
        let client = self.client.clone();
        let event_id = event_id.to_string();

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let token = self.get_valid_access_token().await?;

            let resp = client
                .delete(format!("{}/me/events/{}", GRAPH_BASE, event_id))
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .map_err(|e| format!("Graph delete request failed: {}", e))?;

            if !resp.status().is_success() {
                let body: serde_json::Value =
                    resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
                return Err(format!(
                    "Graph error: {}",
                    body.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown")
                ));
            }
            Ok(())
        })
    }

    pub fn update_event(
        &self,
        event_id: &str,
        title: &str,
        start_iso: &str,
        body: &str,
        duration_minutes: i64,
    ) -> Result<(), String> {
        let client = self.client.clone();
        let start = start_iso.to_string();
        let title = title.to_string();
        let body = body.to_string();
        let event_id = event_id.to_string();

        let start_dt = chrono::DateTime::parse_from_rfc3339(&start)
            .map_err(|e| format!("Invalid start time: {}", e))?;
        let end_dt = start_dt + chrono::Duration::minutes(duration_minutes);
        let end = end_dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        let event_body = serde_json::json!({
            "subject": title,
            "body": {
                "contentType": "Text",
                "content": body
            },
            "start": {
                "dateTime": start,
                "timeZone": "UTC"
            },
            "end": {
                "dateTime": end,
                "timeZone": "UTC"
            },
            "reminderMinutesBeforeStart": 0,
            "isReminderOn": true
        });

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let token = self.get_valid_access_token().await?;

            let resp = client
                .patch(format!("{}/me/events/{}", GRAPH_BASE, event_id))
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&event_body)
                .send()
                .await
                .map_err(|e| format!("Graph update request failed: {}", e))?;

            if !resp.status().is_success() {
                let body: serde_json::Value =
                    resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
                return Err(format!(
                    "Graph error: {}",
                    body.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown")
                ));
            }
            Ok(())
        })
    }
}
