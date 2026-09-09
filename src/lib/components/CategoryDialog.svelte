<script lang="ts">
  import { addCategory } from '$lib/stores/notes';
  import { categories } from '$lib/stores/notes';

  let { onClose } = $props<{ onClose: () => void }>();

  let name = $state('');
  let color = $state('#6b7280');
  let saving = $state(false);
  let error = $state('');

  const presets = [
    '#6b7280', '#3b82f6', '#f59e0b', '#10b981',
    '#ef4444', '#8b5cf6', '#ec4899', '#14b8a6',
  ];

  async function handleCreate() {
    if (!name.trim()) {
      error = 'Name is required';
      return;
    }
    saving = true;
    error = '';
    const ok = await addCategory(name.trim(), color);
    saving = false;
    if (ok) {
      name = '';
      onClose();
    } else {
      error = 'Failed to create category (check console)';
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
    class="bg-bg rounded-lg shadow-xl border border-border p-4 w-72"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <h3 class="text-sm font-semibold mb-3">New Category</h3>

    <label class="block text-xs text-fg-muted mb-1" for="cat-name">Name</label>
    <input
      id="cat-name"
      bind:value={name}
      placeholder="e.g. Phone calls"
      class="w-full text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none mb-3"
    />

    <label class="block text-xs text-fg-muted mb-1">Color</label>
    <div class="flex items-center gap-1.5 mb-1 flex-wrap">
      {#each presets as c}
        <button
          onclick={() => (color = c)}
          class="w-6 h-6 rounded-full border-2 transition {color === c ? 'border-fg scale-110' : 'border-transparent'}"
          style="background: {c}"
          aria-label="Select color {c}"
        ></button>
      {/each}
      <input
        type="color"
        bind:value={color}
        class="w-6 h-6 rounded cursor-pointer bg-transparent border border-border"
        title="Custom color"
      />
    </div>

    {#if error}
      <p class="text-xs text-red-500 mt-2 mb-2">{error}</p>
    {/if}

    {#if $categories.length > 0}
      <div class="mt-3 pt-3 border-t border-border">
        <p class="text-[10px] text-fg-muted mb-1">Existing categories:</p>
        <div class="flex flex-wrap gap-1">
          {#each $categories as cat}
            <span
              class="text-[10px] px-1.5 py-0.5 rounded-full"
              style="background: {cat.color}20; color: {cat.color}"
            >
              {cat.name}
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <div class="flex gap-2 justify-end mt-4">
      <button onclick={onClose} class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border">
        Cancel
      </button>
      <button
        onclick={handleCreate}
        disabled={saving}
        class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90 disabled:opacity-50"
      >
        {saving ? 'Creating…' : 'Create'}
      </button>
    </div>
  </div>
</div>
