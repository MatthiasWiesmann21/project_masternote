<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '$lib/api';
  import type { Coworker } from '$lib/api';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Avatar from '$lib/components/ui/Avatar.svelte';
  import { Users, Search, Plus, Pencil, Trash2, AlertCircle } from '@lucide/svelte';
  import { t } from '$lib/i18n';
  import { get } from 'svelte/store';

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
      error = get(t)('coworker.nameRequired');
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
    if (!confirm(get(t)('coworker.deleteConfirm', { name: fullName(c) }))) return;
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

<Modal onClose={onClose} width="w-[480px]" title={$t('coworker.title')}>
  {#snippet icon()}<Users class="w-4 h-4" />{/snippet}

  <!-- Toolbar -->
  <div class="flex items-center gap-2 px-4 py-2.5 border-b border-border">
    <div class="relative flex-1">
      <Search class="w-3.5 h-3.5 text-fg-muted absolute left-2.5 top-1/2 -translate-y-1/2" />
      <input
        bind:value={searchQuery}
        placeholder={$t('coworker.search')}
        class="w-full text-xs bg-bg-muted rounded-lg pl-8 pr-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20"
      />
    </div>
    <button onclick={startNew} class="text-xs px-2.5 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition inline-flex items-center gap-1">
      <Plus class="w-3.5 h-3.5" />
      {$t('coworker.new')}
    </button>
  </div>

  {#if error}
    <div class="px-4 py-1.5 text-xs text-danger bg-danger-soft flex items-center gap-1.5">
      <AlertCircle class="w-3.5 h-3.5 shrink-0" />
      {error}
    </div>
  {/if}

  {#if isEditing}
    <div class="flex-1 overflow-y-auto p-4 space-y-2.5">
      <div class="flex gap-2">
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('coworker.firstName')}</span>
          <input bind:value={firstName} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
        </label>
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('coworker.lastName')}</span>
          <input bind:value={lastName} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
        </label>
      </div>
      <label class="flex flex-col gap-1">
        <span class="text-[10px] text-fg-muted font-medium">{$t('coworker.email')}</span>
        <input bind:value={email} type="email" class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
      </label>
      <div class="flex gap-2 justify-end pt-2">
        <button onclick={cancelEdit} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
        <button onclick={handleSave} class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition">
          {editing ? $t('common.update') : $t('common.create')}
        </button>
      </div>
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto">
      {#if coworkers.length === 0}
        <div class="flex flex-col items-center justify-center h-full text-fg-muted text-xs py-12 gap-2">
          <Users class="w-8 h-8 opacity-40" />
          <div>{$t('coworker.empty')}<br />{$t('coworker.emptyHint')}</div>
        </div>
      {:else}
        {#each coworkers as c (c.id)}
          <div class="flex items-center gap-2.5 px-4 py-2.5 border-b border-border hover:bg-bg-subtle transition">
            <Avatar name={fullName(c)} color="green" />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium truncate">{fullName(c)}</div>
              {#if c.email}
                <div class="text-[10px] text-fg-muted truncate">{c.email}</div>
              {/if}
            </div>
            <button onclick={() => startEdit(c)} class="text-xs p-1.5 rounded-lg hover:bg-border transition" title={$t('coworker.editTitle')}><Pencil class="w-3.5 h-3.5" /></button>
            <button onclick={() => handleDelete(c)} class="text-xs p-1.5 rounded-lg text-danger hover:bg-danger-soft transition" title={$t('coworker.deleteTitle')}><Trash2 class="w-3.5 h-3.5" /></button>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</Modal>
