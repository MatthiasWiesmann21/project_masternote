<script lang="ts">
  import Modal from '$lib/components/ui/Modal.svelte';
  import { Keyboard } from '@lucide/svelte';
  import { t } from '$lib/i18n';

  let { onClose } = $props<{ onClose: () => void }>();

  const shortcuts = [
    { group: 'shortcuts.global', items: [
      { key: 'Ctrl+Shift+M', desc: 'shortcuts.toggleWidget' },
      { key: 'Ctrl+Shift+N', desc: 'shortcuts.saveClose' },
      { key: 'Ctrl+Shift+Q', desc: 'shortcuts.quickCapture' },
      { key: 'Esc', desc: 'shortcuts.hide' },
    ]},
    { group: 'shortcuts.notes', items: [
      { key: 'Ctrl+N', desc: 'shortcuts.newNote' },
      { key: 'Ctrl+S / Ctrl+Enter', desc: 'shortcuts.saveNote' },
      { key: 'Ctrl+Shift+Tab', desc: 'shortcuts.nextNote' },
      { key: 'Ctrl+F', desc: 'shortcuts.focusSearch' },
      { key: 'Ctrl+K', desc: 'shortcuts.commandPalette' },
    ]},
    { group: 'shortcuts.editor', items: [
      { key: 'Ctrl+Shift+T', desc: 'shortcuts.focusTitle' },
      { key: 'Ctrl+Shift+D', desc: 'shortcuts.focusDesc' },
      { key: 'Ctrl+Shift+C', desc: 'shortcuts.focusCategory' },
      { key: 'Ctrl+Shift+K', desc: 'shortcuts.focusContact' },
      { key: 'Ctrl+Shift+H', desc: 'shortcuts.focusCoworker' },
    ]},
    { group: 'shortcuts.ui', items: [
      { key: 'Ctrl+Shift+?', desc: 'shortcuts.showHelp' },
    ]},
  ];
</script>

<Modal onClose={onClose} width="w-96" title={$t('shortcuts.title')}>
  {#snippet icon()}<Keyboard class="w-4 h-4" />{/snippet}

  <div class="p-4 space-y-3">
    {#each shortcuts as group}
      <div>
        <div class="text-[10px] text-fg-muted font-semibold uppercase tracking-wider mb-1.5">{$t(group.group)}</div>
        {#each group.items as item}
          <div class="flex items-center justify-between py-1 text-xs">
            <span class="text-fg-muted">{$t(item.desc)}</span>
            <kbd class="font-mono text-[10px] bg-bg-muted px-1.5 py-0.5 rounded-md border border-border shadow-sm">{item.key}</kbd>
          </div>
        {/each}
      </div>
    {/each}
  </div>
</Modal>
