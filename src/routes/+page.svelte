<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable } from 'svelte/store';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
  import NoteEditor from '$lib/components/NoteEditor.svelte';
  import NoteList from '$lib/components/NoteList.svelte';
  import SearchBar from '$lib/components/SearchBar.svelte';
  import CategoryDialog from '$lib/components/CategoryDialog.svelte';
  import ContactDialog from '$lib/components/ContactDialog.svelte';
  import CoworkerDialog from '$lib/components/CoworkerDialog.svelte';
  import ShortcutHelp from '$lib/components/ShortcutHelp.svelte';
  import QuickCapture from '$lib/components/QuickCapture.svelte';
  import {
    notes,
    selectedNoteId,
    refreshAll,
    loadNotes,
    activeTagFilter,
    activeCategoryFilter,
    activeContactFilter,
    activeCoworkerFilter,
    dateFrom,
    dateTo,
    tags,
    categories,
    timeRangeSort,
    lastError,
    showArchived,
    selectedNoteIds,
    searchQuery
  } from '$lib/stores/notes';
  import { settings } from '$lib/stores/settings';
  import * as api from '$lib/api';

  export const showFilterDropdown = writable(false);

  let editor = $state<NoteEditor>();
  let searchBar = $state<SearchBar>();
  let showSettings = $state(false);
  let showSidebar = $state(true);
  let showCategoryDialog = $state(false);
  let showContactDialog = $state(false);
  let showCoworkerDialog = $state(false);
  let showShortcutHelp = $state(false);
  let showQuickCapture = $state(false);
  let statistics = $state<api.NoteStatistics | null>(null);
  let contactList = $state<api.Contact[]>([]);
  let coworkerList = $state<api.Coworker[]>([]);
  let contactFilterQuery = $state('');
  let coworkerFilterQuery = $state('');

  let filteredContactList = $derived.by(() => {
    const q = contactFilterQuery.trim().toLowerCase();
    if (!q) return contactList;
    return contactList.filter((c) => {
      const name = c.kind === 'company' && c.companyName
        ? c.companyName
        : [c.firstName, c.lastName].filter(Boolean).join(' ');
      return name.toLowerCase().includes(q) ||
        (c.customerIdentifier ?? '').toLowerCase().includes(q) ||
        (c.email ?? '').toLowerCase().includes(q);
    });
  });

  let filteredCoworkerList = $derived.by(() => {
    const q = coworkerFilterQuery.trim().toLowerCase();
    if (!q) return coworkerList;
    return coworkerList.filter((c) => {
      const name = [c.firstName, c.lastName].filter(Boolean).join(' ');
      return name.toLowerCase().includes(q) || (c.email ?? '').toLowerCase().includes(q);
    });
  });

  // Hotkey settings
  let openHotkey = $state('Ctrl+Shift+M');
  let saveCloseHotkey = $state('Ctrl+Shift+N');
  let hotkeyError = $state('');
  let hotkeySaved = $state('');

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

  // Save and close handler (triggered by global hotkey)
  async function handleSaveAndClose() {
    await editor?.saveNow();
    try {
      await api.hideWidget();
    } catch (e) {
      console.error('Failed to hide widget:', e);
    }
  }

  // Quick capture handler (triggered by global hotkey)
  async function handleQuickCapture() {
    showQuickCapture = true;
  }

  let unlistenSaveClose: UnlistenFn | null = null;
  let unlistenDeviceCode: UnlistenFn | null = null;
  let unlistenQuickCapture: UnlistenFn | null = null;
  let deviceCodeMsg = $state('');
  let graphClientId = $state('');
  let graphClientIdSaved = $state('');

  async function loadStatistics() {
    try {
      statistics = await api.getStatistics();
    } catch (e) {
      console.error('Failed to load statistics:', e);
    }
  }

  onMount(async () => {
    window.addEventListener('select-search-result', handleSearchResult);
    // Listen for save-and-close event from backend (global hotkey)
    unlistenSaveClose = await listen('save-and-close', handleSaveAndClose);
    unlistenQuickCapture = await listen('quick-capture', handleQuickCapture);
    // Listen for Graph device code during sign-in
    unlistenDeviceCode = await listen<{ message: string; user_code: string; verification_uri: string }>(
      'graph-device-code',
      (e) => {
        deviceCodeMsg = e.payload.message;
      }
    );
    // Load hotkey config
    try {
      const config = await api.getHotkeys();
      openHotkey = config.open;
      saveCloseHotkey = config.saveClose;
    } catch (e) {
      console.error('Failed to load hotkeys:', e);
    }
    // Load Graph client ID
    try {
      graphClientId = await api.getGraphClientId();
    } catch (e) {
      console.error('Failed to load Graph client ID:', e);
    }
    await refreshAll();
    loadStatistics();
    try {
      contactList = await api.listContacts();
      coworkerList = await api.listCoworkers();
    } catch (e) {
      console.error('Failed to load contacts/coworkers:', e);
    }
    newNoteAndFocus();
  });

  onDestroy(() => {
    window.removeEventListener('select-search-result', handleSearchResult);
    unlistenSaveClose?.();
    unlistenDeviceCode?.();
    unlistenQuickCapture?.();
  });

  function toggleTagFilter(tagName: string) {
    activeTagFilter.update((v) => (v === tagName ? null : tagName));
  }

  function toggleCategoryFilter(catId: number) {
    activeCategoryFilter.update((v) => (v === catId ? null : catId));
  }

  function toggleContactFilter(contactId: number) {
    activeContactFilter.update((v) => (v === contactId ? null : contactId));
  }

  function toggleCoworkerFilter(coworkerId: number) {
    activeCoworkerFilter.update((v) => (v === coworkerId ? null : coworkerId));
  }

  function clearFilters() {
    activeTagFilter.set(null);
    activeCategoryFilter.set(null);
    activeContactFilter.set(null);
    activeCoworkerFilter.set(null);
    dateFrom.set(null);
    dateTo.set(null);
  }

  async function handleHide() {
    try {
      await api.hideWidget();
    } catch (e) {
      console.error('Failed to hide widget:', e);
    }
  }

  async function signInGraph() {
    deviceCodeMsg = '';
    try {
      const ok = await api.graphSignIn();
      if (ok) {
        settings.update((s) => ({ ...s, graphSignedIn: true }));
        deviceCodeMsg = '';
      }
    } catch (e: any) {
      deviceCodeMsg = e?.message ?? String(e);
    }
  }

  async function saveGraphClientId() {
    try {
      await api.setGraphClientId(graphClientId);
      graphClientIdSaved = 'Client ID saved';
      setTimeout(() => (graphClientIdSaved = ''), 1500);
    } catch (e: any) {
      deviceCodeMsg = e?.message ?? String(e);
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

  async function saveOpenHotkey() {
    hotkeyError = '';
    try {
      await api.setOpenHotkey(openHotkey);
      hotkeySaved = 'Open hotkey saved';
      setTimeout(() => (hotkeySaved = ''), 1500);
    } catch (e: any) {
      hotkeyError = e?.message ?? String(e);
    }
  }

  async function saveSaveCloseHotkey() {
    hotkeyError = '';
    try {
      await api.setSaveCloseHotkey(saveCloseHotkey);
      hotkeySaved = 'Save&close hotkey saved';
      setTimeout(() => (hotkeySaved = ''), 1500);
    } catch (e: any) {
      hotkeyError = e?.message ?? String(e);
    }
  }

  async function handleBackupDatabase() {
    try {
      const path = await saveDialog({
        defaultPath: 'masternote-backup.db',
        filters: [{ name: 'SQLite Database', extensions: ['db'] }]
      });
      if (path) {
        await api.backupDatabase(path);
      }
    } catch (e: any) {
      console.error('Backup failed:', e);
    }
  }

  async function handleRestoreDatabase() {
    try {
      const path = await openDialog({
        filters: [{ name: 'SQLite Database', extensions: ['db'] }]
      });
      if (path) {
        await api.restoreDatabase(path);
        await refreshAll();
        loadStatistics();
      }
    } catch (e: any) {
      console.error('Restore failed:', e);
    }
  }

  async function handleExportNotes() {
    try {
      const path = await saveDialog({
        defaultPath: 'masternote-notes.json',
        filters: [
          { name: 'JSON', extensions: ['json'] },
          { name: 'CSV', extensions: ['csv'] }
        ]
      });
      if (path) {
        const format = path.endsWith('.csv') ? 'csv' : 'json';
        await api.exportNotesToFile(path, format);
      }
    } catch (e: any) {
      console.error('Export notes failed:', e);
    }
  }

  async function handleExportVcard() {
    try {
      const path = await saveDialog({
        defaultPath: 'contacts.vcf',
        filters: [{ name: 'vCard', extensions: ['vcf'] }]
      });
      if (path) {
        await api.exportContactsVcard(path);
      }
    } catch (e: any) {
      console.error('vCard export failed:', e);
    }
  }

  let hasFilters = $derived(
    $activeTagFilter !== null ||
    $activeCategoryFilter !== null ||
    $activeContactFilter !== null ||
    $activeCoworkerFilter !== null ||
    $dateFrom !== null ||
    $dateTo !== null
  );
</script>

<svelte:window
  on:keydown={(e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
      e.preventDefault();
      newNoteAndFocus();
    }
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === '?') {
      e.preventDefault();
      showShortcutHelp = !showShortcutHelp;
    }
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'Q') {
      e.preventDefault();
      showQuickCapture = true;
    }
    if (e.key === 'Escape') {
      // Don't hide if a dialog or search dropdown is open
      const isDialogOpen = showSettings || showCategoryDialog || showContactDialog ||
        showCoworkerDialog || showShortcutHelp || showQuickCapture ||
        $showFilterDropdown;
      if (!isDialogOpen && !$searchQuery) {
        handleHide();
      }
    }
  }}
