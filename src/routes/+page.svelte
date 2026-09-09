<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import NoteEditor from '$lib/components/NoteEditor.svelte';
  import NoteList from '$lib/components/NoteList.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import CategoryDialog from '$lib/components/CategoryDialog.svelte';
  import {
    notes,
    selectedNoteId,
    refreshAll,
    loadNotes,
    activeTagFilter,
    activeCategoryFilter,
    tags,
    categories,
    lastError
  } from '$lib/stores/notes';
  import { settings } from '$lib/stores/settings';
  import * as api from '$lib/api';

  let editor = $state<NoteEditor>();
  let searchBar = $state<SearchBar>();
  let showSettings = $state(false);
  let showSidebar = $state(true);
  let showCategoryDialog = $state(false);

  function newNoteAndFocus() {
    editor?.newNote();
    setTimeout(() => editor?.focus(), 50);
  }

  // React to selected note changes
  let lastSelected = $state<number | null>(null);
  $effect(() => {
    const id = $selectedNoteId;
    if (id !== null && id !== lastSelected) {
      lastSelected = id;
      const n = $notes.find((x) => x.id === id);
      if (n) {
        editor?.loadNote(n);
      }
    }
  });

  // Search result selection handler
  function handleSearchResult(e: Event) {
    const id = (e as CustomEvent).detail as number;
    selectedNoteId.set(id);
    api.getNote(id).then((n) => {
      editor?.loadNote(n);
    });
  }

  onMount(async () => {
    window.addEventListener('select-search-result', handleSearchResult);
    await refreshAll();
    newNoteAndFocus();
  });

  onDestroy(() => {
    window.removeEventListener('select-search-result', handleSearchResult);
  });

  function toggleTagFilter(tagName: string) {
    activeTagFilter.update((v) => (v === tagName ? null : tagName));
  }

  function toggleCategoryFilter(catId: number) {
    activeCategoryFilter.update((v) => (v === catId ? null : catId));
  }

  function clearFilters() {
    activeTagFilter.set(null);
    activeCategoryFilter.set(null);
  }

  async function handleHide() {
    try {
      await api.hideWidget();
    } catch (e) {
      console.error('Failed to hide widget:', e);
    }
  }

  async function signInGraph() {
    try {
      await api.graphSignIn();
      settings.update((s) => ({ ...s, graphSignedIn: true }));
    } catch (e) {
      console.error('Graph sign in failed:', e);
    }
  }

  async function signOutGraph() {
    try {
      await api.graphSignOut();
      settings.update((s) => ({ ...s, graphSignedIn: false }));
    } catch (e) {
      console.error('Graph sign out failed:', e);
    }
  }

  let hasFilters = $derived($activeTagFilter !== null || $activeCategoryFilter !== null);
</script>

<svelte:window
  on:keydown={(e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
      e.preventDefault();
      newNoteAndFocus();
    }
    if (e.key === 'Escape') {
      handleHide();
    }
  }}
/>

<div class="flex flex-col h-screen bg-bg rounded-lg overflow-hidden border border-border shadow-2xl">
  <!-- Title bar / drag region -->
  <div data-tauri-drag-region class="flex items-center gap-2 px-3 py-1.5 bg-bg-subtle border-b border-border select-none">
    <span data-tauri-drag-region class="text-xs font-semibold text-fg flex-1">📝 MasterNote</span>
    <button onclick={() => (showSidebar = !showSidebar)} class="text-fg-muted hover:text-fg text-xs px-1" title="Toggle sidebar">
      ☰
    </button>
    <button onclick={() => (showCategoryDialog = true)} class="text-fg-muted hover:text-fg text-xs px-1" title="New category">
      📁
    </button>
    <button onclick={() => (showSettings = !showSettings)} class="text-fg-muted hover:text-fg text-xs px-1" title="Settings">
      ⚙
    </button>
    <button onclick={handleHide} class="text-fg-muted hover:text-fg text-xs px-1" title="Hide (Esc)">
      ✕
    </button>
  </div>

  <SearchBar bind:this={searchBar} />

  {#if $lastError}
    <div class="px-3 py-1.5 text-xs text-red-500 bg-red-500/10 border-b border-red-500/20 flex items-center gap-2">
      <span class="flex-1">{$lastError}</span>
      <button onclick={() => lastError.set(null)} class="text-red-500 hover:text-red-700">✕</button>
    </div>
  {/if}

  {#if showSettings}
    <div class="flex flex-col gap-2 px-3 py-2 border-b border-border bg-bg-subtle text-xs">
      <label class="flex items-center gap-2">
        <span class="text-fg-muted">Theme</span>
        <select
          value={$settings.theme}
          onchange={(e) => settings.update((s) => ({ ...s, theme: (e.target as HTMLSelectElement).value as any }))}
          class="bg-bg-muted rounded px-2 py-1 border border-border"
        >
          <option value="system">System</option>
          <option value="light">Light</option>
          <option value="dark">Dark</option>
        </select>
      </label>
      <label class="flex items-center gap-2">
        <input
          type="checkbox"
          checked={$settings.hideOnBlur}
          onchange={(e) => settings.update((s) => ({ ...s, hideOnBlur: (e.target as HTMLInputElement).checked }))}
          class="accent-accent"
        />
        <span class="text-fg-muted">Hide widget when clicking outside</span>
      </label>
      <div class="flex items-center gap-2">
        <span class="text-fg-muted">Outlook:</span>
        {#if $settings.graphSignedIn}
          <button onclick={signOutGraph} class="text-xs px-2 py-1 rounded bg-bg-muted hover:bg-border">Sign out</button>
          <span class="text-green-500">✓ Connected</span>
        {:else}
          <button onclick={signInGraph} class="text-xs px-2 py-1 rounded bg-accent text-accent-fg hover:opacity-90">Sign in</button>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Filter chips -->
  {#if $tags.length > 0 || $categories.length > 0}
    <div class="flex items-center gap-1 px-3 py-1 border-b border-border bg-bg-subtle overflow-x-auto">
      {#each $categories as cat}
        <button
          onclick={() => toggleCategoryFilter(cat.id)}
          class="text-[10px] px-1.5 py-0.5 rounded-full whitespace-nowrap transition {$activeCategoryFilter === cat.id ? 'text-white' : 'opacity-60 hover:opacity-100'}"
          style="background: {cat.color}"
        >
          {cat.name}
        </button>
      {/each}
      {#each $tags as tag}
        <button
          onclick={() => toggleTagFilter(tag.name)}
          class="text-[10px] px-1.5 py-0.5 rounded-full whitespace-nowrap transition {$activeTagFilter === tag.name ? 'bg-accent text-accent-fg' : 'bg-bg-muted text-fg-muted hover:bg-border'}"
        >
          #{tag.name}
        </button>
      {/each}
      {#if hasFilters}
        <button onclick={clearFilters} class="text-[10px] px-1.5 py-0.5 text-fg-muted hover:text-fg">clear</button>
      {/if}
    </div>
  {/if}

  <!-- Main content: sidebar + editor -->
  <div class="flex flex-1 overflow-hidden">
    {#if showSidebar}
      <div class="w-56 border-r border-border overflow-hidden shrink-0">
        <NoteList notes={$notes} />
      </div>
    {/if}

    <div class="flex-1 overflow-hidden">
      <NoteEditor bind:this={editor} />
    </div>
  </div>
</div>

{#if showCategoryDialog}
  <CategoryDialog onClose={() => (showCategoryDialog = false)} />
{/if}
