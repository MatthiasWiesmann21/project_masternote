<script lang="ts">
  import type { Snippet } from 'svelte';

  export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'success';
  export type ButtonSize = 'sm' | 'md' | 'icon';

  let {
    variant = 'secondary',
    size = 'sm',
    title,
    disabled = false,
    onclick,
    children,
    class: className = ''
  }: {
    variant?: ButtonVariant;
    size?: ButtonSize;
    title?: string;
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
    children?: Snippet;
    class?: string;
  } = $props();

  const variantClasses: Record<ButtonVariant, string> = {
    primary: 'bg-accent text-accent-fg hover:bg-accent-dark shadow-sm hover:shadow-accent',
    secondary: 'bg-bg-muted hover:bg-border text-fg',
    ghost: 'hover:bg-accent-soft text-fg-muted hover:text-fg',
    danger: 'text-danger hover:bg-danger-soft',
    success: 'bg-success text-white hover:opacity-90 shadow-sm'
  };

  const sizeClasses: Record<ButtonSize, string> = {
    sm: 'text-xs px-2.5 py-1.5 rounded-lg gap-1',
    md: 'text-sm px-3 py-2 rounded-lg gap-1.5',
    icon: 'p-1.5 rounded-lg'
  };
</script>

<button
  {onclick}
  {title}
  {disabled}
  class="inline-flex items-center justify-center font-medium transition-all duration-150 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100 {variantClasses[variant]} {sizeClasses[size]} {className}"
>
  {@render children?.()}
</button>
