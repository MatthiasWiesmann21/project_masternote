<script lang="ts">
  import { categories } from '$lib/stores/notes';
  import type { Category } from '$lib/api';

  let { selectedId, onSelect }: { selectedId: number | null; onSelect: (id: number | null) => void } = $props();

  let query = $state('');
  let showDropdown = $state(false);
  let searchEl = $state<HTMLInputElement | null>(null);
  let dropdownEl = $state<HTMLDivElement | null>(null);
  let highlightedIndex = $state(-1);

  let selectedCategory = $state<Category | null>(null);

  $effect(() => {
    if (selectedId !== null) {
      selectedCategory = $categories.find((c) => c.id === selectedId) ?? null;
    } else {
      selectedCategory = null;
    }
  });

  let filtered = $derived(
    query.trim()
      ? $categories.filter((c) => c.name.toLowerCase().includes(query.trim().toLowerCase()))
      : $categories
  );

  $effect(() => {
    query;
    highlightedIndex = -1;
  });

  export function focusSearch() {
    if (selectedCategory) {
      clear();
      setTimeout(() => { searchEl?.focus(); searchEl?.select(); }, 0);
    } else {
      searchEl?.focus();
      searchEl?.select();
    }
  }

  function pick(c: Category | null) {
    onSelect(c?.id ?? null);
    selectedCategory = c;
    query = '';
    showDropdown = false;
    highlightedIndex = -1;
  }

  function clear() {
    onSelect(null);
    selectedCategory = null;
    query = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!showDropdown) return;
    const items = filtered;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightedIndex = Math.min(highlightedIndex + 1, items.length - 1);
      scrollIntoView();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightedIndex = Math.max(highlightedIndex - 1, -1);
      scrollIntoView();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (highlightedIndex >= 0 && highlightedIndex < items.length) {
        pick(items[highlightedIndex]);
      } else if (highlightedIndex === -1 && items.length > 0) {
        pick(items[0]);
      }
    } else if (e.key === 'Escape') {
      showDropdown = false;
      highlightedIndex = -1;
    }
  }

  function scrollIntoView() {
    setTimeout(() => {
      const items = dropdownEl?.querySelectorAll('[data-idx]');
      if (items && highlightedIndex >= 0 && highlightedIndex < items.length) {
        items[highlightedIndex].scrollIntoView({ block: 'nearest' });
      }
    }, 0);
  }
</script>

<div class="relative">
  {#if selectedCategory}
    <div class="flex items-center gap-1.5 text-xs">
      <span
        class="px-1.5 py-0.5 rounded-full whitespace-nowrap text-white"
        style="background: {selectedCategory.color}"
      >
        {selectedCategory.name}
      </span>
      <button onclick={clear} class="text-fg-muted hover:text-red-500 text-[10px]">✕</button>
    </div>
  {:else}
    <input
      bind:this={searchEl}
      bind:value={query}
      onfocus={() => (showDropdown = true)}
      onblur={() => setTimeout(() => (showDropdown = false), 150)}
      onkeydown={handleKeydown}
      placeholder="Category… (Ctrl+Shift+C)"
      class="text-xs bg-bg-muted rounded px-2 py-1 border border-border outline-none w-32"
    />
    {#if showDropdown && filtered.length > 0}
      <div bind:this={dropdownEl} class="absolute top-full right-0 mt-1 bg-bg rounded-lg border border-border shadow-lg max-h-48 overflow-y-auto z-50 min-w-40">
        {#each filtered as c, i (c.id)}
          <button
            data-idx={i}
            onmousedown={() => pick(c)}
            onmouseenter={() => (highlightedIndex = i)}
            class="w-full text-left px-2 py-1.5 text-xs border-b border-border last:border-0 {i === highlightedIndex ? 'bg-accent text-accent-fg' : 'hover:bg-bg-subtle'}"
          >
            <span class="inline-block w-2 h-2 rounded-full mr-1.5" style="background: {c.color}"></span>
            {c.name}
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>
