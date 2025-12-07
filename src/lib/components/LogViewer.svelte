<script lang="ts">
  import { onMount } from 'svelte';
  import { getPodLogs } from '../stores/kubernetes';

  interface Props {
    namespace: string;
    podName: string;
    container?: string;
    onClose?: () => void;
  }

  let { namespace, podName, container, onClose }: Props = $props();

  let logs = $state('');
  let isLoading = $state(true);
  let tailLines = $state(100);
  let autoRefresh = $state(false);
  let logContainer: HTMLPreElement;

  async function fetchLogs() {
    isLoading = true;
    logs = await getPodLogs(namespace, podName, container, tailLines);
    isLoading = false;

    // Scroll to bottom
    if (logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  }

  onMount(() => {
    fetchLogs();

    let interval: ReturnType<typeof setInterval> | null = null;

    const unsubscribe = $effect.root(() => {
      $effect(() => {
        if (autoRefresh) {
          interval = setInterval(fetchLogs, 5000);
        } else if (interval) {
          clearInterval(interval);
          interval = null;
        }
      });
    });

    return () => {
      unsubscribe();
      if (interval) clearInterval(interval);
    };
  });
</script>

<div class="flex flex-col h-full bg-bg-primary">
  <!-- Log Viewer Header -->
  <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
    <div class="flex items-center gap-3">
      <svg class="w-4 h-4 text-accent-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <div>
        <span class="text-sm text-text-primary font-medium">{podName}</span>
        <span class="text-xs text-text-muted ml-2">{namespace}</span>
        {#if container}
          <span class="text-xs text-text-muted ml-2">/ {container}</span>
        {/if}
      </div>
    </div>

    <div class="flex items-center gap-3">
      <!-- Tail Lines Selector -->
      <select
        bind:value={tailLines}
        onchange={fetchLogs}
        class="px-2 py-1 text-xs bg-bg-tertiary border border-border-subtle rounded text-text-primary focus:outline-none focus:border-accent-primary"
      >
        <option value={50}>Last 50 lines</option>
        <option value={100}>Last 100 lines</option>
        <option value={500}>Last 500 lines</option>
        <option value={1000}>Last 1000 lines</option>
      </select>

      <!-- Auto Refresh Toggle -->
      <label class="flex items-center gap-2 text-xs text-text-secondary cursor-pointer">
        <input
          type="checkbox"
          bind:checked={autoRefresh}
          class="w-3 h-3 rounded border-border-subtle bg-bg-tertiary text-accent-primary focus:ring-accent-primary focus:ring-offset-0"
        />
        Auto-refresh
      </label>

      <!-- Refresh Button -->
      <button
        onclick={fetchLogs}
        class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
        disabled={isLoading}
      >
        <svg class="w-4 h-4 {isLoading ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </button>

      {#if onClose}
        <button
          onclick={onClose}
          class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}
    </div>
  </div>

  <!-- Log Content -->
  <pre
    bind:this={logContainer}
    class="flex-1 overflow-auto p-4 text-xs font-mono text-text-secondary whitespace-pre-wrap break-all"
  >{#if isLoading && !logs}Loading logs...{:else if logs}{logs}{:else}No logs available{/if}</pre>
</div>
