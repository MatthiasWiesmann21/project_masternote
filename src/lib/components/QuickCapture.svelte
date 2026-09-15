<script lang="ts">
  import * as api from '$lib/api';
  import { categories } from '$lib/stores/notes';
  import Modal from '$lib/components/ui/Modal.svelte';
  import { Zap } from '@lucide/svelte';
  import { t } from '$lib/i18n';

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
  }
</script>

<Modal onClose={onClose} width="w-96" title={$t('quick.title')}>
  {#snippet icon()}<Zap class="w-4 h-4" />{/snippet}

  <div class="p-4 space-y-3">
    <textarea
      bind:this={inputEl}
      bind:value={text}
      onkeydown={handleKeydown}
      placeholder={$t('quick.placeholder')}
      class="w-full text-sm bg-bg-muted rounded-lg px-2.5 py-2 border border-border outline-none resize-none h-28 focus:border-accent focus:ring-2 focus:ring-accent/20 placeholder:text-fg-muted"
    ></textarea>

    <div class="flex items-center gap-2">
      <select
        bind:value={categoryId}
        class="text-xs bg-bg-muted rounded-lg px-2 py-1.5 border border-border flex-1 outline-none focus:border-accent"
      >
        <option value={null}>{$t('quick.noCategory')}</option>
        {#each $categories as cat}
          <option value={cat.id}>{cat.name}</option>
        {/each}
      </select>
    </div>
  </div>

  {#snippet footer()}
    <button onclick={onClose} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
    <button
      onclick={handleSave}
      disabled={saving || !text.trim()}
      class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition disabled:opacity-50"
    >
      {saving ? $t('quick.saving') : $t('quick.save')}
    </button>
  {/snippet}
</Modal>
