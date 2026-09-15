<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '$lib/api';
  import type { Contact } from '$lib/api';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import Modal from '$lib/components/ui/Modal.svelte';
  import Avatar from '$lib/components/ui/Avatar.svelte';
  import { Users, Search, FileUp, FileDown, Plus, Pencil, Trash2, AlertCircle } from '@lucide/svelte';
  import { t } from '$lib/i18n';
  import { get } from 'svelte/store';

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
      error = get(t)('contact.nameRequired');
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
      status = get(t)('contact.saved');
      setTimeout(() => (status = ''), 1500);
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  async function handleDelete(c: Contact) {
    if (!confirm(get(t)('contact.deleteConfirm', { name: fullName(c) }))) return;
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
        status = get(t)('contact.exportedTo', { path: filePath });
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
        status = get(t)('contact.imported', { count });
        setTimeout(() => (status = ''), 3000);
      }
    } catch (e: any) {
      error = e?.message ?? String(e);
    }
  }

  function fullName(c: Contact): string {
    return c.kind === 'company' && c.companyName
      ? c.companyName
      : [c.firstName, c.lastName].filter(Boolean).join(' ');
  }
</script>

<Modal onClose={onClose} width="w-[640px]" title={$t('contact.title')}>
  {#snippet icon()}<Users class="w-4 h-4" />{/snippet}

  <!-- Toolbar -->
  <div class="flex items-center gap-2 px-4 py-2.5 border-b border-border">
    <div class="relative flex-1">
      <Search class="w-3.5 h-3.5 text-fg-muted absolute left-2.5 top-1/2 -translate-y-1/2" />
      <input
        bind:value={searchQuery}
        placeholder={$t('contact.search')}
        class="w-full text-xs bg-bg-muted rounded-lg pl-8 pr-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20"
      />
    </div>
    <button onclick={handleImportCsv} class="text-xs px-2 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1" title={$t('contact.import')}>
      <FileUp class="w-3.5 h-3.5" />
    </button>
    <button onclick={handleExportCsv} class="text-xs px-2 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition inline-flex items-center gap-1" title={$t('contact.export')}>
      <FileDown class="w-3.5 h-3.5" />
    </button>
    <button onclick={startNew} class="text-xs px-2.5 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition inline-flex items-center gap-1">
      <Plus class="w-3.5 h-3.5" />
      {$t('contact.new')}
    </button>
  </div>

  {#if error}
    <div class="px-4 py-1.5 text-xs text-danger bg-danger-soft flex items-center gap-1.5">
      <AlertCircle class="w-3.5 h-3.5 shrink-0" />
      {error}
    </div>
  {/if}
  {#if status}
    <div class="px-4 py-1.5 text-xs text-success bg-success-soft">{status}</div>
  {/if}

  {#if isEditing}
    <!-- Edit form -->
    <div class="flex-1 overflow-y-auto p-4 space-y-2.5">
      <div class="flex gap-2">
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('contact.firstName')}</span>
          <input bind:value={firstName} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
        </label>
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('contact.lastName')}</span>
          <input bind:value={lastName} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
        </label>
      </div>

      <label class="flex flex-col gap-1">
        <span class="text-[10px] text-fg-muted font-medium">{$t('contact.address')}</span>
        <input bind:value={address} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
      </label>

      <div class="flex gap-2">
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('contact.email')}</span>
          <input bind:value={email} type="email" class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
        </label>
        <label class="flex flex-col gap-1 w-32">
          <span class="text-[10px] text-fg-muted font-medium">{$t('contact.gender')}</span>
          <select bind:value={gender} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent">
            <option value="">—</option>
            <option value="male">{$t('contact.genderMale')}</option>
            <option value="female">{$t('contact.genderFemale')}</option>
            <option value="diverse">{$t('contact.genderDiverse')}</option>
            <option value="other">{$t('contact.genderOther')}</option>
          </select>
        </label>
      </div>

      <div class="flex gap-2">
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('contact.phone')}</span>
          <input bind:value={phone} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
        </label>
        <label class="flex-1 flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('contact.mobile')}</span>
          <input bind:value={mobile} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
        </label>
      </div>

      <div class="flex gap-2 items-end">
        <label class="flex flex-col gap-1 w-32">
          <span class="text-[10px] text-fg-muted font-medium">{$t('contact.type')}</span>
          <select bind:value={kind} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent">
            <option value="private">{$t('contact.private')}</option>
            <option value="company">{$t('contact.company')}</option>
          </select>
        </label>
        {#if kind === 'company'}
          <label class="flex-1 flex flex-col gap-1">
            <span class="text-[10px] text-fg-muted font-medium">{$t('contact.companyName')}</span>
            <input bind:value={companyName} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
          </label>
        {/if}
      </div>

      {#if kind === 'private'}
        <label class="flex flex-col gap-1">
          <span class="text-[10px] text-fg-muted font-medium">{$t('contact.companyNameOptional')}</span>
          <input bind:value={companyName} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
        </label>
      {/if}

      <label class="flex flex-col gap-1">
        <span class="text-[10px] text-fg-muted font-medium">{$t('contact.customerId')}</span>
        <input bind:value={customerIdentifier} class="text-sm bg-bg-muted rounded-lg px-2.5 py-1.5 border border-border outline-none focus:border-accent focus:ring-2 focus:ring-accent/20" />
      </label>

      <div class="flex gap-2 justify-end pt-2">
        <button onclick={cancelEdit} class="text-xs px-3 py-1.5 rounded-lg bg-bg-muted hover:bg-border transition">{$t('common.cancel')}</button>
        <button onclick={handleSave} class="text-xs px-3 py-1.5 rounded-lg bg-accent text-accent-fg font-medium hover:bg-accent-dark transition">
          {editing ? $t('common.update') : $t('common.create')}
        </button>
      </div>
    </div>
  {:else}
    <!-- List -->
    <div class="flex-1 overflow-y-auto">
      {#if contacts.length === 0}
        <div class="flex flex-col items-center justify-center h-full text-fg-muted text-xs py-12 gap-2">
          <Users class="w-8 h-8 opacity-40" />
          <div>{$t('contact.empty')}<br />{$t('contact.emptyHint')}</div>
        </div>
      {:else}
        {#each contacts as c (c.id)}
          <div class="flex items-center gap-2.5 px-4 py-2.5 border-b border-border hover:bg-bg-subtle transition">
            <Avatar name={fullName(c)} color={c.kind === 'company' ? 'purple' : 'blue'} />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium truncate">{fullName(c)}</div>
              <div class="text-[10px] text-fg-muted truncate">
                {#if c.email}{c.email}{/if}
                {#if c.phone}· {c.phone}{/if}
                {#if c.mobile}· {c.mobile}{/if}
                {#if c.companyName && c.kind === 'private'}· {c.companyName}{/if}
                {#if c.customerIdentifier}· ID: {c.customerIdentifier}{/if}
              </div>
            </div>
            <span class="text-[10px] px-1.5 py-0.5 rounded-full font-medium {c.kind === 'company' ? 'bg-purple-500/15 text-purple-600 dark:text-purple-400' : 'bg-info-soft text-info'}">
              {c.kind === 'company' ? $t('contact.company') : $t('contact.private')}
            </span>
            <button onclick={() => startEdit(c)} class="text-xs p-1.5 rounded-lg hover:bg-border transition" title={$t('contact.editTitle')}><Pencil class="w-3.5 h-3.5" /></button>
            <button onclick={() => handleDelete(c)} class="text-xs p-1.5 rounded-lg text-danger hover:bg-danger-soft transition" title={$t('contact.deleteTitle')}><Trash2 class="w-3.5 h-3.5" /></button>
          </div>
        {/each}
      {/if}
    </div>
  {/if}
</Modal>
