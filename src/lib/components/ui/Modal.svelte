<script lang="ts">
  import type { Snippet } from 'svelte';
  import { X } from '@lucide/svelte';
  import { t } from '$lib/i18n';

  let {
    title,
    icon,
    onClose,
    width = 'w-80',
    children,
    footer
  }: {
    title: string;
    icon?: Snippet;
    onClose: () => void;
    width?: string;
    children: Snippet;
    footer?: Snippet;
  } = $props();
</script>

<svelte:window on:keydown={(e) => { if (e.key === 'Escape') onClose(); }} />

<div
  class="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in"
  role="button"
  tabindex="-1"
  onclick={onClose}
  onkeydown={() => {}}
>
  <div
    class="bg-surface-1 rounded-2xl shadow-xl border border-border {width} max-h-[85vh] flex flex-col animate-scale-in"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="flex items-center gap-2 px-4 py-3 border-b border-border">
      {#if icon}
        <span class="text-accent shrink-0">
          {@render icon()}
        </span>
      {/if}
      <h3 class="text-sm font-semibold flex-1">{title}</h3>
      <button
        onclick={onClose}
        class="text-fg-muted hover:text-fg hover:bg-bg-muted rounded-lg p-1 transition"
        title={$t('common.close')}
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto">
      {@render children()}
    </div>

    <!-- Footer -->
    {#if footer}
      <div class="px-4 py-3 border-t border-border flex gap-2 justify-end">
        {@render footer()}
      </div>
    {/if}
  </div>
</div>
