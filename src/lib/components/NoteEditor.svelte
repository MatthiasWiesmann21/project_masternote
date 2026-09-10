<script lang="ts">
  import type { Note, Coworker } from '$lib/api';
  import * as api from '$lib/api';
  import { categories, saveNote, selectedNoteId, removeNote, notes, filteredNotes } from '$lib/stores/notes';
  import { settings } from '$lib/stores/settings';
  import { save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { marked } from 'marked';
  import TagPicker from './TagPicker.svelte';
  import ReminderDialog from './ReminderDialog.svelte';
  import ContactPicker from './ContactPicker.svelte';
  import CoworkerPicker from './CoworkerPicker.svelte';
  import CategoryPicker from './CategoryPicker.svelte';

  let note = $state<Note | null>(null);
  let title = $state('');
  let content = $state('');
  let categoryId = $state<number | null>(null);
  let contactId = $state<number | null>(null);
  let coworkerId = $state<number | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let showReminder = $state(false);
  let saveStatus = $state('');
  let errorMsg = $state('');
  let showRapportDialog = $state(false);
  let showCalendarDialog = $state(false);
  let showDeleteConfirm = $state(false);
  let showCopyConfirm = $state(false);
  let showArchiveConfirm = $state(false);
  let copyStatus = $state('');
  let rapportCoworkerQuery = $state('');
  let rapportCoworkerResults = $state<Coworker[]>([]);
  let rapportCoworker = $state<Coworker | null>(null);
  let calendarDate = $state('');
  let calendarTime = $state('12:00');
  let graphStatus = $state('');
  let showPreview = $state(false);

  // Element refs for keyboard shortcuts
  let titleEl = $state<HTMLInputElement | null>(null);
  let textareaEl = $state<HTMLTextAreaElement | null>(null);
  let contactPickerEl = $state<any>(null);
  let coworkerPickerEl = $state<any>(null);
  let categoryPickerEl = $state<any>(null);

  export function loadNote(n: Note | null) {
    note = n;
    title = n?.title ?? '';
    content = n?.content ?? '';
    categoryId = n?.categoryId ?? null;
    contactId = n?.contactId ?? null;
    coworkerId = n?.coworkerId ?? null;
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    errorMsg = '';
  }

  export function newNote() {
    note = null;
    title = '';
    content = '';
    categoryId = null;
    contactId = null;
    coworkerId = null;
    selectedNoteId.set(null);
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    errorMsg = '';
  }

  function scheduleAutosave() {
    errorMsg = '';
    if (saveTimer) clearTimeout(saveTimer);
    const interval = $settings.autosaveInterval;
    if (interval === 0) {
      doSave();
    } else {
      saveTimer = setTimeout(doSave, interval);
    }
  }

  async function doSave() {
    if (!title.trim() && !content.trim()) return;
    errorMsg = '';
    const saved = await saveNote(note?.id ?? null, title, content, categoryId, contactId, coworkerId);
    if (saved && !note) {
      note = saved;
      selectedNoteId.set(saved.id);
    }
    if (saved) {
      saveStatus = 'Saved';
      setTimeout(() => (saveStatus = ''), 1500);
    } else {
      errorMsg = 'Save failed — check console for details';
    }
  }

  async function handleSaveNow() {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    await doSave();
  }

  export async function saveNow() {
    await handleSaveNow();
  }

  async function handleDelete() {
    if (note) {
      await removeNote(note.id);
      newNote();
    }
  }

  async function handleCopy() {
    if (!note) return;
    try {
      const copied = await api.createNote(
        note.title ? `${note.title} (copy)` : '(copy)',
        note.content,
        note.categoryId,
        note.contactId,
        note.coworkerId
      );
      copyStatus = 'Copied';
      setTimeout(() => (copyStatus = ''), 2000);
      showCopyConfirm = false;
      selectedNoteId.set(copied.id);
      loadNote(copied);
    } catch (e: any) {
      errorMsg = e?.message ?? String(e);
    }
  }

  async function handleArchive() {
    if (!note) return;
    try {
      await api.archiveNote(note.id);
      showArchiveConfirm = false;
      newNote();
    } catch (e: any) {
      errorMsg = e?.message ?? String(e);
    }
  }

  async function handleExportNote() {
    if (!note) return;
    try {
      const path = await saveDialog({
        defaultPath: `${note.title || 'note'}.md`,
        filters: [
          { name: 'Markdown', extensions: ['md'] },
          { name: 'Text', extensions: ['txt'] }
        ]
      });
      if (path) {
        const format = path.endsWith('.txt') ? 'txt' : 'md';
        await api.exportNoteToFile(note.id, path, format);
        saveStatus = 'Exported';
        setTimeout(() => (saveStatus = ''), 2000);
      }
    } catch (e: any) {
      errorMsg = e?.message ?? String(e);
    }
  }

  function renderPreview(text: string): string {
    try {
      return marked.parse(text, { breaks: true, async: false }) as string;
    } catch {
      return text;
    }
  }

  async function searchRapportCoworkers() {
    if (rapportCoworkerQuery.trim().length < 1) {
      rapportCoworkerResults = [];
      return;
    }
    try {
      rapportCoworkerResults = await api.searchCoworkers(rapportCoworkerQuery.trim());
    } catch {
      rapportCoworkerResults = [];
    }
  }

  $effect(() => {
    rapportCoworkerQuery;
    const timer = setTimeout(searchRapportCoworkers, 150);
    return () => clearTimeout(timer);
  });

  function pickRapportCoworker(c: Coworker) {
    rapportCoworker = c;
    rapportCoworkerQuery = '';
    rapportCoworkerResults = [];
  }

  async function handleOpenRapport() {
    if (!note || !rapportCoworker?.email) return;
    graphStatus = 'Opening Outlook…';
    try {
      await api.openTelephoneRapport(note.id, rapportCoworker.email);
      graphStatus = 'Outlook opened';
      showRapportDialog = false;
      rapportCoworker = null;
      rapportCoworkerQuery = '';
      setTimeout(() => (graphStatus = ''), 3000);
    } catch (e: any) {
      graphStatus = '';
      errorMsg = e?.message ?? String(e);
    }
  }

  async function handleCreateCalendar() {
    if (!note) return;
    graphStatus = 'Creating event…';
    const iso = `${calendarDate}T${calendarTime}:00`;
    try {
      await api.createCalendarWithContact(note.id, iso);
      graphStatus = 'Calendar entry created (1h)';
      showCalendarDialog = false;
      setTimeout(() => (graphStatus = ''), 3000);
    } catch (e: any) {
      graphStatus = '';
      errorMsg = e?.message ?? String(e);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.shiftKey && e.key.toLowerCase() === 't') {
      e.preventDefault();
      titleEl?.focus();
      titleEl?.select();
    }
    if (mod && e.shiftKey && e.key.toLowerCase() === 'd') {
      e.preventDefault();
      textareaEl?.focus();
    }
    if (mod && e.shiftKey && e.key.toLowerCase() === 'c') {
      e.preventDefault();
      categoryPickerEl?.focusSearch();
    }
    if (mod && e.shiftKey && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      contactPickerEl?.focusSearch();
    }
    if (mod && e.shiftKey && e.key.toLowerCase() === 'h') {
      e.preventDefault();
      coworkerPickerEl?.focusSearch();
    }
    if (mod && e.key === 's') {
      e.preventDefault();
      handleSaveNow();
    }
    if (mod && e.key === 'Enter') {
      e.preventDefault();
      handleSaveNow();
    }
    if (mod && e.key === 'n') {
      e.preventDefault();
      newNote();
    }
    if (mod && e.shiftKey && e.key === 'Tab') {
      e.preventDefault();
      // Select next note in the filtered list
      const allNotes = $filteredNotes;
      if (allNotes.length === 0) return;
      const currentId = $selectedNoteId;
      const currentIdx = currentId !== null ? allNotes.findIndex((n) => n.id === currentId) : -1;
      const nextIdx = (currentIdx + 1) % allNotes.length;
      const nextNote = allNotes[nextIdx];
      selectedNoteId.set(nextNote.id);
      loadNote(nextNote);
    }
  }

  export function focus() {
    titleEl?.focus();
  }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="flex flex-col h-full">
  <!-- Title row + category -->
  <div class="flex items-center gap-2 px-3 py-2 border-b border-border bg-bg-subtle">
    <input
      bind:this={titleEl}
      bind:value={title}
      oninput={scheduleAutosave}
      placeholder="Note title…"
      class="flex-1 bg-transparent text-sm font-medium outline-none placeholder:text-fg-muted"
    />
    <CategoryPicker
      bind:this={categoryPickerEl}
      selectedId={categoryId}
      onSelect={(id) => { categoryId = id; scheduleAutosave(); }}
    />
  </div>

  <!-- Contact + Coworker pickers -->
  <div class="flex items-center gap-2 px-3 py-1.5 border-b border-border bg-bg-subtle">
    <ContactPicker bind:this={contactPickerEl} selectedId={contactId} onSelect={(id) => { contactId = id; scheduleAutosave(); }} />
    <CoworkerPicker bind:this={coworkerPickerEl} selectedId={coworkerId} onSelect={(id) => { coworkerId = id; scheduleAutosave(); }} />
  </div>

  {#if showPreview}
    <div class="flex-1 w-full overflow-y-auto p-3 text-sm leading-relaxed prose prose-sm dark:prose-invert max-w-none">
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html renderPreview(content)}
    </div>
  {:else}
    <textarea
      bind:this={textareaEl}
      bind:value={content}
      oninput={scheduleAutosave}
      placeholder="Write what you hear…  (Ctrl+S save, Ctrl+N new, Ctrl+Shift+T title, Ctrl+Shift+D desc, Ctrl+Shift+C cat, Ctrl+Shift+K contact, Ctrl+Shift+H coworker)"
      class="flex-1 w-full resize-none bg-transparent p-3 text-sm leading-relaxed outline-none placeholder:text-fg-muted font-mono"
    ></textarea>
  {/if}

  {#if note}
    <div class="px-3 py-1.5 border-t border-border bg-bg-subtle">
      <TagPicker noteId={note.id} tags={note.tags} />
    </div>
  {/if}

  {#if errorMsg}
    <div class="px-3 py-1 text-xs text-red-500 bg-red-500/10 border-t border-red-500/20">
      {errorMsg}
    </div>
  {/if}

  <div class="flex items-center gap-2 px-3 py-2 border-t border-border bg-bg-subtle">
    <button
      onclick={handleSaveNow}
      class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90 transition"
    >
      Save
    </button>
    <button
      onclick={newNote}
      class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border transition"
    >
      New
    </button>
    {#if note}
      <button
        onclick={() => (showReminder = true)}
        class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border transition"
      >
        Reminder
      </button>
      {#if contactId}
        <button
          onclick={() => {
            rapportCoworker = null;
            rapportCoworkerQuery = '';
            rapportCoworkerResults = [];
            showRapportDialog = true;
          }}
          class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border transition"
          title="Open Outlook with telephone rapport"
        >
          📧 Rapport
        </button>
        <button
          onclick={() => {
            const now = new Date();
            calendarDate = now.toISOString().slice(0, 10);
            calendarTime = `${String(now.getHours()).padStart(2, '0')}:00`;
            showCalendarDialog = true;
          }}
          class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border transition"
          title="Create Outlook calendar entry with contact"
        >
          📅 Calendar
        </button>
      {/if}
      <button
        onclick={() => (showPreview = !showPreview)}
        class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border transition"
        title="Toggle Markdown preview"
      >
        {showPreview ? '✏ Edit' : '👁 Preview'}
      </button>
      <button
        onclick={handleExportNote}
        class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border transition"
        title="Export this note to file"
      >
        ↧ Export
      </button>
      <button
        onclick={() => (showArchiveConfirm = true)}
        class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border transition"
        title="Archive this note"
      >
        📦 Archive
      </button>
      <button
        onclick={() => (showCopyConfirm = true)}
        class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border transition ml-auto"
        title="Duplicate this note"
      >
        📋 Copy
      </button>
      <button
        onclick={() => (showDeleteConfirm = true)}
        class="text-xs px-3 py-1.5 rounded text-red-500 hover:bg-red-500/10 transition"
      >
        Delete
      </button>
    {/if}
    {#if saveStatus}
      <span class="text-[10px] text-green-500 ml-auto">{saveStatus}</span>
    {/if}
    {#if graphStatus}
      <span class="text-[10px] text-blue-500 ml-auto">{graphStatus}</span>
    {/if}
  </div>
</div>

{#if showRapportDialog && note}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
    role="button"
    tabindex="-1"
    onclick={() => (showRapportDialog = false)}
    onkeydown={(e) => { if (e.key === 'Escape') showRapportDialog = false; }}
  >
    <div
      class="bg-bg rounded-lg shadow-2xl border border-border w-[400px] p-4 space-y-3"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="text-sm font-semibold">📧 Telephone Rapport</h3>

      {#if rapportCoworker}
        <div class="flex items-center gap-1.5 text-xs">
          <span class="px-1.5 py-0.5 rounded-full bg-green-500/15 text-green-600 whitespace-nowrap">
            🤝 {[rapportCoworker.firstName, rapportCoworker.lastName].filter(Boolean).join(' ')}
          </span>
          <span class="text-fg-muted">{rapportCoworker.email}</span>
          <button
            onclick={() => { rapportCoworker = null; }}
            class="text-fg-muted hover:text-red-500 text-[10px]"
          >✕ change</button>
        </div>
      {:else}
        <div class="relative">
          <input
            bind:value={rapportCoworkerQuery}
            placeholder="Search coworker to send to…"
            class="text-xs bg-bg-muted rounded px-2 py-1.5 border border-border outline-none w-full"
          />
          {#if rapportCoworkerResults.length > 0}
            <div class="absolute top-full left-0 right-0 mt-1 bg-bg rounded-lg border border-border shadow-lg max-h-48 overflow-y-auto z-50">
              {#each rapportCoworkerResults as c (c.id)}
                <button
                  onmousedown={() => pickRapportCoworker(c)}
                  class="w-full text-left px-2 py-1.5 hover:bg-bg-subtle text-xs border-b border-border last:border-0"
                >
                  <div class="font-medium">{[c.firstName, c.lastName].filter(Boolean).join(' ')}</div>
                  {#if c.email}
                    <div class="text-[10px] text-fg-muted">{c.email}</div>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <p class="text-[10px] text-fg-muted">
        Opens Outlook with a pre-filled email containing the contact's phone/mobile, name, customer ID, and your note. You can review and send manually.
      </p>

      {#if errorMsg}
        <div class="text-xs text-red-500">{errorMsg}</div>
      {/if}

      <div class="flex gap-2 justify-end">
        <button onclick={() => (showRapportDialog = false)} class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border">Cancel</button>
        <button
          onclick={handleOpenRapport}
          disabled={!rapportCoworker?.email}
          class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90 disabled:opacity-40 disabled:cursor-not-allowed"
        >
          Open Outlook
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showCalendarDialog && note}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={() => (showCalendarDialog = false)}
    onkeydown={(e) => { if (e.key === 'Escape') showCalendarDialog = false; }}
  >
    <div
      class="bg-bg rounded-lg shadow-2xl border border-border w-[400px] p-4 space-y-3"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="text-sm font-semibold">📅 Create Calendar Entry</h3>
      <p class="text-[10px] text-fg-muted">
        Title: contact name + customer ID. Description: your note. Duration: 1 hour.
      </p>
      <div class="flex gap-2">
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted">Date</span>
          <input
            bind:value={calendarDate}
            type="date"
            class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none"
          />
        </label>
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted">Time</span>
          <input
            bind:value={calendarTime}
            type="time"
            class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none"
          />
        </label>
      </div>
      {#if errorMsg}
        <div class="text-xs text-red-500">{errorMsg}</div>
      {/if}
      <div class="flex gap-2 justify-end">
        <button onclick={() => (showCalendarDialog = false)} class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border">Cancel</button>
        <button onclick={handleCreateCalendar} class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90">Create</button>
      </div>
    </div>
  </div>
{/if}

{#if showReminder && note}
  <ReminderDialog noteId={note.id} onClose={() => (showReminder = false)} />
{/if}

{#if showCopyConfirm && note}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
    role="button"
    tabindex="-1"
    onclick={() => (showCopyConfirm = false)}
    onkeydown={(e) => { if (e.key === 'Escape') showCopyConfirm = false; }}
  >
    <div
      class="bg-bg rounded-lg shadow-2xl border border-border w-[360px] p-4 space-y-3"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="text-sm font-semibold">📋 Duplicate Note</h3>
      <p class="text-xs text-fg-muted">
        Create a copy of "{note.title || 'Untitled'}" with the same content, category, contact, and coworker?
      </p>
      <div class="flex gap-2 justify-end">
        <button onclick={() => (showCopyConfirm = false)} class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border">Cancel</button>
        <button onclick={handleCopy} class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90">Copy</button>
      </div>
    </div>
  </div>
{/if}

{#if showDeleteConfirm && note}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
    role="button"
    tabindex="-1"
    onclick={() => (showDeleteConfirm = false)}
    onkeydown={(e) => { if (e.key === 'Escape') showDeleteConfirm = false; }}
  >
    <div
      class="bg-bg rounded-lg shadow-2xl border border-border w-[360px] p-4 space-y-3"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="text-sm font-semibold text-red-500">🗑 Delete Note</h3>
      <p class="text-xs text-fg-muted">
        Delete "{note.title || 'Untitled'}"? This cannot be undone.
      </p>
      <div class="flex gap-2 justify-end">
        <button onclick={() => (showDeleteConfirm = false)} class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border">Cancel</button>
        <button onclick={handleDelete} class="text-xs px-3 py-1.5 rounded bg-red-500 text-white font-medium hover:opacity-90">Delete</button>
      </div>
    </div>
  </div>
{/if}

{#if showArchiveConfirm && note}
  <div
    class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
    role="button"
    tabindex="-1"
    onclick={() => (showArchiveConfirm = false)}
    onkeydown={(e) => { if (e.key === 'Escape') showArchiveConfirm = false; }}
  >
    <div
      class="bg-bg rounded-lg shadow-2xl border border-border w-[360px] p-4 space-y-3"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="text-sm font-semibold">📦 Archive Note</h3>
      <p class="text-xs text-fg-muted">
        Archive "{note.title || 'Untitled'}"? Archived notes are hidden by default but can be shown via the filter.
      </p>
      <div class="flex gap-2 justify-end">
        <button onclick={() => (showArchiveConfirm = false)} class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border">Cancel</button>
        <button onclick={handleArchive} class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90">Archive</button>
      </div>
    </div>
  </div>
{/if}
