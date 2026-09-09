import { writable } from 'svelte/store';
import type { Note, Tag, Category } from '../api';
import * as api from '../api';

export const notes = writable<Note[]>([]);
export const tags = writable<Tag[]>([]);
export const categories = writable<Category[]>([]);
export const selectedNoteId = writable<number | null>(null);
export const searchQuery = writable('');
export const activeTagFilter = writable<string | null>(null);
export const activeCategoryFilter = writable<number | null>(null);
export const isLoading = writable(false);
export const lastError = writable<string | null>(null);

export async function loadNotes() {
  isLoading.set(true);
  try {
    const result = await api.listNotes(200, 0);
    notes.set(result);
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    lastError.set(`loadNotes: ${msg}`);
    console.error('Failed to load notes:', e);
  } finally {
    isLoading.set(false);
  }
}

export async function loadTags() {
  try {
    tags.set(await api.listTags());
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    lastError.set(`loadTags: ${msg}`);
    console.error('Failed to load tags:', e);
  }
}

export async function loadCategories() {
  try {
    categories.set(await api.listCategories());
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    lastError.set(`loadCategories: ${msg}`);
    console.error('Failed to load categories:', e);
  }
}

export async function refreshAll() {
  await Promise.all([loadNotes(), loadTags(), loadCategories()]);
}

export async function saveNote(
  id: number | null,
  title: string,
  content: string,
  categoryId: number | null
): Promise<Note | null> {
  try {
    let saved: Note;
    if (id === null) {
      saved = await api.createNote(title, content, categoryId);
    } else {
      saved = await api.updateNote(id, title, content, categoryId);
    }
    await loadNotes();
    return saved;
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    lastError.set(`saveNote: ${msg}`);
    console.error('Failed to save note:', e);
    return null;
  }
}

export async function removeNote(id: number) {
  try {
    await api.deleteNote(id);
    selectedNoteId.set(null);
    await loadNotes();
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    lastError.set(`removeNote: ${msg}`);
    console.error('Failed to delete note:', e);
  }
}

export async function addCategory(name: string, color: string): Promise<boolean> {
  try {
    await api.createCategory(name, color);
    await loadCategories();
    return true;
  } catch (e: any) {
    const msg = e?.message ?? String(e);
    lastError.set(`addCategory: ${msg}`);
    console.error('Failed to create category:', e);
    return false;
  }
}
