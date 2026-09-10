<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '$lib/api';
  import type { Contact } from '$lib/api';
  import { open, save } from '@tauri-apps/plugin-dialog';

  let { onClose } = $props<{ onClose: () => void }>();

  let contacts = $state<Contact[]>([]);
  let searchQuery = $state('');
  let editing = $state<Contact | null>(null);
  let isEditing = $state(false);
  let error = $state('');
  let status = $state('');

  // Form fields
  let lastName = $state('');
  let firstName = $state('');
  let address = $state('');
  let email = $state('');
  let gender = $state('');
  let kind = $state('private');
  let companyName = $state('');
  let customerIdentifier = $state('');
  let phone = $state('');
  let mobile = $state('');

  async function loadContacts() {
    try {
      if (searchQuery.trim()) {
        contacts = await api.searchContacts(searchQuery.trim());
      } else {
        contacts = await api.listContacts();
      }
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  $effect(() => {
    searchQuery;
    const timer = setTimeout(loadContacts, 200);
    return () => clearTimeout(timer);
  });

  onMount(() => loadContacts());

  function startNew() {
    editing = null;
    isEditing = true;
    lastName = '';
    firstName = '';
    address = '';
    email = '';
    gender = '';
    kind = 'private';
    companyName = '';
    customerIdentifier = '';
    phone = '';
    mobile = '';
  }

  function startEdit(c: Contact) {
    editing = c;
    isEditing = true;
    lastName = c.lastName;
    firstName = c.firstName;
    address = c.address ?? '';
    email = c.email ?? '';
    gender = c.gender ?? '';
    kind = c.kind;
    companyName = c.companyName ?? '';
    customerIdentifier = c.customerIdentifier ?? '';
    phone = c.phone ?? '';
    mobile = c.mobile ?? '';
  }

  function cancelEdit() {
    isEditing = false;
    editing = null;
  }

  async function handleSave() {
    error = '';
    if (!lastName.trim() && !firstName.trim() && !companyName.trim()) {
      error = 'At least a name or company name is required';
      return;
    }
    const payload = {
      lastName: lastName.trim(),
      firstName: firstName.trim(),
      address: address.trim() || null,
      email: email.trim() || null,
      gender: gender || null,
      kind,
      companyName: companyName.trim() || null,
      customerIdentifier: customerIdentifier.trim() || null,
      phone: phone.trim() || null,
      mobile: mobile.trim() || null,
    };
    try {
      if (editing) {
        await api.updateContact(editing.id, payload);
      } else {
        await api.createContact(payload);
      }
      isEditing = false;
      editing = null;
      await loadContacts();
      status = 'Saved';
      setTimeout(() => (status = ''), 1500);
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  async function handleDelete(c: Contact) {
    if (!confirm(`Delete ${c.firstName} ${c.lastName}?`)) return;
    try {
      await api.deleteContact(c.id);
      await loadContacts();
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  async function handleExportCsv() {
    try {
      const filePath = await save({
        defaultPath: 'contacts.csv',
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      });
      if (filePath) {
        await api.exportContactsToFile(filePath);
        status = `Exported to ${filePath}`;
        setTimeout(() => (status = ''), 3000);
      }
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  async function handleImportCsv() {
    try {
      const filePath = await open({
        filters: [{ name: 'CSV', extensions: ['csv'] }],
        multiple: false,
      });
      if (filePath && typeof filePath === 'string') {
        const count = await api.importContactsFromFile(filePath);
        await loadContacts();
        status = `Imported ${count} contacts`;
        setTimeout(() => (status = ''), 3000);
      }
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  function fullName(c: Contact): string {
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
    class="bg-bg rounded-lg shadow-2xl border border-border w-[640px] max-h-[80vh] flex flex-col"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <!-- Header -->
    <div class="flex items-center gap-2 px-4 py-3 border-b border-border">
      <h3 class="text-sm font-semibold flex-1">👥 Contacts</h3>
      <input
        bind:value={searchQuery}
        placeholder="Search…"
        class="text-xs bg-bg-muted rounded px-2 py-1 border border-border outline-none w-40"
      />
      <button onclick={handleImportCsv} class="text-xs px-2 py-1 rounded bg-bg-muted hover:bg-border" title="Import CSV">
        📥
      </button>
      <button onclick={handleExportCsv} class="text-xs px-2 py-1 rounded bg-bg-muted hover:bg-border" title="Export CSV">
        📤
      </button>
      <button onclick={startNew} class="text-xs px-3 py-1 rounded bg-accent text-accent-fg font-medium hover:opacity-90">
        + New
      </button>
      <button onclick={onClose} class="text-fg-muted hover:text-fg text-xs px-1">✕</button>
    </div>

    {#if error}
      <div class="px-4 py-1.5 text-xs text-red-500 bg-red-500/10">{error}</div>
    {/if}
    {#if status}
      <div class="px-4 py-1.5 text-xs text-green-500 bg-green-500/10">{status}</div>
    {/if}

    {#if isEditing}
      <!-- Edit form -->
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
          <span class="text-[10px] text-fg-muted">Address</span>
          <input bind:value={address} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
        </label>

        <div class="flex gap-2">
          <label class="flex-1 flex flex-col gap-1">
            <span class="text-[10px] text-fg-muted">Email</span>
            <input bind:value={email} type="email" class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
          </label>
          <label class="flex flex-col gap-1 w-32">
            <span class="text-[10px] text-fg-muted">Gender</span>
            <select bind:value={gender} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none">
              <option value="">—</option>
              <option value="male">Male</option>
              <option value="female">Female</option>
              <option value="diverse">Diverse</option>
              <option value="other">Other</option>
            </select>
          </label>
        </div>

        <div class="flex gap-2">
          <label class="flex-1 flex flex-col gap-1">
            <span class="text-[10px] text-fg-muted">Phone</span>
            <input bind:value={phone} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
          </label>
          <label class="flex-1 flex flex-col gap-1">
            <span class="text-[10px] text-fg-muted">Mobile</span>
            <input bind:value={mobile} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
          </label>
        </div>

        <div class="flex gap-2 items-end">
          <label class="flex flex-col gap-1 w-32">
            <span class="text-[10px] text-fg-muted">Type</span>
            <select bind:value={kind} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none">
              <option value="private">Private</option>
              <option value="company">Company</option>
            </select>
          </label>
          {#if kind === 'company'}
            <label class="flex-1 flex flex-col gap-1">
              <span class="text-[10px] text-fg-muted">Company name</span>
              <input bind:value={companyName} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
            </label>
          {/if}
        </div>

        {#if kind === 'private'}
          <label class="flex flex-col gap-1">
            <span class="text-[10px] text-fg-muted">Company name (optional)</span>
            <input bind:value={companyName} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
          </label>
        {/if}

        <label class="flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted">Customer identifier</span>
          <input bind:value={customerIdentifier} class="text-sm bg-bg-muted rounded px-2 py-1.5 border border-border outline-none" />
        </label>

        <div class="flex gap-2 justify-end pt-2">
          <button onclick={cancelEdit} class="text-xs px-3 py-1.5 rounded bg-bg-muted hover:bg-border">Cancel</button>
          <button onclick={handleSave} class="text-xs px-3 py-1.5 rounded bg-accent text-accent-fg font-medium hover:opacity-90">
            {editing ? 'Update' : 'Create'}
          </button>
        </div>
      </div>
    {:else}
      <!-- List -->
      <div class="flex-1 overflow-y-auto">
        {#if contacts.length === 0}
          <div class="flex flex-col items-center justify-center h-full text-fg-muted text-xs py-8">
            No contacts yet. Click "+ New" to add one, or use 📥 to import a CSV.
          </div>
        {:else}
          {#each contacts as c (c.id)}
            <div class="flex items-center gap-2 px-4 py-2 border-b border-border hover:bg-bg-subtle">
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium truncate">
                  {c.kind === 'company' && c.companyName ? c.companyName : fullName(c)}
                </div>
                <div class="text-[10px] text-fg-muted truncate">
                  {#if c.email}{c.email}{/if}
                  {#if c.phone}· ☎ {c.phone}{/if}
                  {#if c.mobile}· 📱 {c.mobile}{/if}
                  {#if c.companyName && c.kind === 'private'}· {c.companyName}{/if}
                  {#if c.customerIdentifier}· ID: {c.customerIdentifier}{/if}
                </div>
              </div>
              <span class="text-[10px] px-1.5 py-0.5 rounded-full {c.kind === 'company' ? 'bg-purple-500/15 text-purple-600' : 'bg-blue-500/15 text-blue-600'}">
                {c.kind}
              </span>
              <button onclick={() => startEdit(c)} class="text-xs px-1.5 py-1 rounded hover:bg-border" title="Edit">✎</button>
              <button onclick={() => handleDelete(c)} class="text-xs px-1.5 py-1 rounded text-red-500 hover:bg-red-500/10" title="Delete">🗑</button>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  </div>
</div>
