<script lang="ts">
  import type { Tag } from '$lib/api';
  import * as api from '$lib/api';
  import { loadNotes, loadTags, tags as tagsStore } from '$lib/stores/notes';

  let { noteId, tags } = $props<{ noteId: number; tags: Tag[] }>();

  let newTag = $state('');
  let inputEl = $state<HTMLInputElement | null>(null);

  async function addTag() {
    const name = newTag.trim().toLowerCase();
    if (!name) return;
    try {
      await api.addTagToNote(noteId, name);
      newTag = '';
      await Promise.all([loadNotes(), loadTags()]);
    } catch (e) {
      console.error('Failed to add tag:', e);
    }
  }

  async function removeTag(tagId: number) {
    try {
      await api.removeTagFromNote(noteId, tagId);
      await loadNotes();
    } catch (e) {
      console.error('Failed to remove tag:', e);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      addTag();
    }
  }
</script>

<div class="flex items-center gap-1 flex-wrap">
  {#each tags as tag (tag.id)}
    <span class="inline-flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded-full bg-accent/15 text-accent">
      #{tag.name}
      <button onclick={() => removeTag(tag.id)} class="hover:opacity-70 text-[9px]">✕</button>
    </span>
  {/each}
  <input
    bind:this={inputEl}
    bind:value={newTag}
    onkeydown={onKeydown}
    placeholder="add tag…"
    class="text-[10px] bg-transparent outline-none placeholder:text-fg-muted w-20"
  />
</div>
