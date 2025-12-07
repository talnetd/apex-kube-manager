<script lang="ts">
  import { onMount } from 'svelte';
  import { secrets, selectedNamespace, currentContext, loadSecrets, isLoading } from '../../stores/kubernetes';

  onMount(() => {
    loadSecrets($selectedNamespace);
    const interval = setInterval(() => loadSecrets($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadSecrets($selectedNamespace);
  });

  function getTypeColor(type: string): string {
    if (type.includes('tls')) return 'text-accent-primary bg-accent-primary/10';
    if (type.includes('dockerconfig')) return 'text-accent-warning bg-accent-warning/10';
    if (type.includes('service-account')) return 'text-accent-success bg-accent-success/10';
    return 'text-text-muted bg-bg-tertiary';
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">Secrets</h1>
        <p class="text-text-muted mt-1">Sensitive data like passwords, tokens, and keys</p>
      </div>
      <button onclick={() => loadSecrets($selectedNamespace)} class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary border border-border-subtle rounded-lg text-text-primary hover:border-accent-primary transition-colors">
        <svg class="w-4 h-4 {$isLoading ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        Refresh
      </button>
    </div>
  </div>

  <div class="flex-1 overflow-auto px-6 pb-6">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Name</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Namespace</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Type</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Data</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
        </tr>
      </thead>
      <tbody>
        {#each $secrets as sec}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <div class="flex items-center gap-2">
                <svg class="w-4 h-4 text-accent-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
                <span class="text-text-primary font-medium">{sec.name}</span>
              </div>
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
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $secrets.length === 0}
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
