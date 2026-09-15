<script lang="ts">
  import type { Snippet } from 'svelte';

  export type BadgeColor = 'accent' | 'blue' | 'green' | 'amber' | 'red' | 'purple' | 'gray';

  let {
    color = 'gray',
    children,
    class: className = ''
  }: {
    color?: BadgeColor;
    children: Snippet;
    class?: string;
  } = $props();

  const colorClasses: Record<BadgeColor, string> = {
    accent: 'bg-accent-soft text-accent',
    blue: 'bg-info-soft text-info',
    green: 'bg-success-soft text-success',
    amber: 'bg-warning-soft text-warning',
    red: 'bg-danger-soft text-danger',
    purple: 'bg-purple-500/15 text-purple-600 dark:text-purple-400',
    gray: 'bg-bg-muted text-fg-muted'
  };
</script>

<span class="inline-flex items-center gap-0.5 text-[10px] font-medium px-1.5 py-0.5 rounded-full whitespace-nowrap {colorClasses[color]} {className}">
  {@render children()}
</span>
