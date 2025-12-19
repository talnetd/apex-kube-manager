<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { getPodLogs } from '../stores/kubernetes';
  import CustomSelect from './ui/CustomSelect.svelte';

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
  let searchQuery = $state('');
  let showPrevious = $state(false);

  async function fetchLogs() {
    isLoading = true;
    logs = await getPodLogs(namespace, podName, container, tailLines, showPrevious);
    isLoading = false;

    // Wait for DOM to update, then scroll to bottom
    await tick();
    if (logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  }

  // Filtered logs based on search
  const filteredLogs = $derived(() => {
    if (!searchQuery.trim()) return logs;
    const searchLower = searchQuery.toLowerCase();
    return logs
      .split('\n')
      .filter(line => line.toLowerCase().includes(searchLower))
      .join('\n');
  });

  // Count matching lines
  const matchCount = $derived(() => {
    if (!searchQuery.trim()) return 0;
    const searchLower = searchQuery.toLowerCase();
    return logs.split('\n').filter(line => line.toLowerCase().includes(searchLower)).length;
  });

  // Download logs
  function downloadLogs() {
    const content = searchQuery ? filteredLogs() : logs;
    const filename = `${podName}${container ? `-${container}` : ''}-${showPrevious ? 'previous-' : ''}logs-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.txt`;

    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function scrollToBottom() {
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
      <div class="w-32">
        <CustomSelect
          value={String(tailLines)}
          options={[
            { value: '50', label: 'Last 50 lines' },
            { value: '100', label: 'Last 100 lines' },
            { value: '500', label: 'Last 500 lines' },
            { value: '1000', label: 'Last 1000 lines' },
          ]}
          onchange={(v) => { tailLines = Number(v); fetchLogs(); }}
        />
      </div>

      <!-- Previous Logs Toggle -->
      <label class="flex items-center gap-2 text-xs text-text-secondary cursor-pointer">
        <input
          type="checkbox"
          bind:checked={showPrevious}
          onchange={fetchLogs}
          class="w-3 h-3 rounded border-border-subtle bg-bg-tertiary text-accent-primary focus:ring-accent-primary focus:ring-offset-0"
        />
        Previous
      </label>

      <!-- Auto Refresh Toggle -->
      <label class="flex items-center gap-2 text-xs text-text-secondary cursor-pointer">
        <input
          type="checkbox"
          bind:checked={autoRefresh}
          class="w-3 h-3 rounded border-border-subtle bg-bg-tertiary text-accent-primary focus:ring-accent-primary focus:ring-offset-0"
        />
        Tail
      </label>

      <!-- Download Button -->
      <button
        onclick={downloadLogs}
        disabled={!logs}
        class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors disabled:opacity-50"
        title="Download logs"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
      </button>

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

  <!-- Search bar -->
  <div class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary/50 border-b border-border-subtle">
    <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
    <input
      type="text"
      bind:value={searchQuery}
      placeholder="Filter logs..."
      class="flex-1 bg-transparent text-xs text-text-primary placeholder-text-muted focus:outline-none"
    />
    {#if searchQuery}
      <span class="text-xs text-text-muted">
        {matchCount()} match{matchCount() !== 1 ? 'es' : ''}
      </span>
      <button
        onclick={() => searchQuery = ''}
        class="text-text-muted hover:text-text-primary"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    {/if}
  </div>

  <!-- Log Content -->
  <div class="flex-1 overflow-auto relative">
    <pre
      bind:this={logContainer}
      class="h-full overflow-auto p-4 text-xs font-mono text-text-secondary whitespace-pre-wrap break-all"
    >{#if isLoading && !logs}Loading logs...{:else if filteredLogs()}{filteredLogs()}{:else}No logs available{/if}</pre>
    <!-- Scroll to bottom button -->
    <button
      onclick={scrollToBottom}
      class="absolute bottom-4 right-4 p-2 bg-accent-primary text-white rounded-full shadow-lg hover:bg-accent-primary/90 transition-all hover:scale-110"
      title="Scroll to bottom"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
      </svg>
    </button>
  </div>
</div>