/>

<div class="flex flex-col h-screen bg-bg rounded-lg overflow-hidden border border-border shadow-2xl">
  <!-- Title bar / drag region -->
  <div data-tauri-drag-region class="flex items-center gap-2 px-3 py-1.5 bg-bg-subtle border-b border-border select-none">
    <img src="/logo.png" alt="MasterNote" class="w-4 h-4 shrink-0" draggable="false" />
    <span data-tauri-drag-region class="text-xs font-semibold text-fg flex-1">MasterNote</span>
    <button onclick={() => (showSidebar = !showSidebar)} class="text-fg-muted hover:text-fg text-xs px-1" title="Toggle sidebar">
      ☰
    </button>
    <button onclick={() => (showCategoryDialog = true)} class="text-fg-muted hover:text-fg text-xs px-1" title="Manage categories">
      📁
    </button>
    <button onclick={() => (showContactDialog = true)} class="text-fg-muted hover:text-fg text-xs px-1" title="Manage contacts">
      👤
    </button>
    <button onclick={() => (showCoworkerDialog = true)} class="text-fg-muted hover:text-fg text-xs px-1" title="Manage coworkers">
      🤝
    </button>
    <button onclick={() => (showQuickCapture = true)} class="text-fg-muted hover:text-fg text-xs px-1" title="Quick capture (Ctrl+Shift+Q)">
      ⚡
    </button>
    <button onclick={() => (showShortcutHelp = true)} class="text-fg-muted hover:text-fg text-xs px-1" title="Keyboard shortcuts (Ctrl+Shift+?)">
      ?
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
    <div class="flex flex-col gap-2 px-3 py-2 border-b border-border bg-bg-subtle text-xs max-h-80 overflow-y-auto">
      <!-- Theme -->
      <label class="flex items-center gap-2">
        <span class="text-fg-muted w-28">Theme</span>
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

      <!-- Hide on blur -->
      <label class="flex items-center gap-2">
        <input
          type="checkbox"
          checked={$settings.hideOnBlur}
          onchange={(e) => settings.update((s) => ({ ...s, hideOnBlur: (e.target as HTMLInputElement).checked }))}
          class="accent-accent"
        />
        <span class="text-fg-muted">Hide widget when clicking outside</span>
      </label>

      <!-- Autosave interval -->
      <label class="flex items-center gap-2">
        <span class="text-fg-muted w-28">Autosave (ms)</span>
        <input
          type="number"
          min="0"
          max="10000"
          step="100"
          value={$settings.autosaveInterval}
          onchange={(e) => settings.update((s) => ({ ...s, autosaveInterval: parseInt((e.target as HTMLInputElement).value) || 0 }))}
          class="bg-bg-muted rounded px-2 py-1 border border-border w-20"
        />
        <span class="text-[10px] text-fg-muted">0 = instant</span>
      </label>

      <!-- Divider -->
      <div class="border-t border-border my-1"></div>

      <!-- Hotkeys -->
      <div class="text-fg-muted font-medium mb-1">Keyboard Shortcuts</div>

      <div class="flex items-center gap-2">
        <span class="text-fg-muted w-28">Open / toggle</span>
        <input
          bind:value={openHotkey}
          placeholder="e.g. Ctrl+Shift+M"
          class="flex-1 bg-bg-muted rounded px-2 py-1 border border-border outline-none font-mono text-[11px]"
        />
        <button onclick={saveOpenHotkey} class="px-2 py-1 rounded bg-accent text-accent-fg hover:opacity-90">
          Apply
        </button>
      </div>

      <div class="flex items-center gap-2">
        <span class="text-fg-muted w-28">Save & close</span>
        <input
          bind:value={saveCloseHotkey}
          placeholder="e.g. Ctrl+Shift+N"
          class="flex-1 bg-bg-muted rounded px-2 py-1 border border-border outline-none font-mono text-[11px]"
        />
        <button onclick={saveSaveCloseHotkey} class="px-2 py-1 rounded bg-accent text-accent-fg hover:opacity-90">
          Apply
        </button>
      </div>

      <div class="text-[10px] text-fg-muted pl-28">
        Format: Ctrl+Shift+Key, Alt+Key, etc. Changes apply immediately.
      </div>

      {#if hotkeyError}
        <div class="text-[10px] text-red-500 pl-28">{hotkeyError}</div>
      {/if}
      {#if hotkeySaved}
        <div class="text-[10px] text-green-500 pl-28">{hotkeySaved}</div>
      {/if}

      <!-- Divider -->
      <div class="border-t border-border my-1"></div>

      <!-- Outlook -->
      <div class="text-fg-muted font-medium mb-1">Outlook Integration</div>
      <div class="flex items-center gap-2">
        <span class="text-fg-muted w-28">Connection</span>
        {#if $settings.graphSignedIn}
          <button onclick={signOutGraph} class="px-2 py-1 rounded bg-bg-muted hover:bg-border">Sign out</button>
          <span class="text-green-500">✓ Connected</span>
        {:else}
          <button onclick={signInGraph} class="px-2 py-1 rounded bg-accent text-accent-fg hover:opacity-90">Sign in</button>
        {/if}
      </div>
      <div class="flex items-center gap-2">
        <span class="text-fg-muted w-28">Client ID</span>
        <input
          bind:value={graphClientId}
          placeholder="Azure app client ID"
          class="flex-1 text-xs bg-bg-muted rounded px-2 py-1 border border-border outline-none"
        />
        <button onclick={saveGraphClientId} class="text-xs px-2 py-1 rounded bg-bg-muted hover:bg-border">Save</button>
      </div>
      {#if graphClientIdSaved}
        <div class="text-[10px] text-green-500 pl-28">{graphClientIdSaved}</div>
      {/if}
      {#if deviceCodeMsg}
        <div class="text-[10px] text-blue-500 pl-28 whitespace-pre-wrap">{deviceCodeMsg}</div>
      {/if}

      <!-- Divider -->
      <div class="border-t border-border my-1"></div>

      <!-- Data management -->
      <div class="text-fg-muted font-medium mb-1">Data Management</div>
      <div class="flex items-center gap-2 flex-wrap">
        <button onclick={handleBackupDatabase} class="px-2 py-1 rounded bg-bg-muted hover:bg-border">Backup DB</button>
        <button onclick={handleRestoreDatabase} class="px-2 py-1 rounded bg-bg-muted hover:bg-border">Restore DB</button>
        <button onclick={handleExportNotes} class="px-2 py-1 rounded bg-bg-muted hover:bg-border">Export Notes</button>
        <button onclick={handleExportVcard} class="px-2 py-1 rounded bg-bg-muted hover:bg-border">Export vCard</button>
      </div>

      <!-- Divider -->
      <div class="border-t border-border my-1"></div>

      <!-- Statistics -->
      {#if statistics}
        <div class="text-fg-muted font-medium mb-1">Statistics</div>
        <div class="grid grid-cols-2 gap-1 text-[10px]">
          <div>Total notes: <span class="text-fg">{statistics.totalNotes}</span></div>
          <div>Archived: <span class="text-fg">{statistics.archivedNotes}</span></div>
          <div>With reminders: <span class="text-fg">{statistics.notesWithReminders}</span></div>
          <div>This week: <span class="text-fg">{statistics.notesThisWeek}</span></div>
          <div>Contacts: <span class="text-fg">{statistics.totalContacts}</span></div>
          <div>Coworkers: <span class="text-fg">{statistics.totalCoworkers}</span></div>
        </div>
        {#if statistics.notesPerCategory.length > 0}
          <div class="mt-1">
            {#each statistics.notesPerCategory as cat}
              <div class="text-[10px] text-fg-muted">{cat.name}: {cat.count}</div>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  <!-- Filter button + dropdown -->
  <div class="relative px-3 py-1 border-b border-border bg-bg-subtle">
    <button
      onclick={() => showFilterDropdown.update((v) => !v)}
      class="text-[10px] px-2 py-0.5 rounded-full bg-bg-muted hover:bg-border transition flex items-center gap-1"
    >
      🔍 Filter
      {#if hasFilters}
        <span class="text-accent">●</span>
      {/if}
    </button>
    {#if hasFilters}
      <button onclick={clearFilters} class="text-[10px] px-1.5 text-fg-muted hover:text-fg ml-1">clear</button>
    {/if}

    <label class="text-[10px] text-fg-muted ml-2 inline-flex items-center gap-1">
      <input type="checkbox" checked={$showArchived} onchange={(e) => showArchived.set((e.target as HTMLInputElement).checked)} class="accent-accent" />
      Show archived
    </label>

    {#if $showFilterDropdown}
      <div class="absolute top-full left-3 mt-1 bg-bg rounded-lg border border-border shadow-lg p-3 z-50 w-72 max-h-72 overflow-y-auto">
        <!-- Time range -->
        <div class="mb-2">
          <div class="text-[10px] text-fg-muted mb-1 font-medium">Sort by time</div>
          <div class="flex gap-1 flex-wrap">
            {#each ['newest', 'oldest', 'today', 'week'] as option}
              <button
                onclick={() => timeRangeSort.set(option as any)}
                class="text-[10px] px-2 py-0.5 rounded-full {$timeRangeSort === option ? 'bg-accent text-accent-fg' : 'bg-bg-muted hover:bg-border'}"
              >
                {option === 'newest' ? 'Newest' : option === 'oldest' ? 'Oldest' : option === 'today' ? 'Today' : 'This week'}
              </button>
            {/each}
          </div>
        </div>

        {#if $categories.length > 0}
          <div class="mb-2">
            <div class="text-[10px] text-fg-muted mb-1 font-medium">Categories</div>
            <div class="flex gap-1 flex-wrap">
              {#each $categories as cat}
                <button
                  onclick={() => toggleCategoryFilter(cat.id)}
                  class="text-[10px] px-1.5 py-0.5 rounded-full whitespace-nowrap transition {$activeCategoryFilter === cat.id ? 'text-white' : 'opacity-60 hover:opacity-100'}"
                  style="background: {cat.color}"
                >
                  {cat.name}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        {#if $tags.length > 0}
          <div class="mb-2">
            <div class="text-[10px] text-fg-muted mb-1 font-medium">Tags</div>
            <div class="flex gap-1 flex-wrap">
              {#each $tags as tag}
                <button
                  onclick={() => toggleTagFilter(tag.name)}
                  class="text-[10px] px-1.5 py-0.5 rounded-full whitespace-nowrap transition {$activeTagFilter === tag.name ? 'bg-accent text-accent-fg' : 'bg-bg-muted text-fg-muted hover:bg-border'}"
                >
                  #{tag.name}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        {#if contactList.length > 0}
          <div class="mb-2">
            <div class="text-[10px] text-fg-muted mb-1 font-medium">Customers ({contactList.length})</div>
            {#if contactList.length > 8}
              <input
                bind:value={contactFilterQuery}
                placeholder="Search customers…"
                class="w-full text-[10px] bg-bg-muted rounded px-1.5 py-1 border border-border outline-none mb-1"
              />
            {/if}
            <div class="flex gap-1 flex-wrap max-h-24 overflow-y-auto">
              {#each filteredContactList as c (c.id)}
                <button
                  onclick={() => toggleContactFilter(c.id)}
                  class="text-[10px] px-1.5 py-0.5 rounded-full whitespace-nowrap transition {$activeContactFilter === c.id ? 'bg-accent text-accent-fg' : 'bg-bg-muted text-fg-muted hover:bg-border'}"
                >
                  👤 {c.kind === 'company' && c.companyName ? c.companyName : [c.firstName, c.lastName].filter(Boolean).join(' ')}
                </button>
              {/each}
              {#if filteredContactList.length === 0}
                <span class="text-[10px] text-fg-muted">No matches</span>
              {/if}
            </div>
          </div>
        {/if}

        {#if coworkerList.length > 0}
          <div class="mb-2">
            <div class="text-[10px] text-fg-muted mb-1 font-medium">Coworkers ({coworkerList.length})</div>
            {#if coworkerList.length > 8}
              <input
                bind:value={coworkerFilterQuery}
                placeholder="Search coworkers…"
                class="w-full text-[10px] bg-bg-muted rounded px-1.5 py-1 border border-border outline-none mb-1"
              />
            {/if}
            <div class="flex gap-1 flex-wrap max-h-24 overflow-y-auto">
              {#each filteredCoworkerList as c (c.id)}
                <button
                  onclick={() => toggleCoworkerFilter(c.id)}
                  class="text-[10px] px-1.5 py-0.5 rounded-full whitespace-nowrap transition {$activeCoworkerFilter === c.id ? 'bg-accent text-accent-fg' : 'bg-bg-muted text-fg-muted hover:bg-border'}"
                >
                  🤝 {[c.firstName, c.lastName].filter(Boolean).join(' ')}
                </button>
              {/each}
              {#if filteredCoworkerList.length === 0}
                <span class="text-[10px] text-fg-muted">No matches</span>
              {/if}
            </div>
          </div>
        {/if}

        <div class="mb-2">
          <div class="text-[10px] text-fg-muted mb-1 font-medium">Date range</div>
          <div class="flex items-center gap-1">
            <input
              type="date"
              value={$dateFrom ?? ''}
              onchange={(e) => dateFrom.set((e.target as HTMLInputElement).value || null)}
              class="text-[10px] bg-bg-muted rounded px-1.5 py-1 border border-border outline-none flex-1"
            />
            <span class="text-fg-muted text-[10px]">to</span>
            <input
              type="date"
              value={$dateTo ?? ''}
              onchange={(e) => dateTo.set((e.target as HTMLInputElement).value || null)}
              class="text-[10px] bg-bg-muted rounded px-1.5 py-1 border border-border outline-none flex-1"
            />
          </div>
          {#if $dateFrom || $dateTo}
            <button
              onclick={() => { dateFrom.set(null); dateTo.set(null); }}
              class="text-[10px] text-fg-muted hover:text-fg mt-1"
            >clear dates</button>
          {/if}
        </div>
      </div>
    {/if}
  </div>

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

{#if showContactDialog}
  <ContactDialog onClose={() => (showContactDialog = false)} />
{/if}

{#if showCoworkerDialog}
  <CoworkerDialog onClose={() => (showCoworkerDialog = false)} />
{/if}

{#if showShortcutHelp}
  <ShortcutHelp onClose={() => (showShortcutHelp = false)} />
{/if}

{#if showQuickCapture}
  <QuickCapture onClose={() => (showQuickCapture = false)} onSaved={() => { loadNotes(); loadStatistics(); }} />
{/if}
