<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable, get } from 'svelte/store';
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
  import Toast from '$lib/components/ui/Toast.svelte';
  import Toggle from '$lib/components/ui/Toggle.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import HotkeyInput from '$lib/components/ui/HotkeyInput.svelte';
  import { showToast } from '$lib/stores/toast';
  import {
    PanelLeft, FolderPlus, UserPlus, Users, Zap, Keyboard, Settings, X,
    Database, FileDown, FileUp, Download, ShieldCheck, LogOut, LogIn, AlertCircle, Filter
  } from '@lucide/svelte';
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
  import { t, localeLabels, type Locale } from '$lib/i18n';
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
  let showCommandPalette = $state(false);
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
  let quickCaptureHotkey = $state('Ctrl+Shift+Q');
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
      quickCaptureHotkey = config.quickCapture;
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
      graphClientIdSaved = get(t)('settings.clientIdSaved');
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

  function setLanguage(lang: string) {
    settings.update((s) => ({ ...s, language: lang as Locale }));
  }

  async function saveHotkey(which: 'open' | 'saveClose' | 'quickCapture', value: string) {
    hotkeyError = '';
    try {
      if (which === 'open') await api.setOpenHotkey(value);
      else if (which === 'saveClose') await api.setSaveCloseHotkey(value);
      else await api.setQuickCaptureHotkey(value);
      hotkeySaved = value ? get(t)('settings.shortcutSaved') : get(t)('settings.shortcutDisabled');
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
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      showCommandPalette = !showCommandPalette;
    }
    if (e.key === 'Escape') {
      // Don't hide if a dialog or search dropdown is open
      const isDialogOpen = showSettings || showCategoryDialog || showContactDialog ||
        showCoworkerDialog || showShortcutHelp || showQuickCapture || showCommandPalette ||
        $showFilterDropdown;
      if (!isDialogOpen && !$searchQuery) {
        handleHide();
      }
    }
  }}
/>

