<script lang="ts">
  let { onClose } = $props<{ onClose: () => void }>();

  const shortcuts = [
    { group: 'Global', items: [
      { key: 'Ctrl+Shift+M', desc: 'Toggle widget visibility' },
      { key: 'Ctrl+Shift+N', desc: 'Save and close widget' },
      { key: 'Ctrl+Shift+Q', desc: 'Quick capture' },
      { key: 'Esc', desc: 'Hide widget' },
    ]},
    { group: 'Notes', items: [
      { key: 'Ctrl+N', desc: 'New note' },
      { key: 'Ctrl+S / Ctrl+Enter', desc: 'Save note' },
      { key: 'Ctrl+Shift+Tab', desc: 'Select next note' },
      { key: 'Ctrl+F', desc: 'Focus search' },
    ]},
    { group: 'Editor', items: [
      { key: 'Ctrl+Shift+T', desc: 'Focus title' },
      { key: 'Ctrl+Shift+D', desc: 'Focus description' },
      { key: 'Ctrl+Shift+C', desc: 'Focus category picker' },
      { key: 'Ctrl+Shift+K', desc: 'Focus contact picker' },
      { key: 'Ctrl+Shift+H', desc: 'Focus coworker picker' },
    ]},
    { group: 'UI', items: [
      { key: 'Ctrl+Shift+?', desc: 'Show this help' },
    ]},
  ];
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
    class="bg-bg rounded-lg shadow-xl border border-border p-4 w-96 max-h-[80vh] overflow-y-auto"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-sm font-semibold">Keyboard Shortcuts</h3>
      <button onclick={onClose} class="text-fg-muted hover:text-fg text-xs">✕</button>
    </div>

    {#each shortcuts as group}
      <div class="mb-3">
        <div class="text-[10px] text-fg-muted font-medium uppercase mb-1">{group.group}</div>
        {#each group.items as item}
          <div class="flex items-center justify-between py-0.5 text-xs">
            <span class="text-fg-muted">{item.desc}</span>
            <kbd class="font-mono text-[10px] bg-bg-muted px-1.5 py-0.5 rounded border border-border">{item.key}</kbd>
          </div>
        {/each}
      </div>
    {/each}
  </div>
</div>
