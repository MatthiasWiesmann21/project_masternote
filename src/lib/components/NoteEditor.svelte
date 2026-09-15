<script lang="ts">
  import type { Note, Coworker } from '$lib/api';
  import * as api from '$lib/api';
  import { categories, saveNote, selectedNoteId, removeNote, notes, filteredNotes, loadNotes } from '$lib/stores/notes';
  import { settings } from '$lib/stores/settings';
  import { showToast } from '$lib/stores/toast';
  import { t } from '$lib/i18n';
  import { get } from 'svelte/store';
  import { save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { marked } from 'marked';
  import TagPicker from './TagPicker.svelte';
  import ReminderDialog from './ReminderDialog.svelte';
  import ContactPicker from './ContactPicker.svelte';
  import CoworkerPicker from './CoworkerPicker.svelte';
  import CategoryPicker from './CategoryPicker.svelte';
  import Modal from '$lib/components/ui/Modal.svelte';
  import {
    Save, FilePlus, Bell, Mail, Calendar, Eye, Pencil, Download,
    Archive, ArchiveRestore, Copy, Trash2, AlertCircle, Pin, PinOff
  } from '@lucide/svelte';

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
  let showUnarchiveConfirm = $state(false);
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
    // Reset all dialog states to prevent stale modals
    showDeleteConfirm = false;
    showArchiveConfirm = false;
    showUnarchiveConfirm = false;
    showCopyConfirm = false;
    showRapportDialog = false;
    showCalendarDialog = false;
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
    // Reset all dialog states to prevent stale modals
    showDeleteConfirm = false;
    showArchiveConfirm = false;
    showUnarchiveConfirm = false;
    showCopyConfirm = false;
    showRapportDialog = false;
    showCalendarDialog = false;
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
      saveStatus = get(t)('editor.saved');
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
      showDeleteConfirm = false;
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
      copyStatus = get(t)('editor.copied');
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
      await loadNotes();
    } catch (e: any) {
      errorMsg = e?.message ?? String(e);
    }
  }

  async function handleUnarchive() {
    if (!note) return;
    try {
      await api.unarchiveNote(note.id);
      showUnarchiveConfirm = false;
      await loadNotes();
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
        saveStatus = get(t)('editor.exported');
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
    graphStatus = get(t)('editor.openingOutlook');
    try {
      await api.openTelephoneRapport(note.id, rapportCoworker.email);
      graphStatus = get(t)('editor.outlookOpened');
      showRapportDialog = false;
      rapportCoworker = null;
      rapportCoworkerQuery = '';
      setTimeout(() => (graphStatus = ''), 3000);
    } catch (e: any) {
      graphStatus = '';
      errorMsg = e?.message ?? String(e);
    }
  }

  async function handleRapportClick() {
    if (!note) return;
    // No coworker linked at all → show the picker
    if (!note.coworker?.email && !coworkerId) {
      rapportCoworker = null;
      rapportCoworkerQuery = '';
      rapportCoworkerResults = [];
      showRapportDialog = true;
      return;
    }
    // Resolve the email: enriched note coworker, or the just-selected coworkerId
    let email = note.coworker?.email ?? null;
    if (!email && coworkerId) {
      try {
        email = (await api.listCoworkers()).find((c) => c.id === coworkerId)?.email ?? null;
      } catch { /* fall through to backend resolution */ }
    }
    // Open Outlook directly — backend falls back to the linked coworker's email
    graphStatus = get(t)('editor.openingOutlook');
    try {
      await api.openTelephoneRapport(note.id, email);
      graphStatus = get(t)('editor.outlookOpened');
      setTimeout(() => (graphStatus = ''), 3000);
    } catch (e: any) {
      graphStatus = '';
      // Coworker has no email — fall back to the picker dialog
      rapportCoworker = null;
      rapportCoworkerQuery = '';
      rapportCoworkerResults = [];
      showRapportDialog = true;
    }
  }

  async function handleCreateCalendar() {
    if (!note) return;
    graphStatus = get(t)('editor.creatingEvent');
    const iso = `${calendarDate}T${calendarTime}:00`;
    try {
      await api.createCalendarWithContact(note.id, iso);
      graphStatus = get(t)('editor.calendarCreated');
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
  <div class="flex items-center gap-2 px-3 py-2.5 border-b border-border bg-bg-subtle">
    <input
      bind:this={titleEl}
      bind:value={title}
      oninput={scheduleAutosave}
      placeholder={$t('editor.title')}
      class="flex-1 bg-transparent text-base font-semibold outline-none placeholder:text-fg-muted"
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
      placeholder={$t('editor.placeholder')}
      class="flex-1 w-full resize-none bg-transparent p-3 text-sm leading-relaxed outline-none placeholder:text-fg-muted font-mono"
    ></textarea>
  {/if}

  {#if note}
    <div class="px-3 py-1.5 border-t border-border bg-bg-subtle">
      <TagPicker noteId={note.id} tags={note.tags} />
    </div>
  {/if}

  {#if errorMsg}
    <div class="px-3 py-1.5 text-xs text-danger bg-danger-soft border-t border-danger/20 flex items-center gap-1.5">
      <AlertCircle class="w-3.5 h-3.5 shrink-0" />
      {errorMsg}
    </div>
  {/if}

  <!-- Toolbar -->
  <div class="flex items-center gap-1 px-3 py-2 border-t border-border bg-bg-subtle">
    <button
      onclick={handleSaveNow}
      class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition inline-flex items-center gap-1"
    >
      <Save class="w-3.5 h-3.5" />
      {$t('editor.save')}
    </button>
    <button
      onclick={newNote}
      class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1"
    >
      <FilePlus class="w-3.5 h-3.5" />
      {$t('editor.new')}
    </button>
    {#if note}
      <button
        onclick={() => (showReminder = true)}
        class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1"
        title={$t('editor.reminderTitle')}
      >
        <Bell class="w-3.5 h-3.5" />
        {$t('editor.reminder')}
      </button>
      {#if contactId || coworkerId}
        <button
          onclick={handleRapportClick}
          class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1"
          title={$t('editor.rapportTitle')}
        >
          <Mail class="w-3.5 h-3.5" />
          {$t('editor.rapport')}
        </button>
        {#if contactId}
          <button
            onclick={() => {
              const now = new Date();
              calendarDate = now.toISOString().slice(0, 10);
              calendarTime = `${String(now.getHours()).padStart(2, '0')}:00`;
              showCalendarDialog = true;
            }}
            class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1"
            title={$t('editor.calendarTitle')}
          >
            <Calendar class="w-3.5 h-3.5" />
            {$t('editor.calendar')}
          </button>
        {/if}
      {/if}
      <button
        onclick={() => (showPreview = !showPreview)}
        class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1"
        title={$t('editor.previewTitle')}
      >
        {#if showPreview}
          <Pencil class="w-3.5 h-3.5" />
          {$t('editor.edit')}
        {:else}
          <Eye class="w-3.5 h-3.5" />
          {$t('editor.preview')}
        {/if}
      </button>
      <button
        onclick={handleExportNote}
        class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1"
        title={$t('editor.exportTitle')}
      >
        <Download class="w-3.5 h-3.5" />
        {$t('editor.export')}
      </button>
      {#if note.pinned}
        <button
          onclick={async () => { try { await api.unpinNote(note!.id); await loadNotes(); } catch (e) { console.error('Unpin failed:', e); } }}
          class="text-xs px-3 py-1.5 rounded-lg bg-accent-soft text-accent hover:bg-accent/20 transition inline-flex items-center gap-1"
          title={$t('editor.unpinTitle')}
        >
          <PinOff class="w-3.5 h-3.5" />
          {$t('editor.unpin')}
        </button>
      {:else}
        <button
          onclick={async () => { try { await api.pinNote(note!.id); await loadNotes(); } catch (e) { console.error('Pin failed:', e); } }}
          class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1"
          title={$t('editor.pinTitle')}
        >
          <Pin class="w-3.5 h-3.5" />
          {$t('editor.pin')}
        </button>
      {/if}
      {#if note.archived}
        <button
          onclick={() => (showUnarchiveConfirm = true)}
          class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1"
          title={$t('editor.unarchiveTitle')}
        >
          <ArchiveRestore class="w-3.5 h-3.5" />
          {$t('editor.unarchive')}
        </button>
      {:else}
        <button
          onclick={() => (showArchiveConfirm = true)}
          class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1"
          title={$t('editor.archiveTitle')}
        >
          <Archive class="w-3.5 h-3.5" />
          {$t('editor.archive')}
        </button>
      {/if}
      <button
        onclick={() => (showCopyConfirm = true)}
        class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition ml-auto inline-flex items-center gap-1"
        title={$t('editor.copyTitle')}
      >
        <Copy class="w-3.5 h-3.5" />
        {$t('editor.copy')}
      </button>
      <button
        onclick={() => (showDeleteConfirm = true)}
        class="text-xs px-3 py-1.5 rounded-lg text-danger hover:bg-danger-soft transition inline-flex items-center gap-1"
      >
        <Trash2 class="w-3.5 h-3.5" />
        {$t('editor.delete')}
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
  <Modal onClose={() => (showRapportDialog = false)} width="w-[400px]" title={$t('rapport.title')}>
    {#snippet icon()}<Mail class="w-4 h-4" />{/snippet}
    <div class="p-4 space-y-3">
      {#if rapportCoworker}
        <div class="flex items-center gap-1.5 text-xs">
          <span class="px-1.5 py-0.5 rounded-full bg-green-500/15 text-green-600 whitespace-nowrap">
            {[rapportCoworker.firstName, rapportCoworker.lastName].filter(Boolean).join(' ')}
          </span>
          <span class="text-fg-muted">{rapportCoworker.email}</span>
          <button
            onclick={() => { rapportCoworker = null; }}
            class="text-fg-muted hover:text-red-500 text-[10px]"
          >{$t('rapport.change')}</button>
        </div>
      {:else}
        <div class="relative">
          <input
            bind:value={rapportCoworkerQuery}
            placeholder={$t('rapport.search')}
            class="text-xs bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none w-full focus:border-accent focus:ring-2 focus:ring-accent/20"
          />
          {#if rapportCoworkerResults.length > 0}
            <div class="absolute top-full left-0 right-0 mt-1 bg-surface-1 rounded-xl border border-border shadow-lg max-h-48 overflow-y-auto z-50">
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
        {$t('rapport.desc')}
      </p>

      {#if errorMsg}
        <div class="text-xs text-danger">{errorMsg}</div>
      {/if}
    </div>

    {#snippet footer()}
      <button onclick={() => (showRapportDialog = false)} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
      <button
        onclick={handleOpenRapport}
        disabled={!rapportCoworker?.email}
        class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition disabled:opacity-40 disabled:cursor-not-allowed"
      >
        {$t('rapport.open')}
      </button>
    {/snippet}
  </Modal>
{/if}

{#if showCalendarDialog && note}
  <Modal onClose={() => (showCalendarDialog = false)} width="w-[400px]" title={$t('calendar.title')}>
    {#snippet icon()}<Calendar class="w-4 h-4" />{/snippet}
    <div class="p-4 space-y-3">
      <p class="text-[10px] text-fg-muted">
        {$t('calendar.desc')}
      </p>
      <div class="flex gap-2">
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('calendar.date')}</span>
          <input
            bind:value={calendarDate}
            type="date"
            class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20"
          />
        </label>
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('calendar.time')}</span>
          <input
            bind:value={calendarTime}
            type="time"
            class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20"
          />
        </label>
      </div>
      {#if errorMsg}
        <div class="text-xs text-danger">{errorMsg}</div>
      {/if}
    </div>
    {#snippet footer()}
      <button onclick={() => (showCalendarDialog = false)} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
      <button onclick={handleCreateCalendar} class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition">{$t('calendar.create')}</button>
    {/snippet}
  </Modal>
{/if}

{#if showReminder && note}
  <ReminderDialog noteId={note.id} onClose={() => (showReminder = false)} />
{/if}

{#if showCopyConfirm && note}
  <Modal onClose={() => (showCopyConfirm = false)} width="w-[360px]" title={$t('copy.title')}>
    {#snippet icon()}<Copy class="w-4 h-4" />{/snippet}
    <div class="p-4">
      <p class="text-xs text-fg-muted">
        {$t('copy.text', { title: note.title || $t('notes.untitled') })}
      </p>
    </div>
    {#snippet footer()}
      <button onclick={() => (showCopyConfirm = false)} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
      <button onclick={handleCopy} class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition">{$t('copy.confirm')}</button>
    {/snippet}
  </Modal>
{/if}

{#if showDeleteConfirm && note}
  <Modal onClose={() => (showDeleteConfirm = false)} width="w-[360px]" title={$t('delete.title')}>
    {#snippet icon()}<Trash2 class="w-4 h-4 text-danger" />{/snippet}
    <div class="p-4">
      <p class="text-xs text-fg-muted">
        {$t('delete.text', { title: note.title || $t('notes.untitled') })}
      </p>
    </div>
    {#snippet footer()}
      <button onclick={() => (showDeleteConfirm = false)} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
      <button onclick={handleDelete} class="text-xs px-3 py-1.5 rounded-lg bg-danger text-white font-medium hover:opacity-90 transition">{$t('common.delete')}</button>
    {/snippet}
  </Modal>
{/if}

{#if showArchiveConfirm && note}
  <Modal onClose={() => (showArchiveConfirm = false)} width="w-[360px]" title={$t('archive.title')}>
    {#snippet icon()}<Archive class="w-4 h-4" />{/snippet}
    <div class="p-4">
      <p class="text-xs text-fg-muted">
        {$t('archive.text', { title: note.title || $t('notes.untitled') })}
      </p>
    </div>
    {#snippet footer()}
      <button onclick={() => (showArchiveConfirm = false)} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
      <button onclick={handleArchive} class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition">{$t('archive.confirm')}</button>
    {/snippet}
  </Modal>
{/if}

{#if showUnarchiveConfirm && note}
  <Modal onClose={() => (showUnarchiveConfirm = false)} width="w-[360px]" title={$t('unarchive.title')}>
    {#snippet icon()}<ArchiveRestore class="w-4 h-4" />{/snippet}
    <div class="p-4">
      <p class="text-xs text-fg-muted">
        {$t('unarchive.text', { title: note.title || $t('notes.untitled') })}
      </p>
    </div>
    {#snippet footer()}
      <button onclick={() => (showUnarchiveConfirm = false)} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
      <button onclick={handleUnarchive} class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition">{$t('unarchive.confirm')}</button>
    {/snippet}
  </Modal>
{/if}