<div class="flex flex-col h-screen bg-bg rounded-lg overflow-hidden border border-border shadow-xl">
  <!-- Title bar / drag region -->
  <div data-tauri-drag-region class="flex items-center gap-2 px-3 py-2 bg-gradient-to-r from-accent/5 to-transparent border-b border-border select-none">
    <img src="/logo.png" alt="MasterNote" class="w-5 h-5 shrink-0 rounded" draggable="false" />
    <span data-tauri-drag-region class="text-sm font-bold text-fg flex-1 tracking-tight">MasterNote</span>

    <!-- Navigation group -->
    <div class="flex items-center gap-0.5">
      <button onclick={() => (showSidebar = !showSidebar)} class="text-fg-muted hover:text-fg hover:bg-accent-soft p-1.5 rounded-lg transition" title={$t('titlebar.sidebar')}>
        <PanelLeft class="w-3.5 h-3.5" />
      </button>
      <button onclick={() => (showQuickCapture = true)} class="text-fg-muted hover:text-fg hover:bg-accent-soft p-1.5 rounded-lg transition" title={$t('titlebar.quickCapture')}>
        <Zap class="w-3.5 h-3.5" />
      </button>
    </div>

    <div class="w-px h-4 bg-border"></div>

    <!-- Manage group -->
    <div class="flex items-center gap-0.5">
      <button onclick={() => (showCategoryDialog = true)} class="text-fg-muted hover:text-fg hover:bg-accent-soft p-1.5 rounded-lg transition" title={$t('titlebar.categories')}>
        <FolderPlus class="w-3.5 h-3.5" />
      </button>
      <button onclick={() => (showContactDialog = true)} class="text-fg-muted hover:text-fg hover:bg-accent-soft p-1.5 rounded-lg transition" title={$t('titlebar.contacts')}>
        <UserPlus class="w-3.5 h-3.5" />
      </button>
      <button onclick={() => (showCoworkerDialog = true)} class="text-fg-muted hover:text-fg hover:bg-accent-soft p-1.5 rounded-lg transition" title={$t('titlebar.coworkers')}>
        <Users class="w-3.5 h-3.5" />
      </button>
    </div>

    <div class="w-px h-4 bg-border"></div>

    <!-- System group -->
    <div class="flex items-center gap-0.5">
      <button onclick={() => (showShortcutHelp = true)} class="text-fg-muted hover:text-fg hover:bg-accent-soft p-1.5 rounded-lg transition" title={$t('titlebar.shortcuts')}>
        <Keyboard class="w-3.5 h-3.5" />
      </button>
      <button onclick={() => (showSettings = !showSettings)} class="text-fg-muted hover:text-fg hover:bg-accent-soft p-1.5 rounded-lg transition" title={$t('titlebar.settings')}>
        <Settings class="w-3.5 h-3.5" />
      </button>
      <button onclick={handleHide} class="text-fg-muted hover:text-danger hover:bg-danger-soft p-1.5 rounded-lg transition" title={$t('titlebar.hide')}>
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>

  <SearchBar bind:this={searchBar} />

  {#if $lastError}
    <div class="px-3 py-1.5 text-xs text-danger bg-danger-soft border-b border-danger/20 flex items-center gap-2">
      <AlertCircle class="w-3.5 h-3.5 shrink-0" />
      <span class="flex-1">{$lastError}</span>
      <button onclick={() => lastError.set(null)} class="text-danger hover:opacity-70"><X class="w-3 h-3" /></button>
    </div>
  {/if}

  {#if showSettings}
    <div class="flex flex-col gap-3 px-3 py-3 border-b border-border bg-bg-subtle text-xs max-h-96 overflow-y-auto animate-slide-down">
      <!-- Appearance section -->
      <div>
        <div class="flex items-center gap-1.5 text-fg-muted font-semibold mb-1.5">
          <Settings class="w-3 h-3" />
          {$t('settings.appearance')}
        </div>
        <div class="flex items-center gap-2 mb-1.5">
          <span class="text-fg-muted w-28">{$t('settings.theme')}</span>
          <select
            value={$settings.theme}
            onchange={(e) => settings.update((s) => ({ ...s, theme: (e.target as HTMLSelectElement).value as any }))}
            class="bg-bg-muted rounded-lg px-2 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20"
          >
            <option value="system">{$t('settings.system')}</option>
            <option value="light">{$t('settings.light')}</option>
            <option value="dark">{$t('settings.dark')}</option>
          </select>
        </div>
        <div class="flex items-center gap-2 mb-1.5">
          <span class="text-fg-muted w-28">{$t('settings.language')}</span>
          <select
            value={$settings.language}
            onchange={(e) => setLanguage((e.target as HTMLSelectElement).value)}
            class="bg-bg-muted rounded-lg px-2 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20"
          >
            {#each Object.entries(localeLabels) as [code, label]}
              <option value={code}>{label}</option>
            {/each}
          </select>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-fg-muted w-28">{$t('settings.hideOnBlur')}</span>
          <Toggle checked={$settings.hideOnBlur} onchange={() => settings.update((s) => ({ ...s, hideOnBlur: !s.hideOnBlur }))} label={$t('settings.hideOnBlurHint')} />
          <span class="text-fg-muted">{$t('settings.hideOnBlurHint')}</span>
        </div>
      </div>

      <div class="border-t border-border"></div>

      <!-- Behavior section -->
      <div>
        <div class="flex items-center gap-1.5 text-fg-muted font-semibold mb-1.5">
          <Zap class="w-3 h-3" />
          {$t('settings.behavior')}
        </div>
        <div class="flex items-center gap-2">
          <span class="text-fg-muted w-28">{$t('settings.autosave')}</span>
          <input
            type="number"
            min="0"
            max="10000"
            step="100"
            value={$settings.autosaveInterval}
            onchange={(e) => settings.update((s) => ({ ...s, autosaveInterval: parseInt((e.target as HTMLInputElement).value) || 0 }))}
            class="bg-bg-muted rounded-lg px-2 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20 w-20"
          />
          <span class="text-[10px] text-fg-muted">{$t('settings.instant')}</span>
        </div>
      </div>

      <div class="border-t border-border"></div>

      <!-- Keyboard shortcuts -->
      <div>
        <div class="flex items-center gap-1.5 text-fg-muted font-semibold mb-1.5">
          <Keyboard class="w-3 h-3" />
          {$t('settings.shortcuts')}
        </div>
        <div class="flex items-center gap-2 mb-1.5">
          <span class="text-fg-muted w-28">{$t('settings.openToggle')}</span>
          <HotkeyInput bind:value={openHotkey} onApply={(v) => saveHotkey('open', v)} />
          <button
            onclick={() => { openHotkey = ''; saveHotkey('open', ''); }}
            class="px-2 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition text-[10px]"
            title={$t('settings.disableShortcut')}
          >{$t('common.none')}</button>
        </div>
        <div class="flex items-center gap-2 mb-1.5">
          <span class="text-fg-muted w-28">{$t('settings.saveClose')}</span>
          <HotkeyInput bind:value={saveCloseHotkey} onApply={(v) => saveHotkey('saveClose', v)} />
          <button
            onclick={() => { saveCloseHotkey = ''; saveHotkey('saveClose', ''); }}
            class="px-2 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition text-[10px]"
            title={$t('settings.disableShortcut')}
          >{$t('common.none')}</button>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-fg-muted w-28">{$t('settings.quickCapture')}</span>
          <HotkeyInput bind:value={quickCaptureHotkey} onApply={(v) => saveHotkey('quickCapture', v)} />
          <button
            onclick={() => { quickCaptureHotkey = ''; saveHotkey('quickCapture', ''); }}
            class="px-2 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition text-[10px]"
            title={$t('settings.disableShortcut')}
          >{$t('common.none')}</button>
        </div>
        <div class="text-[10px] text-fg-muted mt-1">
          {$t('settings.shortcutHint')}
        </div>
        {#if hotkeyError}
          <div class="text-[10px] text-danger mt-1">{hotkeyError}</div>
        {/if}
        {#if hotkeySaved}
          <div class="text-[10px] text-success mt-1">{hotkeySaved}</div>
        {/if}
      </div>

      <div class="border-t border-border"></div>

      <!-- Outlook integration -->
      <div>
        <div class="flex items-center gap-1.5 text-fg-muted font-semibold mb-1.5">
          <ShieldCheck class="w-3 h-3" />
          {$t('settings.outlook')}
        </div>
        <div class="flex items-center gap-2 mb-1.5">
          <span class="text-fg-muted w-28">{$t('settings.connection')}</span>
          {#if $settings.graphSignedIn}
            <button onclick={signOutGraph} class="px-2.5 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1">
              <LogOut class="w-3 h-3" />
              {$t('settings.signOut')}
            </button>
            <span class="text-success inline-flex items-center gap-1">
              <ShieldCheck class="w-3 h-3" />
              {$t('settings.connected')}
            </span>
          {:else}
            <button onclick={signInGraph} class="px-2.5 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition inline-flex items-center gap-1">
              <LogIn class="w-3 h-3" />
              {$t('settings.signIn')}
            </button>
          {/if}
        </div>
        <div class="flex items-center gap-2">
          <span class="text-fg-muted w-28">{$t('settings.clientId')}</span>
          <input
            bind:value={graphClientId}
            placeholder={$t('settings.clientIdPlaceholder')}
            class="flex-1 bg-bg-muted rounded-lg px-2 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20"
          />
          <button onclick={saveGraphClientId} class="px-2.5 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.save')}</button>
        </div>
        {#if graphClientIdSaved}
          <div class="text-[10px] text-success mt-1">{graphClientIdSaved}</div>
        {/if}
        {#if deviceCodeMsg}
          <div class="text-[10px] text-info mt-1 whitespace-pre-wrap">{deviceCodeMsg}</div>
        {/if}
      </div>

      <div class="border-t border-border"></div>

      <!-- Data management -->
      <div>
        <div class="flex items-center gap-1.5 text-fg-muted font-semibold mb-1.5">
          <Database class="w-3 h-3" />
          {$t('settings.data')}
        </div>
        <div class="flex items-center gap-1.5 flex-wrap">
          <button onclick={handleBackupDatabase} class="px-2.5 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1">
            <FileDown class="w-3 h-3" />
            {$t('settings.backup')}
          </button>
          <button onclick={handleRestoreDatabase} class="px-2.5 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1">
            <FileUp class="w-3 h-3" />
            {$t('settings.restore')}
          </button>
          <button onclick={handleExportNotes} class="px-2.5 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1">
            <Download class="w-3 h-3" />
            {$t('settings.exportNotes')}
          </button>
          <button onclick={handleExportVcard} class="px-2.5 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1">
            <Download class="w-3 h-3" />
            {$t('settings.exportVcard')}
          </button>
        </div>
      </div>

      <!-- Statistics -->
      {#if statistics}
        <div class="border-t border-border"></div>
        <div>
          <div class="flex items-center gap-1.5 text-fg-muted font-semibold mb-1.5">
            <Database class="w-3 h-3" />
            {$t('settings.statistics')}
          </div>
          <div class="grid grid-cols-3 gap-1.5">
            <div class="bg-bg-muted rounded-lg px-2 py-1.5 text-center">
              <div class="text-sm font-bold text-fg">{statistics.totalNotes}</div>
              <div class="text-[9px] text-fg-muted">{$t('settings.totalNotes')}</div>
            </div>
            <div class="bg-bg-muted rounded-lg px-2 py-1.5 text-center">
              <div class="text-sm font-bold text-fg">{statistics.archivedNotes}</div>
              <div class="text-[9px] text-fg-muted">{$t('settings.archived')}</div>
            </div>
            <div class="bg-bg-muted rounded-lg px-2 py-1.5 text-center">
              <div class="text-sm font-bold text-fg">{statistics.notesWithReminders}</div>
              <div class="text-[9px] text-fg-muted">{$t('settings.withReminders')}</div>
            </div>
            <div class="bg-bg-muted rounded-lg px-2 py-1.5 text-center">
              <div class="text-sm font-bold text-fg">{statistics.notesThisWeek}</div>
              <div class="text-[9px] text-fg-muted">{$t('settings.thisWeek')}</div>
            </div>
            <div class="bg-bg-muted rounded-lg px-2 py-1.5 text-center">
              <div class="text-sm font-bold text-fg">{statistics.totalContacts}</div>
              <div class="text-[9px] text-fg-muted">{$t('settings.contacts')}</div>
            </div>
            <div class="bg-bg-muted rounded-lg px-2 py-1.5 text-center">
              <div class="text-sm font-bold text-fg">{statistics.totalCoworkers}</div>
              <div class="text-[9px] text-fg-muted">{$t('settings.coworkers')}</div>
            </div>
          </div>
          {#if statistics.notesPerCategory.length > 0}
            <div class="mt-2 space-y-1">
              {#each statistics.notesPerCategory as cat}
                <div class="flex items-center gap-1.5">
                  <span class="w-2 h-2 rounded-full shrink-0" style="background: {cat.color}"></span>
                  <span class="text-[10px] text-fg-muted flex-1">{cat.name}</span>
                  <div class="flex-1 h-1.5 bg-bg-muted rounded-full overflow-hidden">
                    <div class="h-full rounded-full" style="width: {Math.min(100, (cat.count / Math.max(1, statistics.totalNotes)) * 100)}%; background: {cat.color}"></div>
                  </div>
                  <span class="text-[10px] text-fg font-medium w-6 text-right">{cat.count}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Filter button + dropdown -->
  <div class="relative px-3 py-1.5 border-b border-border bg-bg-subtle">
    <button
      onclick={() => showFilterDropdown.update((v) => !v)}
      class="text-[10px] px-2 py-1 rounded-lg bg-bg-muted hover:bg-border transition flex items-center gap-1 font-medium"
    >
      <Filter class="w-3 h-3" />
      {$t('filter.filters')}
      {#if hasFilters}
        <span class="text-accent text-[8px]">●</span>
      {/if}
    </button>
    {#if hasFilters}
      <button onclick={clearFilters} class="text-[10px] px-1.5 text-fg-muted hover:text-fg ml-1">{$t('common.clear')}</button>
    {/if}

    <label class="text-[10px] text-fg-muted ml-2 inline-flex items-center gap-1 cursor-pointer">
      <input type="checkbox" checked={$showArchived} onchange={(e) => showArchived.set((e.target as HTMLInputElement).checked)} class="accent-accent w-3 h-3" />
      {$t('filter.showArchived')}
    </label>

    {#if $showFilterDropdown}
      <div class="absolute top-full left-3 mt-1 bg-surface-1 rounded-xl border border-border shadow-lg p-3 z-50 w-72 max-h-72 overflow-y-auto animate-slide-down">
        <!-- Time range -->
        <div class="mb-2">
          <div class="text-[10px] text-fg-muted mb-1 font-medium">{$t('filter.sortTime')}</div>
          <div class="flex gap-1 flex-wrap">
            {#each ['newest', 'oldest', 'today', 'week'] as option}
              <button
                onclick={() => timeRangeSort.set(option as any)}
                class="text-[10px] px-2 py-0.5 rounded-full {$timeRangeSort === option ? 'bg-accent text-accent-fg' : 'bg-bg-muted hover:bg-border'}"
              >
                {$t('filter.' + option)}
              </button>
            {/each}
          </div>
        </div>

        {#if $categories.length > 0}
          <div class="mb-2">
            <div class="text-[10px] text-fg-muted mb-1 font-medium">{$t('filter.categories')}</div>
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
            <div class="text-[10px] text-fg-muted mb-1 font-medium">{$t('filter.tags')}</div>
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
            <div class="text-[10px] text-fg-muted mb-1 font-medium">{$t('filter.customers')} ({contactList.length})</div>
            {#if contactList.length > 8}
              <input
                bind:value={contactFilterQuery}
                placeholder={$t('filter.searchCustomers')}
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
                <span class="text-[10px] text-fg-muted">{$t('common.noMatches')}</span>
              {/if}
            </div>
          </div>
        {/if}

        {#if coworkerList.length > 0}
          <div class="mb-2">
            <div class="text-[10px] text-fg-muted mb-1 font-medium">{$t('filter.coworkers')} ({coworkerList.length})</div>
            {#if coworkerList.length > 8}
              <input
                bind:value={coworkerFilterQuery}
                placeholder={$t('filter.searchCoworkers')}
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
                <span class="text-[10px] text-fg-muted">{$t('common.noMatches')}</span>
              {/if}
            </div>
          </div>
        {/if}

        <div class="mb-2">
          <div class="text-[10px] text-fg-muted mb-1 font-medium">{$t('filter.dateRange')}</div>
          <div class="flex items-center gap-1">
            <input
              type="date"
              value={$dateFrom ?? ''}
              onchange={(e) => dateFrom.set((e.target as HTMLInputElement).value || null)}
              class="text-[10px] bg-bg-muted rounded px-1.5 py-1 border border-border outline-none flex-1"
            />
            <span class="text-fg-muted text-[10px]">{$t('filter.to')}</span>
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
            >{$t('filter.clearDates')}</button>
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

{#if showCommandPalette}
  <CommandPalette
    onClose={() => (showCommandPalette = false)}
    onNewNote={() => { showCommandPalette = false; newNoteAndFocus(); }}
    onToggleSidebar={() => { showCommandPalette = false; showSidebar = !showSidebar; }}
    onOpenSettings={() => { showCommandPalette = false; showSettings = true; }}
    onOpenCategories={() => { showCommandPalette = false; showCategoryDialog = true; }}
    onOpenContacts={() => { showCommandPalette = false; showContactDialog = true; }}
    onOpenCoworkers={() => { showCommandPalette = false; showCoworkerDialog = true; }}
    onQuickCapture={() => { showCommandPalette = false; showQuickCapture = true; }}
    onShowShortcuts={() => { showCommandPalette = false; showShortcutHelp = true; }}
    onToggleTheme={() => { showCommandPalette = false; settings.update((s) => ({ ...s, theme: s.theme === 'dark' ? 'light' : 'dark' })); }}
  />
{/if}

<Toast />
