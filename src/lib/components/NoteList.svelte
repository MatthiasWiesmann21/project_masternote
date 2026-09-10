<script lang="ts">
  import type { Note } from '$lib/api';
  import { selectedNoteId, activeTagFilter, activeCategoryFilter, searchQuery, timeRangeSort, filteredNotes } from '$lib/stores/notes';
  import * as api from '$lib/api';

  let { notes } = $props<{ notes: Note[] }>();

  let filtered = $derived.by(() => {
    let result: Note[] = notes;
    if ($activeTagFilter) {
      result = result.filter((n: Note) => n.tags.some((t) => t.name === $activeTagFilter));
    }
    if ($activeCategoryFilter !== null) {
      result = result.filter((n: Note) => n.categoryId === $activeCategoryFilter);
    }

    // Time range filtering
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
    if (mins < 1) return 'just now';
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  }

  function preview(text: string): string {
    return text.replace(/[#*`_~\[\]()]/g, '').slice(0, 80);
  }
</script>

<div class="flex flex-col h-full overflow-hidden">
  <div class="flex-1 overflow-y-auto">
    {#if filtered.length === 0}
      <div class="flex flex-col items-center justify-center h-full text-fg-muted text-xs px-4 text-center">
        {#if $searchQuery}
          No notes found.
        {:else}
          No notes yet.<br />Start typing above to capture one.
        {/if}
      </div>
    {:else}
      {#each filtered as n (n.id)}
        <button
          onclick={() => selectNote(n)}
          class="w-full text-left px-3 py-2.5 border-b border-border hover:bg-bg-subtle transition {$selectedNoteId === n.id ? 'bg-bg-subtle border-l-2 border-l-accent' : ''}"
        >
          <div class="flex items-start justify-between gap-2">
            <span class="text-sm font-medium truncate flex-1">
              {n.title || 'Untitled'}
            </span>
            <span class="text-[10px] text-fg-muted whitespace-nowrap mt-0.5">
              {formatDate(n.updatedAt)}
            </span>
          </div>
          {#if n.content}
            <p class="text-xs text-fg-muted truncate mt-0.5">{preview(n.content)}</p>
          {/if}
          <div class="flex items-center gap-1 mt-1 flex-wrap">
            {#if n.categoryName}
              <span
                class="text-[10px] px-1.5 py-0.5 rounded-full"
                style="background: {n.categoryColor || '#888'}20; color: {n.categoryColor || '#888'}"
              >
                {n.categoryName}
              </span>
            {/if}
            {#if n.contact}
              <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-blue-500/15 text-blue-600 dark:text-blue-400">
                👤 {n.contact.kind === 'company' && n.contact.companyName ? n.contact.companyName : [n.contact.firstName, n.contact.lastName].filter(Boolean).join(' ')}
              </span>
            {/if}
            {#if n.coworker}
              <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-green-500/15 text-green-600 dark:text-green-400">
                🤝 {[n.coworker.firstName, n.coworker.lastName].filter(Boolean).join(' ')}
              </span>
            {/if}
            {#each n.tags as tag}
              <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-bg-muted text-fg-muted">
                #{tag.name}
              </span>
            {/each}
            {#if n.reminder}
              <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-amber-500/15 text-amber-600 dark:text-amber-400">
                ⏰ {new Date(n.reminder.dueAt).toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}
              </span>
            {/if}
          </div>
        </button>
      {/each}
    {/if}
  </div>
</div>
