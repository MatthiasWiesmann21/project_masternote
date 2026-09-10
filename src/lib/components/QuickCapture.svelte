<script lang="ts">
  import * as api from '$lib/api';
  import { categories } from '$lib/stores/notes';

  let { onClose, onSaved } = $props<{ onClose: () => void; onSaved: () => void }>();

  let text = $state('');
  let categoryId = $state<number | null>(null);
  let saving = $state(false);
  let inputEl = $state<HTMLTextAreaElement | null>(null);

  setTimeout(() => inputEl?.focus(), 50);

  async function handleSave() {
    if (!text.trim()) {
      onClose();
      return;
    }
    saving = true;
    try {
      await api.quickCapture(text.trim(), categoryId);
      onSaved();
      onClose();
    } catch (e: any) {
      console.error('Quick capture failed:', e);
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSave();
    }
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window on:keydown={(e) => { if (e.key === 'Escape') onClose(); }} />

<div
  class="fixed inset-0 bg-black/30 flex items-start justify-center pt-20 z-50"
  role="button"
  tabindex="-1"
  onclick={onClose}
  onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}
>
  <div
    class="bg-bg rounded-lg shadow-xl border border-border p-3 w-96"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <div class="flex items-center justify-between mb-2">
      <h3 class="text-sm font-semibold">⚡ Quick Capture</h3>
      <button onclick={onClose} class="text-fg-muted hover:text-fg text-xs">✕</button>
    </div>

    <textarea
      bind:this={inputEl}
      bind:value={text}
      onkeydown={handleKeydown}
      placeholder="Type a note and press Enter to save…"
      class="w-full text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none resize-none h-24"
    ></textarea>

    <div class="flex items-center gap-2 mt-2">
      <select
        bind:value={categoryId}
        class="text-xs bg-bg-muted rounded px-2 py-1 border border-border flex-1"
      >
        <option value={null}>No category</option>
        {#each $categories as cat}
          <option value={cat.id}>{cat.name}</option>
        {/each}
      </select>
      <button
        onclick={handleSave}
        disabled={saving || !text.trim()}
        class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90 disabled:opacity-50"
      >
        {saving ? 'Saving…' : 'Save (Enter)'}
      </button>
    </div>
  </div>
</div>
