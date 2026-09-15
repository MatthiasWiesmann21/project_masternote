<script lang="ts">
  import { addCategory } from '$lib/stores/notes';
  import { categories } from '$lib/stores/notes';
  import Modal from '$lib/components/ui/Modal.svelte';
  import { FolderPlus } from '@lucide/svelte';
  import { t } from '$lib/i18n';
  import { get } from 'svelte/store';

  let { onClose } = $props<{ onClose: () => void }>();

  let name = $state('');
  let color = $state('#6366f1');
  let saving = $state(false);
  let error = $state('');

  const presets = [
    '#6366f1', '#3b82f6', '#f59e0b', '#10b981',
    '#ef4444', '#8b5cf6', '#ec4899', '#14b8a6',
  ];

  async function handleCreate() {
    if (!name.trim()) {
      error = get(t)('cat.nameRequired');
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
      error = get(t)('cat.createFailed');
    }
  }
</script>

<Modal onClose={onClose} width="w-80" title={$t('cat.title')}>
  {#snippet icon()}<FolderPlus class="w-4 h-4" />{/snippet}

  <div class="p-4 space-y-3">
    <label class="block">
      <span class="text-xs text-fg-muted mb-1 block font-medium">{$t('cat.name')}</span>
      <input
        bind:value={name}
        placeholder={$t('cat.namePlaceholder')}
        class="w-full text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20"
      />
    </label>

    <div>
      <span class="text-xs text-fg-muted mb-1.5 block font-medium">{$t('cat.color')}</span>
      <div class="flex items-center gap-1.5 flex-wrap">
        {#each presets as c}
          <button
            onclick={() => (color = c)}
            class="w-6 h-6 rounded-full border-2 transition {color === c ? 'border-fg scale-110' : 'border-transparent'}"
            style="background: {c}"
            aria-label={$t('cat.selectColor', { color: c })}
          ></button>
        {/each}
        <input
          type="color"
          bind:value={color}
          class="w-6 h-6 rounded cursor-pointer bg-transparent border border-border"
          title={$t('cat.customColor')}
        />
      </div>
    </div>

    {#if error}
      <p class="text-xs text-danger">{error}</p>
    {/if}

    {#if $categories.length > 0}
      <div class="pt-3 border-t border-border">
        <p class="text-[10px] text-fg-muted mb-1.5 font-medium">{$t('cat.existing')}</p>
        <div class="flex flex-wrap gap-1">
          {#each $categories as cat}
            <span
              class="text-[10px] px-1.5 py-0.5 rounded-full font-medium"
              style="background: {cat.color}20; color: {cat.color}"
            >
              {cat.name}
            </span>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <button onclick={onClose} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
    <button
      onclick={handleCreate}
      disabled={saving}
      class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition disabled:opacity-50"
    >
      {saving ? $t('cat.creating') : $t('common.create')}
    </button>
  {/snippet}
</Modal>
