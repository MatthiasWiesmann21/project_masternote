<script lang="ts">
  import type { Tag } from '$lib/api';
  import * as api from '$lib/api';
  import { loadNotes, loadTags } from '$lib/stores/notes';
  import { t } from '$lib/i18n';

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
    } else if (e.key === 'Backspace' && newTag === '' && tags.length > 0) {
      // Remove last tag on backspace when input is empty
      removeTag(tags[tags.length - 1].id);
    }
  }
</script>

<div class="flex items-center gap-1 flex-wrap">
  {#each tags as tag (tag.id)}
    <span class="inline-flex items-center gap-0.5 text-[10px] px-1.5 py-0.5 rounded-full bg-accent/15 text-accent">
      #{tag.name}
      <button
        onclick={() => removeTag(tag.id)}
        class="hover:text-red-500 text-[9px] leading-none"
        title={$t('picker.removeTag')}
      >✕</button>
    </span>
  {/each}
  <input
    bind:this={inputEl}
    bind:value={newTag}
    onkeydown={onKeydown}
    placeholder={tags.length === 0 ? $t('picker.addTag') : ''}
    class="text-[10px] bg-transparent outline-none placeholder:text-fg-muted flex-1 min-w-16"
  />
</div>
