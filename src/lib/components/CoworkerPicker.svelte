<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '$lib/api';
  import type { Coworker } from '$lib/api';
  import { t } from '$lib/i18n';

  let { selectedId, onSelect }: { selectedId: number | null; onSelect: (id: number | null) => void } = $props();

  let query = $state('');
  let results = $state<Coworker[]>([]);
  let showDropdown = $state(false);
  let selectedCoworker = $state<Coworker | null>(null);
  let searchEl = $state<HTMLInputElement | null>(null);
  let highlightedIndex = $state(-1);
  let dropdownEl = $state<HTMLDivElement | null>(null);

  export function focusSearch() {
    if (selectedCoworker) {
      clear();
      setTimeout(() => { searchEl?.focus(); searchEl?.select(); }, 0);
    } else {
      searchEl?.focus();
      searchEl?.select();
    }
  }

  async function loadSelected() {
    if (selectedId !== null) {
      try {
        const all = await api.listCoworkers();
        selectedCoworker = all.find((c) => c.id === selectedId) ?? null;
      } catch {
        selectedCoworker = null;
      }
    } else {
      selectedCoworker = null;
    }
  }

  onMount(loadSelected);

  let lastSelected = $state<number | null>(null);
  $effect(() => {
    if (selectedId !== lastSelected) {
      lastSelected = selectedId;
      loadSelected();
    }
  });

  async function search() {
    if (query.trim().length < 1) {
      results = [];
      return;
    }
    try {
      results = await api.searchCoworkers(query.trim());
      highlightedIndex = -1;
    } catch {
      results = [];
    }
  }

  $effect(() => {
    query;
    const timer = setTimeout(search, 150);
    return () => clearTimeout(timer);
  });

  function pick(c: Coworker) {
    onSelect(c.id);
    selectedCoworker = c;
    query = '';
    results = [];
    showDropdown = false;
    highlightedIndex = -1;
  }

  function clear() {
    onSelect(null);
    selectedCoworker = null;
    query = '';
    results = [];
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!showDropdown || results.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      highlightedIndex = Math.min(highlightedIndex + 1, results.length - 1);
      scrollIntoView();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      highlightedIndex = Math.max(highlightedIndex - 1, 0);
      scrollIntoView();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (highlightedIndex >= 0 && highlightedIndex < results.length) {
        pick(results[highlightedIndex]);
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

  function fullName(c: Coworker): string {
    return [c.firstName, c.lastName].filter(Boolean).join(' ');
  }
</script>

<div class="relative">
  {#if selectedCoworker}
    <div class="flex items-center gap-1.5 text-xs">
      <span class="px-1.5 py-0.5 rounded-full bg-green-500/15 text-green-600 dark:text-green-400 whitespace-nowrap">
        🤝 {fullName(selectedCoworker)}
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
      placeholder={$t('picker.coworker')}
      class="text-xs bg-bg-muted rounded px-2 py-1 border border-border outline-none w-full"
    />
    {#if showDropdown && results.length > 0}
      <div bind:this={dropdownEl} class="absolute top-full left-0 right-0 mt-1 bg-bg rounded-lg border border-border shadow-lg max-h-48 overflow-y-auto z-50">
        {#each results as c, i (c.id)}
          <button
            data-idx={i}
            onmousedown={() => pick(c)}
            onmouseenter={() => (highlightedIndex = i)}
            class="w-full text-left px-2 py-1.5 text-xs border-b border-border last:border-0 {i === highlightedIndex ? 'bg-accent text-accent-fg' : 'hover:bg-bg-subtle'}"
          >
            <div class="font-medium">{fullName(c)}</div>
            {#if c.email}
              <div class="text-[10px] text-fg-muted">{c.email}</div>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>
