# MasterNote

A fast, cross-platform quick-note widget app built with Tauri 2 + Rust + SvelteKit + TypeScript.

## Dev commands

```bash
# Install frontend deps
npm install

# Run in dev mode (starts Vite + Tauri)
npm run tauri dev

# Build production installers
npm run tauri build

# Type-check frontend
npm run check

# Run Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run Rust clippy
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

## Architecture

- `src-tauri/src/lib.rs` — Tauri app entry, plugin registration, tray, global hotkey
- `src-tauri/src/db.rs` — SQLite via rusqlite (bundled + FTS5), migrations, CRUD
- `src-tauri/src/commands.rs` — `#[tauri::command]` functions exposed to frontend
- `src-tauri/src/graph.rs` — Microsoft Graph OAuth (device code flow) + event creation
- `src-tauri/src/reminders.rs` — Background scheduler for due reminders → notifications
- `src-tauri/src/models.rs` — Serde structs (Note, Tag, Category, Reminder)
- `src-tauri/migrations/0001_init.sql` — Schema + FTS5 virtual table + triggers
- `src/lib/api.ts` — `invoke()` wrappers for all Tauri commands
- `src/lib/stores/` — Svelte stores (notes, settings)
- `src/lib/components/` — NoteEditor, NoteList, SearchBar, TagPicker, ReminderDialog
- `src/routes/+page.svelte` — Main widget UI

## Key bindings

- `Ctrl+Shift+Space` (global) — Toggle widget visibility
- `Ctrl+N` — New note
- `Ctrl+F` — Focus search
- `Ctrl+Enter` — Save current note
- `Esc` — Hide widget

## Microsoft Graph setup

To enable Outlook calendar event creation:

1. Register an app in [Microsoft Entra ID](https://entra.microsoft.com) as a public desktop client (no secret).
2. Add the `Calendars.ReadWrite` and `offline_access` Microsoft Graph scopes.
3. Set the client ID at build/run time:
   ```bash
   $env:MASTERNOTE_GRAPH_CLIENT_ID = "your-client-id"
   npm run tauri dev
   ```
4. Sign in via the Settings panel in the widget (device code flow).

## Storage

- SQLite database at `<user_data_dir>/masternote/masternote.db`
- Full-text search via FTS5 virtual table synced with triggers
- Window position/size persisted via `tauri-plugin-window-state`
- Graph tokens stored via `tauri-plugin-store` in `graph.json`
