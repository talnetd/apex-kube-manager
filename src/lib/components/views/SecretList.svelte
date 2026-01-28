<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import { secrets, selectedNamespace, currentContext, refreshTrigger, loadSecrets } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';
  import SecretCreator from '../ui/SecretCreator.svelte';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });
  let filterQuery = $state('');
  let showCreator = $state(false);
  let editingSecret = $state<any>(null);
  let loadingEdit = $state<string | null>(null);
  let deletingSecret = $state<string | null>(null);
  let showDeleteConfirm = $state<{ name: string; namespace: string } | null>(null);

  async function openDetail(sec: { name: string; namespace: string }) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'secret',
        name: sec.name,
        namespace: sec.namespace,
        context: $currentContext
      });
    } catch (e) {
      console.error('Failed to open detail:', e);
    }
  }

  const sortedData = $derived(() => {
    const filtered = filterBySearch($secrets, filterQuery, ['name', 'namespace', 'secret_type']);
    return sortData(filtered, sort.field, sort.direction);
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadSecrets($selectedNamespace);
    const interval = setInterval(() => loadSecrets($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    loadSecrets($selectedNamespace);
  });

  function getTypeColor(type: string): string {
    if (type.includes('tls')) return 'text-accent-primary bg-accent-primary/10';
    if (type.includes('dockerconfig')) return 'text-accent-warning bg-accent-warning/10';
    if (type.includes('service-account')) return 'text-accent-success bg-accent-success/10';
    return 'text-text-muted bg-bg-tertiary';
  }

  async function handleEdit(e: Event, sec: { name: string; namespace: string; secret_type: string }) {
    e.stopPropagation();
    loadingEdit = sec.name;
    try {
      // Get the decoded secret data
      const data = await invoke('get_secret_data', {
        contextName: $currentContext,
        namespace: sec.namespace,
        name: sec.name
      }) as Record<string, string>;

      editingSecret = {
        name: sec.name,
        namespace: sec.namespace,
        secret_type: sec.secret_type,
        data
      };
      showCreator = true;
    } catch (err) {
      console.error('Failed to load secret for editing:', err);
    } finally {
      loadingEdit = null;
    }
  }

  function handleCloseCreator() {
    showCreator = false;
    editingSecret = null;
  }

  function confirmDelete(e: Event, sec: { name: string; namespace: string }) {
    e.stopPropagation();
    showDeleteConfirm = sec;
  }

  async function handleDelete() {
    if (!showDeleteConfirm) return;

    deletingSecret = showDeleteConfirm.name;
    try {
      await invoke('delete_secret', {
        contextName: $currentContext,
        namespace: showDeleteConfirm.namespace,
        name: showDeleteConfirm.name
      });
      await loadSecrets($selectedNamespace);
    } catch (err) {
      console.error('Failed to delete secret:', err);
    } finally {
      deletingSecret = null;
      showDeleteConfirm = null;
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-semibold text-text-primary">Secrets</h1>
      <div class="flex items-center gap-3">
        <button
          onclick={() => showCreator = true}
          class="flex items-center gap-2 px-3 py-1.5 bg-accent-primary text-bg-primary text-sm font-medium rounded hover:bg-accent-primary/80 transition-colors"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Create
        </button>
        <ViewFilter value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter secrets..." />
      </div>
    </div>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Namespace" field="namespace" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Type" field="secret_type" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Data" field="data_count" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-20">Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each sortedData() as sec}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer" onclick={() => openDetail(sec)}>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{sec.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sec.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-xs px-2 py-0.5 rounded {getTypeColor(sec.secret_type)}">{sec.secret_type}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sec.data_count} {sec.data_count === 1 ? 'key' : 'keys'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sec.age}</span>
            </td>
            <td class="py-3 pr-4">
              <div class="flex items-center gap-1">
                <button
                  onclick={(e) => handleEdit(e, sec)}
                  disabled={loadingEdit === sec.name}
                  class="p-1.5 rounded hover:bg-bg-tertiary transition-colors text-text-muted hover:text-accent-primary disabled:opacity-50"
                  title="Edit secret"
                >
                  {#if loadingEdit === sec.name}
                    <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  {/if}
                </button>
                <button
                  onclick={(e) => confirmDelete(e, sec)}
                  disabled={deletingSecret === sec.name}
                  class="p-1.5 rounded hover:bg-accent-error/10 transition-colors text-text-muted hover:text-accent-error disabled:opacity-50"
                  title="Delete secret"
                >
                  {#if deletingSecret === sec.name}
                    <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  {/if}
                </button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if sortedData().length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
          <p class="text-text-muted">No secrets found</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Secret Creator Modal -->
<SecretCreator show={showCreator} editData={editingSecret} onclose={handleCloseCreator} />

<!-- Delete Confirmation Modal -->
{#if showDeleteConfirm}
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onclick={() => showDeleteConfirm = null}>
    <div
      class="bg-bg-secondary rounded-lg border border-border-subtle w-[400px] p-6"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center gap-3 mb-4">
        <div class="p-2 rounded-full bg-accent-error/10">
          <svg class="w-6 h-6 text-accent-error" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
        </div>
        <div>
          <h3 class="text-lg font-semibold text-text-primary">Delete Secret</h3>
          <p class="text-sm text-text-muted">This action cannot be undone</p>
        </div>
      </div>

      <p class="text-sm text-text-secondary mb-6">
        Are you sure you want to delete <span class="font-medium text-text-primary">{showDeleteConfirm.name}</span> from namespace <span class="font-medium text-text-primary">{showDeleteConfirm.namespace}</span>?
      </p>

      <div class="flex justify-end gap-3">
        <button
          onclick={() => showDeleteConfirm = null}
          class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={handleDelete}
          disabled={deletingSecret !== null}
          class="px-4 py-2 text-sm bg-accent-error text-white rounded hover:bg-accent-error/80 transition-colors disabled:opacity-50"
        >
          {deletingSecret ? 'Deleting...' : 'Delete'}
        </button>
      </div>
    </div>
  </div>
{/if}
