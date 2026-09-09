<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '$lib/api';
  import type { Contact } from '$lib/api';

  let { selectedId, onSelect }: { selectedId: number | null; onSelect: (id: number | null) => void } = $props();

  let query = $state('');
  let results = $state<Contact[]>([]);
  let showDropdown = $state(false);
  let selectedContact = $state<Contact | null>(null);

  async function loadSelected() {
    if (selectedId !== null) {
      try {
        const all = await api.listContacts();
        selectedContact = all.find((c) => c.id === selectedId) ?? null;
      } catch {
        selectedContact = null;
      }
    } else {
      selectedContact = null;
    }
  }

  onMount(loadSelected);

  // Reload when selectedId changes
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
      results = await api.searchContacts(query.trim());
    } catch {
      results = [];
    }
  }

  $effect(() => {
    query;
    const timer = setTimeout(search, 150);
    return () => clearTimeout(timer);
  });

  function pick(c: Contact) {
    onSelect(c.id);
    selectedContact = c;
    query = '';
    results = [];
    showDropdown = false;
  }

  function clear() {
    onSelect(null);
    selectedContact = null;
    query = '';
    results = [];
  }

  function fullName(c: Contact): string {
    return c.kind === 'company' && c.companyName
      ? c.companyName
      : [c.firstName, c.lastName].filter(Boolean).join(' ');
  }
</script>

<div class="relative">
  {#if selectedContact}
    <div class="flex items-center gap-1.5 text-xs">
      <span class="px-1.5 py-0.5 rounded-full bg-blue-500/15 text-blue-600 dark:text-blue-400 whitespace-nowrap">
        👤 {fullName(selectedContact)}
      </span>
      <button onclick={clear} class="text-fg-muted hover:text-red-500 text-[10px]">✕</button>
    </div>
  {:else}
    <input
      bind:value={query}
      onfocus={() => (showDropdown = true)}
      onblur={() => setTimeout(() => (showDropdown = false), 150)}
      placeholder="Search contact…"
      class="text-xs bg-bg-muted rounded px-2 py-1 border border-border outline-none w-full"
    />
    {#if showDropdown && results.length > 0}
      <div class="absolute top-full left-0 right-0 mt-1 bg-bg rounded-lg border border-border shadow-lg max-h-48 overflow-y-auto z-50">
        {#each results as c (c.id)}
          <button
            onmousedown={() => pick(c)}
            class="w-full text-left px-2 py-1.5 hover:bg-bg-subtle text-xs border-b border-border last:border-0"
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
