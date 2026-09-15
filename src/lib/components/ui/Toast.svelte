<script lang="ts">
  import { toasts, dismissToast } from '$lib/stores/toast';
  import { CheckCircle, AlertCircle, Info, AlertTriangle, X } from '@lucide/svelte';
  import type { ToastType } from '$lib/stores/toast';

  const config: Record<ToastType, { icon: typeof Info, cls: string }> = {
    success: { icon: CheckCircle, cls: 'text-success' },
    error: { icon: AlertCircle, cls: 'text-danger' },
    info: { icon: Info, cls: 'text-info' },
    warning: { icon: AlertTriangle, cls: 'text-warning' }
  };
</script>

{#if $toasts.length > 0}
  <div class="fixed top-2 right-2 z-[100] flex flex-col gap-1.5 pointer-events-none">
    {#each $toasts as toast (toast.id)}
      {@const C = config[toast.type].icon}
      <div
        class="flex items-center gap-2 bg-surface-1 border border-border rounded-xl shadow-lg px-3 py-2 max-w-72 animate-slide-down pointer-events-auto"
      >
        <span class="shrink-0 {config[toast.type].cls}">
          <C class="w-4 h-4" />
        </span>
        <span class="text-xs flex-1">{toast.message}</span>
        <button
          onclick={() => dismissToast(toast.id)}
          class="text-fg-muted hover:text-fg shrink-0"
        >
          <X class="w-3 h-3" />
        </button>
      </div>
    {/each}
  </div>
{/if}
