import { invoke } from '@tauri-apps/api/core';

export interface Note {
  id: number;
  title: string;
  content: string;
  categoryId: number | null;
  categoryName: string | null;
  categoryColor: string | null;
  createdAt: string;
  updatedAt: string;
  source: string | null;
  tags: Tag[];
  reminder: Reminder | null;
  contactId: number | null;
  coworkerId: number | null;
  contact: Contact | null;
  coworker: Coworker | null;
  archived: boolean;
  pinned: boolean;
  sortOrder: number;
  links: NoteLink[];
  backlinks: NoteLink[];
}

export interface NoteLink {
  id: number;
  title: string;
}

export interface Tag {
  id: number;
  name: string;
}

export interface Category {
  id: number;
  name: string;
  color: string;
}

export interface Reminder {
  id: number;
  noteId: number;
  dueAt: string;
  fired: boolean;
  calendarEventId: string | null;
  recurInterval: number | null;
  recurUnit: string | null;
}

export interface SearchResult {
  id: number;
  title: string;
  content: string;
  snippet: string;
  updatedAt: string;
}

export interface Contact {
  id: number;
  lastName: string;
  firstName: string;
  address: string | null;
  email: string | null;
  gender: string | null;
  kind: string; // 'private' or 'company'
  companyName: string | null;
  customerIdentifier: string | null;
  phone: string | null;
  mobile: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface Coworker {
  id: number;
  lastName: string;
  firstName: string;
  email: string | null;
  createdAt: string;
  updatedAt: string;
}

// Note commands
export const createNote = (
  title: string,
  content: string,
  categoryId: number | null,
  contactId: number | null = null,
  coworkerId: number | null = null
) => invoke<Note>('create_note', { title, content, categoryId, contactId, coworkerId });

export const updateNote = (
  id: number,
  title: string,
  content: string,
  categoryId: number | null,
  contactId: number | null = null,
  coworkerId: number | null = null
) => invoke<Note>('update_note', { id, title, content, categoryId, contactId, coworkerId });

export const deleteNote = (id: number) => invoke<void>('delete_note', { id });

export const listNotes = (limit: number = 100, offset: number = 0) =>
  invoke<Note[]>('list_notes', { limit, offset });

export const searchNotes = (query: string, limit: number = 50) =>
  invoke<SearchResult[]>('search_notes', { query, limit });

export const getNote = (id: number) => invoke<Note>('get_note', { id });

// Tag commands
export const listTags = () => invoke<Tag[]>('list_tags');

export const addTagToNote = (noteId: number, tagName: string) =>
  invoke<Tag>('add_tag_to_note', { noteId, tagName });

export const removeTagFromNote = (noteId: number, tagId: number) =>
  invoke<void>('remove_tag_from_note', { noteId, tagId });

// Category commands
export const listCategories = () => invoke<Category[]>('list_categories');

export const createCategory = (name: string, color: string) =>
  invoke<Category>('create_category', { name, color });

// Reminder commands
export const setReminder = (
  noteId: number,
  dueAt: string,
  createCalendarEvent: boolean,
  recurInterval: number | null = null,
  recurUnit: string | null = null
) =>
  invoke<Reminder>('set_reminder', {
    noteId,
    dueAt,
    createCalendarEvent,
    recurInterval,
    recurUnit
  });

export const deleteReminder = (noteId: number) => invoke<void>('delete_reminder', { noteId });

// Microsoft Graph commands
export const graphSignIn = () => invoke<boolean>('graph_sign_in');

export const graphSignOut = () => invoke<void>('graph_sign_out');

export const graphIsSignedIn = () => invoke<boolean>('graph_is_signed_in');

export const getGraphClientId = () => invoke<string>('get_graph_client_id');

export const setGraphClientId = (clientId: string) =>
  invoke<void>('set_graph_client_id', { clientId });

// Widget commands
export const hideWidget = () => invoke<void>('hide_widget');

// Hotkey commands
export interface HotkeyConfig {
  open: string;
  saveClose: string;
  quickCapture: string;
}

export const getHotkeys = () => invoke<HotkeyConfig>('get_hotkeys');

export const setOpenHotkey = (hotkey: string) =>
  invoke<void>('set_open_hotkey', { hotkey });

export const setSaveCloseHotkey = (hotkey: string) =>
  invoke<void>('set_save_close_hotkey', { hotkey });

export const setQuickCaptureHotkey = (hotkey: string) =>
  invoke<void>('set_quick_capture_hotkey', { hotkey });

// Contact commands
export const listContacts = () => invoke<Contact[]>('list_contacts');

export const searchContacts = (query: string) =>
  invoke<Contact[]>('search_contacts', { query });

export const createContact = (contact: Omit<Contact, 'id' | 'createdAt' | 'updatedAt'>) =>
  invoke<Contact>('create_contact', { contact });

export const updateContact = (id: number, contact: Omit<Contact, 'id' | 'createdAt' | 'updatedAt'>) =>
  invoke<Contact>('update_contact', { id, contact });

export const deleteContact = (id: number) => invoke<void>('delete_contact', { id });

// Coworker commands
export const listCoworkers = () => invoke<Coworker[]>('list_coworkers');

export const searchCoworkers = (query: string) =>
  invoke<Coworker[]>('search_coworkers', { query });

export const createCoworker = (coworker: Omit<Coworker, 'id' | 'createdAt' | 'updatedAt'>) =>
  invoke<Coworker>('create_coworker', { coworker });

export const updateCoworker = (id: number, coworker: Omit<Coworker, 'id' | 'createdAt' | 'updatedAt'>) =>
  invoke<Coworker>('update_coworker', { id, coworker });

export const deleteCoworker = (id: number) => invoke<void>('delete_coworker', { id });

// CSV import/export for contacts
export const exportContactsCsv = () => invoke<string>('export_contacts_csv');

export const exportContactsToFile = (path: string) =>
  invoke<void>('export_contacts_to_file', { path });

export const importContactsCsv = (csvContent: string) =>
  invoke<number>('import_contacts_csv', { csvContent });

export const importContactsFromFile = (path: string) =>
  invoke<number>('import_contacts_from_file', { path });

// Telephone rapport + calendar with contact
export const openTelephoneRapport = (noteId: number, toEmail: string | null) =>
  invoke<void>('open_telephone_rapport', { noteId, toEmail });

export const createCalendarWithContact = (noteId: number, dueAt: string) =>
  invoke<string>('create_calendar_with_contact', { noteId, dueAt });

// Calendar event management
export const deleteCalendarEvent = (noteId: number) =>
  invoke<void>('delete_calendar_event', { noteId });

export const updateCalendarEvent = (noteId: number, dueAt: string) =>
  invoke<void>('update_calendar_event', { noteId, dueAt });

// Archive
export const archiveNote = (id: number) => invoke<void>('archive_note', { id });
export const unarchiveNote = (id: number) => invoke<void>('unarchive_note', { id });
export const pinNote = (id: number) => invoke<void>('pin_note', { id });
export const unpinNote = (id: number) => invoke<void>('unpin_note', { id });

// Reorder
export const reorderNote = (id: number, sortOrder: number) =>
  invoke<void>('reorder_note', { id, sortOrder });

// Note links
export const linkNotes = (fromId: number, toId: number) =>
  invoke<void>('link_notes', { fromId, toId });
export const unlinkNotes = (fromId: number, toId: number) =>
  invoke<void>('unlink_notes', { fromId, toId });

// Recent contacts/coworkers
export const recentContacts = (limit: number = 5) =>
  invoke<Contact[]>('recent_contacts', { limit });
export const recentCoworkers = (limit: number = 5) =>
  invoke<Coworker[]>('recent_coworkers', { limit });

// Contact note history
export const listNotesForContact = (contactId: number) =>
  invoke<Note[]>('list_notes_for_contact', { contactId });

// Note templates
export interface NoteTemplate {
  id: number;
  name: string;
  title: string;
  content: string;
  categoryId: number | null;
  tags: string;
}

export const listTemplates = () => invoke<NoteTemplate[]>('list_templates');
export const createTemplate = (
  name: string,
  title: string,
  content: string,
  categoryId: number | null,
  tags: string
) => invoke<NoteTemplate>('create_template', { name, title, content, categoryId, tags });
export const deleteTemplate = (id: number) => invoke<void>('delete_template', { id });

// Saved searches
export interface SavedSearch {
  id: number;
  name: string;
  query: string;
  categoryId: number | null;
  tagName: string | null;
  timeRange: string | null;
}

export const listSavedSearches = () => invoke<SavedSearch[]>('list_saved_searches');
export const createSavedSearch = (
  name: string,
  query: string,
  categoryId: number | null,
  tagName: string | null,
  timeRange: string | null
) =>
  invoke<SavedSearch>('create_saved_search', { name, query, categoryId, tagName, timeRange });
export const deleteSavedSearch = (id: number) => invoke<void>('delete_saved_search', { id });

// Statistics
export interface CategoryCount {
  name: string;
  count: number;
  color: string;
}
export interface NoteStatistics {
  totalNotes: number;
  archivedNotes: number;
  notesWithReminders: number;
  totalContacts: number;
  totalCoworkers: number;
  notesThisWeek: number;
  notesPerCategory: CategoryCount[];
}

export const getStatistics = () => invoke<NoteStatistics>('get_statistics');

// Quick capture
export const quickCapture = (text: string, categoryId: number | null = null) =>
  invoke<Note>('quick_capture', { text, categoryId });

// Single note export
export const exportNoteToFile = (id: number, path: string, format: string) =>
  invoke<void>('export_note_to_file', { id, path, format });

// Notes CSV/JSON export
export const exportNotesToFile = (path: string, format: string) =>
  invoke<void>('export_notes_to_file', { path, format });

// vCard export
export const exportContactsVcard = (path: string) =>
  invoke<void>('export_contacts_vcard', { path });

// Database backup/restore
export const backupDatabase = (path: string) => invoke<void>('backup_database', { path });
export const restoreDatabase = (path: string) => invoke<void>('restore_database', { path });
