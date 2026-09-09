<script lang="ts">
  import * as api from '$lib/api';
  import { settings } from '$lib/stores/settings';
  import { loadNotes } from '$lib/stores/notes';

  let { noteId, onClose } = $props<{ noteId: number; onClose: () => void }>();

  let dueAt = $state('');
  let createCalendarEvent = $state(false);
  let saving = $state(false);
  let error = $state('');

  function defaultDateTime(): string {
    const d = new Date(Date.now() + 60 * 60 * 1000);
    d.setMinutes(0, 0, 0);
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }

  dueAt = defaultDateTime();

  async function handleSet() {
    if (!dueAt) return;
    saving = true;
    error = '';
    try {
      await api.setReminder(noteId, new Date(dueAt).toISOString(), createCalendarEvent);
      await loadNotes();
      onClose();
    } catch (e: any) {
      error = e?.message ?? 'Failed to set reminder';
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    try {
      await api.deleteReminder(noteId);
      await loadNotes();
      onClose();
    } catch (e: any) {
      error = e?.message ?? 'Failed to delete reminder';
    }
  }
</script>

<svelte:window on:keydown={(e) => { if (e.key === 'Escape') onClose(); }} />

<div
  class="fixed inset-0 bg-black/30 flex items-center justify-center z-50"
  role="button"
  tabindex="-1"
  onclick={onClose}
  onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}
>
  <div
    class="bg-bg rounded-lg shadow-xl border border-border p-4 w-80"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <h3 class="text-sm font-semibold mb-3">Set Reminder</h3>

    <label class="block text-xs text-fg-muted mb-1" for="reminder-due-at">When</label>
    <input
      id="reminder-due-at"
      type="datetime-local"
      bind:value={dueAt}
      class="w-full text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none mb-3"
    />

    {#if $settings.graphSignedIn}
      <label class="flex items-center gap-2 text-xs mb-3 cursor-pointer">
        <input type="checkbox" bind:checked={createCalendarEvent} class="accent-accent" />
        Create Outlook calendar event
      </label>
    {/if}

    {#if error}
      <p class="text-xs text-red-500 mb-2">{error}</p>
    {/if}

    <div class="flex gap-2 justify-end">
      <button onclick={handleDelete} class="text-xs px-3 py-1.5 rounded text-red-500 hover:bg-red-500/10">
        Remove
      </button>
      <button onclick={onClose} class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border">
        Cancel
      </button>
      <button
        onclick={handleSet}
        disabled={saving}
        class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90 disabled:opacity-50"
      >
        {saving ? 'Saving…' : 'Set'}
      </button>
    </div>
  </div>
</div>
