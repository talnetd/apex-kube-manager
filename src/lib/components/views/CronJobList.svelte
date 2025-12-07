<script lang="ts">
  import { onMount } from 'svelte';
  import {
    cronjobs,
    selectedNamespace,
    currentContext,
    loadCronJobs,
    isLoading,
  } from '../../stores/kubernetes';

  onMount(() => {
    loadCronJobs($selectedNamespace);
    const interval = setInterval(() => loadCronJobs($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadCronJobs($selectedNamespace);
  });
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">CronJobs</h1>
        <p class="text-text-muted mt-1">Schedule recurring jobs</p>
      </div>
      <button
        onclick={() => loadCronJobs($selectedNamespace)}
        class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary border border-border-subtle rounded-lg text-text-primary hover:border-accent-primary transition-colors"
      >
        <svg class="w-4 h-4 {$isLoading ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        Refresh
      </button>
    </div>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto px-6 pb-6">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Name</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Namespace</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Schedule</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Suspend</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Active</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Last Schedule</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-20"></th>
        </tr>
      </thead>
      <tbody>
        {#each $cronjobs as cj}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full {cj.suspend ? 'bg-accent-warning' : 'bg-accent-success'}"></div>
                <span class="text-text-primary font-medium">{cj.name}</span>
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{cj.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <code class="text-xs bg-bg-tertiary px-2 py-0.5 rounded text-accent-primary">{cj.schedule}</code>
            </td>
            <td class="py-3 pr-4">
              {#if cj.suspend}
                <span class="px-2 py-0.5 rounded text-xs font-medium text-accent-warning bg-accent-warning/10">Suspended</span>
              {:else}
                <span class="px-2 py-0.5 rounded text-xs font-medium text-accent-success bg-accent-success/10">Active</span>
              {/if}
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{cj.active}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{cj.last_schedule || 'Never'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{cj.age}</span>
            </td>
            <td class="py-3">
              <div class="flex items-center gap-1">
                <button
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
                  title="Trigger Now"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </button>
                <button
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
                  title="{cj.suspend ? 'Resume' : 'Suspend'}"
                >
                  {#if cj.suspend}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    </svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                  {/if}
                </button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $cronjobs.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-text-muted">No cronjobs found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
