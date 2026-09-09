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

// Note commands
export const createNote = (title: string, content: string, categoryId: number | null) =>
  invoke<Note>('create_note', { title, content, categoryId });

export const updateNote = (id: number, title: string, content: string, categoryId: number | null) =>
  invoke<Note>('update_note', { id, title, content, categoryId });

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
