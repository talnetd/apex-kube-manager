<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import { networkPolicies, selectedNamespace, currentContext, refreshTrigger, loadNetworkPolicies } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';
  import NetworkPolicyCreator from '../ui/NetworkPolicyCreator.svelte';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });
  let filterQuery = $state('');
  let showCreator = $state(false);
  let editingPolicy = $state<any>(null);
  let loadingEdit = $state<string | null>(null);
  let deletingPolicy = $state<string | null>(null);
  let showDeleteConfirm = $state<{ name: string; namespace: string } | null>(null);

  async function handleEdit(e: Event, np: { name: string; namespace: string }) {
    e.stopPropagation();
    loadingEdit = np.name;
    try {
      const detail = await invoke('get_networkpolicy_detail', {
        contextName: $currentContext,
        namespace: np.namespace,
        name: np.name
      });
      editingPolicy = detail;
      showCreator = true;
    } catch (err) {
      console.error('Failed to load policy for editing:', err);
    } finally {
      loadingEdit = null;
    }
  }

  function handleCloseCreator() {
    showCreator = false;
    editingPolicy = null;
  }

  function confirmDelete(e: Event, np: { name: string; namespace: string }) {
    e.stopPropagation();
    showDeleteConfirm = np;
  }

  async function handleDelete() {
    if (!showDeleteConfirm) return;

    deletingPolicy = showDeleteConfirm.name;
    try {
      await invoke('delete_network_policy', {
        contextName: $currentContext,
        namespace: showDeleteConfirm.namespace,
        name: showDeleteConfirm.name
      });
      await loadNetworkPolicies($selectedNamespace);
    } catch (err) {
      console.error('Failed to delete policy:', err);
    } finally {
      deletingPolicy = null;
      showDeleteConfirm = null;
    }
  }

  async function openDetail(np: { name: string; namespace: string }) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'networkpolicy',
        name: np.name,
        namespace: np.namespace,
        context: $currentContext
      });
    } catch (e) {
      console.error('Failed to open detail:', e);
    }
  }

  const sortedData = $derived(() => {
    const filtered = filterBySearch($networkPolicies, filterQuery, ['name', 'namespace']);
    return sortData(filtered, sort.field, sort.direction);
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadNetworkPolicies($selectedNamespace);
    const interval = setInterval(() => loadNetworkPolicies($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    loadNetworkPolicies($selectedNamespace);
  });
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-semibold text-text-primary">Network Policies</h1>
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
        <ViewFilter value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter policies..." />
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
          <SortableHeader label="Pod Selector" field="pod_selector" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Policy Types</th>
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-20">Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each sortedData() as np}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer" onclick={() => openDetail(np)}>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{np.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{np.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <code class="text-xs bg-bg-tertiary px-2 py-0.5 rounded text-text-secondary">{np.pod_selector || '<all>'}</code>
            </td>
            <td class="py-3 pr-4">
              <div class="flex gap-1">
                {#each np.policy_types as ptype}
                  <span class="text-xs px-2 py-0.5 rounded {ptype === 'Ingress' ? 'bg-accent-success/10 text-accent-success' : 'bg-accent-warning/10 text-accent-warning'}">{ptype}</span>
                {/each}
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{np.age}</span>
            </td>
            <td class="py-3 pr-4">
              <div class="flex items-center gap-1">
                <button
                  onclick={(e) => handleEdit(e, np)}
                  disabled={loadingEdit === np.name}
                  class="p-1.5 rounded hover:bg-bg-tertiary transition-colors text-text-muted hover:text-accent-primary disabled:opacity-50"
                  title="Edit policy"
                >
                  {#if loadingEdit === np.name}
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
                  onclick={(e) => confirmDelete(e, np)}
                  disabled={deletingPolicy === np.name}
                  class="p-1.5 rounded hover:bg-accent-error/10 transition-colors text-text-muted hover:text-accent-error disabled:opacity-50"
                  title="Delete policy"
                >
                  {#if deletingPolicy === np.name}
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
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
          </svg>
          <p class="text-text-muted">No network policies found</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Network Policy Creator Modal -->
<NetworkPolicyCreator show={showCreator} editData={editingPolicy} onclose={handleCloseCreator} />

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
          <h3 class="text-lg font-semibold text-text-primary">Delete Network Policy</h3>
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
          disabled={deletingPolicy !== null}
          class="px-4 py-2 text-sm bg-accent-error text-white rounded hover:bg-accent-error/80 transition-colors disabled:opacity-50"
        >
          {deletingPolicy ? 'Deleting...' : 'Delete'}
        </button>
      </div>
    </div>
  </div>
{/if}
