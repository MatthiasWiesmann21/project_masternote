<script lang="ts">
  import * as api from '$lib/api';
  import { settings } from '$lib/stores/settings';
  import { loadNotes } from '$lib/stores/notes';
  import Modal from '$lib/components/ui/Modal.svelte';
  import { Bell } from '@lucide/svelte';
  import { t } from '$lib/i18n';
  import { get } from 'svelte/store';

  let { noteId, onClose } = $props<{ noteId: number; onClose: () => void }>();

  let dueAt = $state('');
  let createCalendarEvent = $state(false);
  let saving = $state(false);
  let error = $state('');
  let recurInterval = $state<number | null>(null);
  let recurUnit = $state<string | null>(null);

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
      await api.setReminder(
        noteId,
        new Date(dueAt).toISOString(),
        createCalendarEvent,
        recurInterval,
        recurUnit
      );
      await loadNotes();
      onClose();
    } catch (e: any) {
      error = e?.message ?? get(t)('reminder.setFailed');
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
      error = e?.message ?? get(t)('reminder.deleteFailed');
    }
  }
</script>

<Modal onClose={onClose} width="w-80" title={$t('reminder.title')}>
  {#snippet icon()}<Bell class="w-4 h-4" />{/snippet}

  <div class="p-4 space-y-3">
    <label class="block">
      <span class="text-xs text-fg-muted mb-1 block font-medium">{$t('reminder.when')}</span>
      <input
        type="datetime-local"
        bind:value={dueAt}
        class="w-full text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20"
      />
    </label>

    <div>
      <span class="text-xs text-fg-muted mb-1 block font-medium">{$t('reminder.repeat')}</span>
      <div class="flex items-center gap-2">
        <select
          bind:value={recurUnit}
          class="flex-1 text-xs bg-bg-muted rounded-lg px-2 py-1.5 border border-border outline-none focus:border-accent"
        >
          <option value={null}>{$t('reminder.noRepeat')}</option>
          <option value="minutes">{$t('reminder.minutes')}</option>
          <option value="hours">{$t('reminder.hours')}</option>
          <option value="days">{$t('reminder.days')}</option>
          <option value="weeks">{$t('reminder.weeks')}</option>
          <option value="months">{$t('reminder.months')}</option>
        </select>
        {#if recurUnit}
          <input
            type="number"
            min="1"
            max="999"
            bind:value={recurInterval}
            placeholder={$t('reminder.every')}
            class="w-16 text-xs bg-bg-muted rounded-lg px-2 py-1.5 border border-border outline-none focus:border-accent"
          />
        {/if}
      </div>
    </div>

    {#if $settings.graphSignedIn}
      <label class="flex items-center gap-2 text-xs cursor-pointer">
        <input type="checkbox" bind:checked={createCalendarEvent} class="accent-accent w-3.5 h-3.5" />
        {$t('reminder.calendarEvent')}
      </label>
    {/if}

    {#if error}
      <p class="text-xs text-danger">{error}</p>
    {/if}
  </div>

  {#snippet footer()}
    <button onclick={handleDelete} class="text-xs px-3 py-1.5 rounded-lg text-danger hover:bg-danger-soft transition mr-auto">{$t('reminder.remove')}</button>
    <button onclick={onClose} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
    <button
      onclick={handleSet}
      disabled={saving}
      class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition disabled:opacity-50"
    >
      {saving ? $t('reminder.saving') : $t('reminder.set')}
    </button>
  {/snippet}
</Modal>
