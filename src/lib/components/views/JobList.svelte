<script lang="ts">
  import { onMount } from 'svelte';
  import {
    jobs,
    selectedNamespace,
    currentContext,
    loadJobs,
    isLoading,
  } from '../../stores/kubernetes';

  onMount(() => {
    loadJobs($selectedNamespace);
    const interval = setInterval(() => loadJobs($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadJobs($selectedNamespace);
  });

  function getStatusClass(status: string): string {
    switch (status) {
      case 'Complete':
        return 'text-accent-success bg-accent-success/10';
      case 'Running':
        return 'text-accent-primary bg-accent-primary/10';
      case 'Failed':
        return 'text-accent-error bg-accent-error/10';
      default:
        return 'text-accent-warning bg-accent-warning/10';
    }
  }

  function getStatusDot(status: string): string {
    switch (status) {
      case 'Complete':
        return 'bg-accent-success';
      case 'Running':
        return 'bg-accent-primary';
      case 'Failed':
        return 'bg-accent-error';
      default:
        return 'bg-accent-warning';
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">Jobs</h1>
        <p class="text-text-muted mt-1">Run-to-completion workloads</p>
      </div>
      <button
        onclick={() => loadJobs($selectedNamespace)}
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
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Completions</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Duration</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Status</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-20"></th>
        </tr>
      </thead>
      <tbody>
        {#each $jobs as job}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full {getStatusDot(job.status)}"></div>
                <span class="text-text-primary font-medium">{job.name}</span>
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{job.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{job.completions}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{job.duration || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="px-2 py-0.5 rounded text-xs font-medium {getStatusClass(job.status)}">{job.status}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{job.age}</span>
            </td>
            <td class="py-3">
              <div class="flex items-center gap-1">
                <button
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
                  title="View Logs"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                </button>
                <button
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-error transition-colors"
                  title="Delete"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $jobs.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
          </svg>
          <p class="text-text-muted">No jobs found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
