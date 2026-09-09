<script lang="ts">
  import type { Note } from '$lib/api';
  import * as api from '$lib/api';
  import { categories, saveNote, selectedNoteId, removeNote } from '$lib/stores/notes';
  import TagPicker from './TagPicker.svelte';
  import ReminderDialog from './ReminderDialog.svelte';

  let note = $state<Note | null>(null);
  let title = $state('');
  let content = $state('');
  let categoryId = $state<number | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let showReminder = $state(false);
  let saveStatus = $state('');
  let errorMsg = $state('');

  export function loadNote(n: Note | null) {
    note = n;
    title = n?.title ?? '';
    content = n?.content ?? '';
    categoryId = n?.categoryId ?? null;
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
    saveTimer = setTimeout(doSave, 800);
  }

  async function doSave() {
    if (!title.trim() && !content.trim()) return;
    errorMsg = '';
    const saved = await saveNote(note?.id ?? null, title, content, categoryId);
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

  async function handleDelete() {
    if (note) {
      await removeNote(note.id);
      newNote();
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      handleSaveNow();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      handleSaveNow();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
      e.preventDefault();
      newNote();
    }
  }

  let textareaEl = $state<HTMLTextAreaElement | null>(null);

  export function focus() {
    textareaEl?.focus();
  }
</script>

<svelte:window on:keydown={onKeydown} />

<div class="flex flex-col h-full">
  <div class="flex items-center gap-2 px-3 py-2 border-b border-border bg-bg-subtle">
    <input
      bind:value={title}
      oninput={scheduleAutosave}
      placeholder="Note title…"
      class="flex-1 bg-transparent text-sm font-medium outline-none placeholder:text-fg-muted"
    />
    <select
      bind:value={categoryId}
      onchange={scheduleAutosave}
      class="text-xs bg-bg-muted rounded px-2 py-1 border border-border outline-none cursor-pointer"
    >
      <option value={null}>No category</option>
      {#each $categories as cat}
        <option value={cat.id}>{cat.name}</option>
      {/each}
    </select>
  </div>

  <textarea
    bind:this={textareaEl}
    bind:value={content}
    oninput={scheduleAutosave}
    placeholder="Write what you hear…  (Ctrl+S to save, Ctrl+N for new)"
    class="flex-1 w-full resize-none bg-transparent p-3 text-sm leading-relaxed outline-none placeholder:text-fg-muted font-mono"
  ></textarea>

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
      <button
        onclick={handleDelete}
        class="text-xs px-3 py-1.5 rounded text-red-500 hover:bg-red-500/10 transition ml-auto"
      >
        Delete
      </button>
    {/if}
    {#if saveStatus}
      <span class="text-[10px] text-green-500 ml-auto">{saveStatus}</span>
    {/if}
  </div>
</div>

{#if showReminder && note}
  <ReminderDialog noteId={note.id} onClose={() => (showReminder = false)} />
{/if}
