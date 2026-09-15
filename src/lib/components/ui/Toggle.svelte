<script lang="ts">
  let {
    checked = $bindable(),
    onchange,
    label = ''
  }: {
    checked?: boolean;
    onchange?: (e: Event) => void;
    label?: string;
  } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {onchange}
  onclick={() => (checked = !checked)}
  class="relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors duration-200 {checked ? 'bg-accent' : 'bg-bg-muted border border-border'}"
>
  <span class="inline-block h-3.5 w-3.5 transform rounded-full bg-white shadow-sm transition-transform duration-200 {checked ? 'translate-x-4' : 'translate-x-1'}"></span>
</button>
