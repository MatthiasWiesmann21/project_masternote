<script lang="ts">
  import { searchQuery, notes, loadNotes } from '$lib/stores/notes';
  import * as api from '$lib/api';
  import type { SearchResult } from '$lib/api';

  let results = $state<SearchResult[]>([]);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let isSearching = $state(false);

  let query = $state('');

  function onInput() {
    searchQuery.set(query);
    if (debounceTimer) clearTimeout(debounceTimer);
    if (!query.trim()) {
      results = [];
      isSearching = false;
      return;
    }
    isSearching = true;
    debounceTimer = setTimeout(async () => {
      try {
        results = await api.searchNotes(query, 50);
      } catch (e) {
        console.error('Search failed:', e);
        results = [];
      } finally {
        isSearching = false;
      }
    }, 200);
  }

  function clearSearch() {
    query = '';
    results = [];
    isSearching = false;
    searchQuery.set('');
  }

  function selectResult(r: SearchResult) {
    // Emit event to load the note in editor
    const event = new CustomEvent('select-search-result', { detail: r.id });
    window.dispatchEvent(event);
    clearSearch();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      clearSearch();
    }
  }

  let inputEl = $state<HTMLInputElement | null>(null);

  export function focus() {
    inputEl?.focus();
    inputEl?.select();
  }
</script>

<svelte:window on:keydown={(e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
    e.preventDefault();
    focus();
  }
}} />

<div class="relative">
  <div class="flex items-center gap-2 px-3 py-2 border-b border-border bg-bg-subtle">
    <svg class="w-3.5 h-3.5 text-fg-muted shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </svg>
    <input
      bind:this={inputEl}
      bind:value={query}
      oninput={onInput}
      onkeydown={onKeydown}
      placeholder="Search notes… (Ctrl+F)"
      class="flex-1 bg-transparent text-sm outline-none placeholder:text-fg-muted"
    />
    {#if query}
      <button onclick={clearSearch} class="text-fg-muted hover:text-fg text-xs">✕</button>
    {/if}
  </div>

  {#if isSearching || results.length > 0}
    <div class="absolute left-0 right-0 top-full bg-bg border border-border rounded-b-lg shadow-lg z-50 max-h-64 overflow-y-auto">
      {#if isSearching}
        <div class="px-3 py-2 text-xs text-fg-muted">Searching…</div>
      {:else if results.length === 0}
        <div class="px-3 py-2 text-xs text-fg-muted">No results</div>
      {:else}
        {#each results as r (r.id)}
          <button
            onclick={() => selectResult(r)}
            class="w-full text-left px-3 py-2 hover:bg-bg-subtle border-b border-border last:border-0"
          >
            <div class="text-sm font-medium truncate">{r.title || 'Untitled'}</div>
            <div class="text-xs text-fg-muted truncate">{r.snippet}</div>
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>
