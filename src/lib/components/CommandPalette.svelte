<script lang="ts">
  import { Search, FilePlus, PanelLeft, Settings, FolderPlus, UserPlus, Users, Zap, Keyboard, Archive, Sun, Moon, Monitor } from '@lucide/svelte';
  import { t } from '$lib/i18n';

  interface Command {
    id: string;
    title: string;
    subtitle?: string;
    icon: any;
    action: () => void;
    keywords: string[];
  }

  let {
    onClose,
    onNewNote,
    onToggleSidebar,
    onOpenSettings,
    onOpenCategories,
    onOpenContacts,
    onOpenCoworkers,
    onQuickCapture,
    onShowShortcuts,
    onToggleTheme
  }: {
    onClose: () => void;
    onNewNote: () => void;
    onToggleSidebar: () => void;
    onOpenSettings: () => void;
    onOpenCategories: () => void;
    onOpenContacts: () => void;
    onOpenCoworkers: () => void;
    onQuickCapture: () => void;
    onShowShortcuts: () => void;
    onToggleTheme: () => void;
  } = $props();

  let query = $state('');
  let selectedIndex = $state(0);
  let inputEl = $state<HTMLInputElement | null>(null);
  let listEl = $state<HTMLDivElement | null>(null);

  let commands = $derived<Command[]>([
    { id: 'new', title: $t('palette.new'), subtitle: 'Ctrl+N', icon: FilePlus, action: () => onNewNote(), keywords: ['new', 'note', 'create', 'neu', 'notiz', 'nouvelle', 'nueva'] },
    { id: 'search', title: $t('palette.search'), subtitle: 'Ctrl+F', icon: Search, action: () => { onClose(); document.querySelector<HTMLInputElement>('[placeholder]')?.focus(); }, keywords: ['search', 'find', 'query', 'suche', 'suchen', 'rechercher', 'buscar'] },
    { id: 'sidebar', title: $t('palette.sidebar'), icon: PanelLeft, action: () => onToggleSidebar(), keywords: ['sidebar', 'toggle', 'list', 'panel', 'seitenleiste', 'panneau'] },
    { id: 'settings', title: $t('palette.settings'), subtitle: $t('palette.settingsSub'), icon: Settings, action: () => onOpenSettings(), keywords: ['settings', 'preferences', 'config', 'einstellungen', 'paramètres', 'configuración'] },
    { id: 'categories', title: $t('palette.categories'), icon: FolderPlus, action: () => onOpenCategories(), keywords: ['category', 'categories', 'folder', 'kategorien', 'catégories', 'categorías'] },
    { id: 'contacts', title: $t('palette.contacts'), icon: UserPlus, action: () => onOpenContacts(), keywords: ['contact', 'contacts', 'customer', 'kontakte', 'kunde', 'contactos'] },
    { id: 'coworkers', title: $t('palette.coworkers'), icon: Users, action: () => onOpenCoworkers(), keywords: ['coworker', 'coworkers', 'colleague', 'kollegen', 'collègues', 'compañeros'] },
    { id: 'quick', title: $t('palette.quick'), subtitle: 'Ctrl+Shift+Q', icon: Zap, action: () => onQuickCapture(), keywords: ['quick', 'capture', 'fast', 'schnellerfassung', 'capture rapide', 'captura'] },
    { id: 'shortcuts', title: $t('palette.shortcuts'), icon: Keyboard, action: () => onShowShortcuts(), keywords: ['shortcuts', 'keys', 'hotkey', 'help', 'tastenkürzel', 'raccourcis', 'atajos'] },
    { id: 'theme', title: $t('palette.theme'), subtitle: $t('palette.themeSub'), icon: Sun, action: () => onToggleTheme(), keywords: ['theme', 'dark', 'light', 'appearance', 'farbschema', 'thème', 'tema'] },
  ]);

  let filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return commands;
    return commands.filter((c) => {
      const haystack = `${c.title} ${c.subtitle ?? ''} ${c.keywords.join(' ')}`.toLowerCase();
      return q.split(/\s+/).every((w) => haystack.includes(w));
    });
  });

  $effect(() => {
    inputEl?.focus();
    selectedIndex = 0;
  });

  $effect(() => {
    query;
    selectedIndex = 0;
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filtered.length - 1);
      scrollIntoView();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      scrollIntoView();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < filtered.length) {
        filtered[selectedIndex].action();
        onClose();
      }
    } else if (e.key === 'Escape') {
      onClose();
    }
  }

  function scrollIntoView() {
    setTimeout(() => {
      if (!listEl) return;
      const el = listEl.querySelector(`[data-idx="${selectedIndex}"]`) as HTMLElement | null;
      el?.scrollIntoView({ block: 'nearest' });
    }, 0);
  }
</script>

<svelte:window on:keydown={(e) => { if (e.key === 'Escape') onClose(); }} />

<div
  class="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-start justify-center pt-16 z-50 animate-fade-in"
  role="button"
  tabindex="-1"
  onclick={onClose}
  onkeydown={() => {}}
>
  <div
    class="bg-surface-1 rounded-2xl shadow-xl border border-border w-96 animate-scale-in"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <!-- Search input -->
    <div class="flex items-center gap-2 px-4 py-3 border-b border-border">
      <Search class="w-4 h-4 text-fg-muted shrink-0" />
      <input
        bind:this={inputEl}
        bind:value={query}
        onkeydown={handleKeydown}
        placeholder={$t('palette.placeholder')}
        class="flex-1 bg-transparent text-sm outline-none placeholder:text-fg-muted"
      />
      <kbd class="text-[10px] text-fg-muted bg-bg-muted px-1.5 py-0.5 rounded-md border border-border font-mono">Esc</kbd>
    </div>

    <!-- Results -->
    <div bind:this={listEl} class="max-h-64 overflow-y-auto">
      {#if filtered.length === 0}
        <div class="px-4 py-3 text-xs text-fg-muted">{$t('palette.empty')}</div>
      {:else}
        {#each filtered as cmd, i (cmd.id)}
          <button
            data-idx={i}
            onclick={() => { cmd.action(); onClose(); }}
            onmouseenter={() => (selectedIndex = i)}
            class="w-full text-left px-4 py-2.5 flex items-center gap-3 transition {selectedIndex === i ? 'bg-accent-soft' : 'hover:bg-bg-subtle'} {i !== filtered.length - 1 ? 'border-b border-border' : ''}"
          >
            <span class="text-fg-muted shrink-0">
              <cmd.icon class="w-4 h-4" />
            </span>
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium">{cmd.title}</div>
              {#if cmd.subtitle}
                <div class="text-[10px] text-fg-muted">{cmd.subtitle}</div>
              {/if}
            </div>
            {#if selectedIndex === i}
              <kbd class="text-[10px] text-fg-muted bg-bg-muted px-1.5 py-0.5 rounded-md border border-border font-mono shrink-0">Enter</kbd>
            {/if}
          </button>
        {/each}
      {/if}
    </div>
  </div>
</div>
