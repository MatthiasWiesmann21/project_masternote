<script lang="ts">
  import { Keyboard } from '@lucide/svelte';
  import { t } from '$lib/i18n';

  let { value = $bindable(''), onApply }: { value: string; onApply: (v: string) => void } = $props();

  let recording = $state(false);
  let btnEl = $state<HTMLButtonElement | null>(null);

  const MODIFIER_KEYS = ['Control', 'Shift', 'Alt', 'Meta'];

  const KEY_MAP: Record<string, string> = {
    ' ': 'Space',
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
    Enter: 'Enter',
    Tab: 'Tab',
    Home: 'Home',
    End: 'End',
    PageUp: 'PageUp',
    PageDown: 'PageDown',
    Insert: 'Insert',
    Delete: 'Delete',
    Backspace: 'Backspace',
    Escape: 'Esc',
  };

  function comboFromEvent(e: KeyboardEvent): string | null {
    if (MODIFIER_KEYS.includes(e.key)) return null;
    const parts: string[] = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    if (e.metaKey) parts.push('Meta');
    const key = KEY_MAP[e.key] ?? (e.key.length === 1 ? e.key.toUpperCase() : e.key);
    parts.push(key);
    return parts.join('+');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === 'Escape' && !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) {
      recording = false;
      btnEl?.blur();
      return;
    }
    if ((e.key === 'Backspace' || e.key === 'Delete') && !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) {
      value = '';
      recording = false;
      onApply('');
      btnEl?.blur();
      return;
    }
    const combo = comboFromEvent(e);
    if (combo) {
      value = combo;
      recording = false;
      onApply(combo);
      btnEl?.blur();
    }
  }

  let parts = $derived(value ? value.split('+') : []);
</script>

<button
  bind:this={btnEl}
  onclick={() => { recording = true; btnEl?.focus(); }}
  onkeydown={handleKeydown}
  onblur={() => (recording = false)}
  class="flex-1 min-w-0 flex items-center gap-1 bg-bg-muted rounded-lg px-2 py-1.5 border outline-none transition text-left {recording ? 'border-accent ring-2 ring-accent/20' : 'border-border hover:border-border-strong'}"
  title={$t('hotkey.hint')}
>
  <Keyboard class="w-3.5 h-3.5 text-fg-muted shrink-0" />
  {#if recording}
    <span class="text-[11px] text-accent animate-pulse truncate">{$t('hotkey.recording')}</span>
  {:else if parts.length > 0}
    <span class="flex items-center gap-0.5 flex-wrap min-w-0">
      {#each parts as part, i}
        {#if i > 0}<span class="text-fg-muted text-[10px]">+</span>{/if}
        <kbd class="font-mono text-[10px] bg-bg px-1.5 py-0.5 rounded-md border border-border shadow-sm">{part}</kbd>
      {/each}
    </span>
  {:else}
    <span class="text-[11px] text-fg-muted italic">{$t('hotkey.none')}</span>
  {/if}
</button>
