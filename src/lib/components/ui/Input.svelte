<script lang="ts">
  let {
    value = $bindable(),
    type = 'text',
    placeholder = '',
    class: className = '',
    oninput,
    onkeydown,
    onfocus,
    onblur
  }: {
    value?: string;
    type?: string;
    placeholder?: string;
    class?: string;
    oninput?: (e: Event) => void;
    onkeydown?: (e: KeyboardEvent) => void;
    onfocus?: (e: FocusEvent) => void;
    onblur?: (e: FocusEvent) => void;
  } = $props();

  let el = $state<HTMLInputElement | null>(null);
  export function getEl() { return el; }
  export function focus() { el?.focus(); el?.select(); }
</script>

<input
  bind:this={el}
  bind:value
  {type}
  {placeholder}
  {oninput}
  {onkeydown}
  {onfocus}
  {onblur}
  class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none transition focus:border-accent focus:ring-2 focus:ring-accent/20 placeholder:text-fg-muted {className}"
/>
