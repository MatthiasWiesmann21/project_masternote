<script lang="ts">
  let {
    name,
    color = 'blue',
    size = 'sm'
  }: {
    name: string;
    color?: 'blue' | 'green' | 'accent' | 'purple' | 'amber';
    size?: 'xs' | 'sm' | 'md';
  } = $props();

  // Generate initials from name
  let initials = $derived.by(() => {
    const parts = name.trim().split(/\s+/).filter(Boolean);
    if (parts.length === 0) return '?';
    if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
    return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  });

  // Generate a consistent color from name hash
  let bgClass = $derived.by(() => {
    const colors: Record<string, string> = {
      blue: 'bg-info-soft text-info',
      green: 'bg-success-soft text-success',
      accent: 'bg-accent-soft text-accent',
      purple: 'bg-purple-500/15 text-purple-600 dark:text-purple-400',
      amber: 'bg-warning-soft text-warning'
    };
    return colors[color];
  });

  let sizeClass = $derived(size === 'xs' ? 'w-5 h-5 text-[9px]' : size === 'sm' ? 'w-6 h-6 text-[10px]' : 'w-8 h-8 text-xs');
</script>

<span class="inline-flex items-center justify-center rounded-full font-semibold shrink-0 {sizeClass} {bgClass}">
  {initials}
</span>
