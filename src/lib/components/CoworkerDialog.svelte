<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '$lib/api';
  import type { Coworker } from '$lib/api';

  let { onClose } = $props<{ onClose: () => void }>();

  let coworkers = $state<Coworker[]>([]);
  let searchQuery = $state('');
  let editing = $state<Coworker | null>(null);
  let isEditing = $state(false);
  let error = $state('');

  let lastName = $state('');
  let firstName = $state('');
  let email = $state('');

  async function loadCoworkers() {
    try {
      if (searchQuery.trim()) {
        coworkers = await api.searchCoworkers(searchQuery.trim());
      } else {
        coworkers = await api.listCoworkers();
      }
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  $effect(() => {
    searchQuery;
    const timer = setTimeout(loadCoworkers, 200);
    return () => clearTimeout(timer);
  });

  onMount(() => loadCoworkers());

  function startNew() {
    editing = null;
    isEditing = true;
    lastName = '';
    firstName = '';
    email = '';
  }

  function startEdit(c: Coworker) {
    editing = c;
    isEditing = true;
    lastName = c.lastName;
    firstName = c.firstName;
    email = c.email ?? '';
  }

  function cancelEdit() {
    isEditing = false;
    editing = null;
  }

  async function handleSave() {
    error = '';
    if (!lastName.trim() && !firstName.trim()) {
      error = 'At least a name is required';
      return;
    }
    const payload = {
      lastName: lastName.trim(),
      firstName: firstName.trim(),
      email: email.trim() || null,
    };
    try {
      if (editing) {
        await api.updateCoworker(editing.id, payload);
      } else {
        await api.createCoworker(payload);
      }
      isEditing = false;
      editing = null;
      await loadCoworkers();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  async function handleDelete(c: Coworker) {
    if (!confirm(`Delete ${c.firstName} ${c.lastName}?`)) return;
    try {
      await api.deleteCoworker(c.id);
      await loadCoworkers();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  function fullName(c: Coworker): string {
    return [c.firstName, c.lastName].filter(Boolean).join(' ');
  }
</script>

<svelte:window on:keydown={(e) => { if (e.key === 'Escape' && !isEditing) onClose(); }} />

<div
  class="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
  role="button"
  tabindex="-1"
  onclick={onClose}
  onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}
>
  <div
    class="bg-bg rounded-lg shadow-2xl border border-border w-[480px] max-h-[80vh] flex flex-col"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <div class="flex items-center gap-2 px-4 py-3 border-b border-border">
      <h3 class="text-sm font-semibold flex-1">🤝 Coworkers</h3>
      <input
        bind:value={searchQuery}
        placeholder="Search…"
        class="text-xs bg-bg-muted rounded px-2 py-1 border border-border outline-none w-40"
      />
      <button onclick={startNew} class="text-xs px-3 py-1 rounded bg-accent text-accent-fg font-medium hover:opacity-90">
        + New
      </button>
      <button onclick={onClose} class="text-fg-muted hover:text-fg text-xs px-1">✕</button>
    </div>

    {#if error}
      <div class="px-4 py-1.5 text-xs text-red-500 bg-red-500/10">{error}</div>
    {/if}

    {#if isEditing}
      <div class="flex-1 overflow-y-auto p-4 space-y-2">
        <div class="flex gap-2">
          <label class="flex-1 flex flex-col gap-1">
            <span class="text-[10px] text-fg-muted">First name</span>
            <input bind:value={firstName} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
          </label>
          <label class="flex-1 flex flex-col gap-1">
            <span class="text-[10px] text-fg-muted">Last name</span>
            <input bind:value={lastName} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
          </label>
        </div>
        <label class="flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted">Email</span>
          <input bind:value={email} type="email" class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
        </label>
        <div class="flex gap-2 justify-end pt-2">
          <button onclick={cancelEdit} class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border">Cancel</button>
          <button onclick={handleSave} class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90">
            {editing ? 'Update' : 'Create'}
          </button>
        </div>
      </div>
    {:else}
      <div class="flex-1 overflow-y-auto">
        {#if coworkers.length === 0}
          <div class="flex flex-col items-center justify-center h-full text-fg-muted text-xs py-8">
            No coworkers yet. Click "+ New" to add one.
          </div>
        {:else}
          {#each coworkers as c (c.id)}
            <div class="flex items-center gap-2 px-4 py-2 border-b border-border hover:bg-bg-subtle">
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium truncate">{fullName(c)}</div>
                {#if c.email}
                  <div class="text-[10px] text-fg-muted truncate">{c.email}</div>
                {/if}
              </div>
              <button onclick={() => startEdit(c)} class="text-xs px-1.5 py-1 rounded hover:bg-border" title="Edit">✎</button>
              <button onclick={() => handleDelete(c)} class="text-xs px-1.5 py-1 rounded text-red-500 hover:bg-red-500/10" title="Delete">🗑</button>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </div>
</div>
