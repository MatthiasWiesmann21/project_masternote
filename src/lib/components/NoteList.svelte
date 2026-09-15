<script lang="ts">
  import type { Note } from '$lib/api';
  import { selectedNoteId, activeTagFilter, activeCategoryFilter, activeContactFilter, activeCoworkerFilter, dateFrom, dateTo, searchQuery, timeRangeSort, filteredNotes, showArchived, selectedNoteIds, loadNotes } from '$lib/stores/notes';
  import * as api from '$lib/api';
  import Avatar from '$lib/components/ui/Avatar.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { ArchiveRestore, Bell, FileText, Inbox, Pin } from '@lucide/svelte';
  import { t } from '$lib/i18n';
  import { get } from 'svelte/store';

  let { notes } = $props<{ notes: Note[] }>();

  async function unarchiveFromList(e: MouseEvent, n: Note) {
    e.stopPropagation();
    try {
      await api.unarchiveNote(n.id);
      await loadNotes();
    } catch (err) {
      console.error('Failed to unarchive:', err);
    }
  }

  let filtered = $derived.by(() => {
    let result: Note[] = notes;
    // Filter archived unless showArchived is on
    if (!$showArchived) {
      result = result.filter((n: Note) => !n.archived);
    }
    // Search query filtering (title and content)
    const q = $searchQuery.trim().toLowerCase();
    if (q) {
      result = result.filter((n: Note) =>
        n.title.toLowerCase().includes(q) || n.content.toLowerCase().includes(q)
      );
    }
    if ($activeTagFilter) {
      result = result.filter((n: Note) => n.tags.some((t) => t.name === $activeTagFilter));
    }
    if ($activeCategoryFilter !== null) {
      result = result.filter((n: Note) => n.categoryId === $activeCategoryFilter);
    }
    if ($activeContactFilter !== null) {
      result = result.filter((n: Note) => n.contactId === $activeContactFilter);
    }
    if ($activeCoworkerFilter !== null) {
      result = result.filter((n: Note) => n.coworkerId === $activeCoworkerFilter);
    }

    // Date range filtering
    if ($dateFrom) {
      const fromTime = new Date($dateFrom + 'T00:00:00').getTime();
      result = result.filter((n: Note) => new Date(n.createdAt).getTime() >= fromTime);
    }
    if ($dateTo) {
      const toTime = new Date($dateTo + 'T23:59:59').getTime();
      result = result.filter((n: Note) => new Date(n.createdAt).getTime() <= toTime);
    }

    // Time range filtering (preset)
    const sort = $timeRangeSort;
    const now = new Date();
    if (sort === 'today') {
      const startOfDay = new Date(now.getFullYear(), now.getMonth(), now.getDate());
      result = result.filter((n: Note) => new Date(n.createdAt) >= startOfDay);
    } else if (sort === 'week') {
      const startOfWeek = new Date(now);
      startOfWeek.setDate(now.getDate() - 7);
      result = result.filter((n: Note) => new Date(n.createdAt) >= startOfWeek);
    }

    // Sort
    if (sort === 'oldest') {
      result = [...result].sort((a, b) => new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime());
    } else {
      result = [...result].sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
    }

    return result;
  });

  // Sync filtered notes to store for keyboard navigation
  $effect(() => {
    filteredNotes.set(filtered);
  });

  function selectNote(n: Note) {
    selectedNoteId.set(n.id);
  }

  function formatDate(iso: string): string {
    const d = new Date(iso);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return get(t)('notes.justNow');
    if (mins < 60) return get(t)('notes.minutesAgo', { count: mins });
    const hours = Math.floor(mins / 60);
    if (hours < 24) return get(t)('notes.hoursAgo', { count: hours });
    const days = Math.floor(hours / 24);
    if (days === 1) return get(t)('notes.yesterday');
    if (days < 7) return get(t)('notes.daysAgo', { count: days });
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  }

  function preview(text: string): string {
    return text.replace(/[#*`_~\[\]()]/g, '').slice(0, 80);
  }

  function contactName(n: Note): string {
    if (n.contact?.kind === 'company' && n.contact.companyName) return n.contact.companyName;
    return [n.contact?.firstName, n.contact?.lastName].filter(Boolean).join(' ');
  }

  function coworkerName(n: Note): string {
    return [n.coworker?.firstName, n.coworker?.lastName].filter(Boolean).join(' ');
  }

  function reminderColor(dueAt: string): 'red' | 'amber' | 'blue' {
    const due = new Date(dueAt);
    const now = new Date();
    const diff = due.getTime() - now.getTime();
    const hours = diff / (1000 * 60 * 60);
    if (hours < 0) return 'red';
    if (hours < 24) return 'amber';
    return 'blue';
  }

  let lastClickedId = $state<number | null>(null);

  function handleClick(e: MouseEvent, n: Note) {
    if (e.shiftKey && lastClickedId !== null) {
      // Range selection
      const ids = filtered.map((f) => f.id);
      const startIdx = ids.indexOf(lastClickedId!);
      const endIdx = ids.indexOf(n.id);
      if (startIdx !== -1 && endIdx !== -1) {
        const [from, to] = startIdx < endIdx ? [startIdx, endIdx] : [endIdx, startIdx];
        selectedNoteIds.update((s) => {
          const newSet = new Set(s);
          for (let i = from; i <= to; i++) newSet.add(ids[i]);
          return newSet;
        });
      }
    } else if (e.ctrlKey || e.metaKey) {
      // Toggle individual selection
      selectedNoteIds.update((s) => {
        const newSet = new Set(s);
        if (newSet.has(n.id)) newSet.delete(n.id);
        else newSet.add(n.id);
        return newSet;
      });
    } else {
      selectedNoteIds.set(new Set());
      selectNote(n);
    }
    lastClickedId = n.id;
  }
</script>

<div class="flex flex-col h-full overflow-hidden">
  {#if $selectedNoteIds.size > 0}
    <div class="px-3 py-1.5 bg-accent-soft border-b border-border flex items-center gap-2 text-[10px]">
      <span class="text-accent font-medium">{$t('notes.selected', { count: $selectedNoteIds.size })}</span>
      <button onclick={() => selectedNoteIds.set(new Set())} class="text-fg-muted hover:text-fg">{$t('common.clear')}</button>
    </div>
  {/if}
  <div class="flex-1 overflow-y-auto">
    {#if filtered.length === 0}
      <div class="flex flex-col items-center justify-center h-full text-fg-muted text-xs px-4 text-center gap-2 py-8">
        {#if $searchQuery}
          <Inbox class="w-8 h-8 opacity-40" />
          <div>{$t('notes.emptySearch')}</div>
        {:else}
          <FileText class="w-8 h-8 opacity-40" />
          <div>{$t('notes.empty')}<br />{$t('notes.emptyHint')}</div>
        {/if}
      </div>
    {:else}
      {#each filtered as n (n.id)}
        <button
          onclick={(e) => handleClick(e, n)}
          class="relative w-full text-left px-3 py-2.5 border-b border-border hover:bg-bg-subtle transition-all duration-150 {$selectedNoteId === n.id ? 'bg-accent-soft' : ''} {$selectedNoteIds.has(n.id) ? 'bg-accent/10' : ''} {n.archived ? 'opacity-60' : ''}"
        >
          <!-- Category color strip -->
          {#if n.categoryColor}
            <span class="absolute left-0 top-0 bottom-0 w-1" style="background: {n.categoryColor}"></span>
          {/if}

          <div class="flex items-start justify-between gap-2">
            <span class="text-sm font-medium truncate flex-1 {n.archived ? 'line-through' : ''}">
              {n.title || $t('notes.untitled')}
              {#if n.pinned}
                <Pin class="w-3 h-3 text-accent inline-block ml-0.5 -mt-0.5" />
              {/if}
            </span>
            <span class="text-[10px] text-fg-muted whitespace-nowrap mt-0.5">
              {formatDate(n.updatedAt)}
            </span>
          </div>

          {#if n.content}
            <p class="text-xs text-fg-muted truncate mt-0.5">{preview(n.content)}</p>
          {/if}

          <!-- Badges row -->
          <div class="flex items-center gap-1 mt-1.5 flex-wrap">
            {#if n.archived}
              <Badge color="gray">
                <ArchiveRestore class="w-2.5 h-2.5" />
                {$t('notes.archived')}
              </Badge>
              <span
                role="button"
                tabindex="0"
                onclick={(e) => unarchiveFromList(e, n)}
                onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); unarchiveFromList(e as any, n); } }}
                class="text-[10px] text-accent hover:underline cursor-pointer font-medium"
                title={$t('notes.restoreTitle')}
              >
                {$t('notes.restore')}
              </span>
            {/if}
            {#if n.categoryName}
              <Badge color="accent">{n.categoryName}</Badge>
            {/if}
            {#if n.contact}
              <span class="inline-flex items-center gap-1">
                <Avatar name={contactName(n)} color="blue" size="xs" />
                <span class="text-[10px] text-fg-muted truncate max-w-20">{contactName(n)}</span>
              </span>
            {/if}
            {#if n.coworker}
              <span class="inline-flex items-center gap-1">
                <Avatar name={coworkerName(n)} color="green" size="xs" />
                <span class="text-[10px] text-fg-muted truncate max-w-20">{coworkerName(n)}</span>
              </span>
            {/if}
            {#each n.tags as tag}
              <Badge color="gray">#{tag.name}</Badge>
            {/each}
            {#if n.reminder}
              {@const color = reminderColor(n.reminder.dueAt)}
              <Badge color={color}>
                <Bell class="w-2.5 h-2.5" />
                {new Date(n.reminder.dueAt).toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}
              </Badge>
            {/if}
          </div>
        </button>
      {/each}
    {/if}
  </div>
</div>
