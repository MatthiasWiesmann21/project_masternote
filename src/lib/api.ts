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
export const setReminder = (noteId: number, dueAt: string, createCalendarEvent: boolean) =>
  invoke<Reminder>('set_reminder', { noteId, dueAt, createCalendarEvent });

export const deleteReminder = (noteId: number) => invoke<void>('delete_reminder', { noteId });

// Microsoft Graph commands
export const graphSignIn = () => invoke<boolean>('graph_sign_in');

export const graphSignOut = () => invoke<void>('graph_sign_out');

export const graphIsSignedIn = () => invoke<boolean>('graph_is_signed_in');

// Widget commands
export const hideWidget = () => invoke<void>('hide_widget');

// Hotkey commands
export interface HotkeyConfig {
  open: string;
  saveClose: string;
}

export const getHotkeys = () => invoke<HotkeyConfig>('get_hotkeys');

export const setOpenHotkey = (hotkey: string) =>
  invoke<void>('set_open_hotkey', { hotkey });

export const setSaveCloseHotkey = (hotkey: string) =>
  invoke<void>('set_save_close_hotkey', { hotkey });

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
