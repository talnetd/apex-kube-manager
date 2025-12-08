<script lang="ts">
  import { onMount } from 'svelte';
  import Terminal from '../Terminal.svelte';

  // Parameters from URL
  let context = $state<string>('');
  let namespace = $state<string>('');
  let podName = $state<string>('');
  let container = $state<string | undefined>(undefined);
  let isLoaded = $state<boolean>(false);
  let loadError = $state<string | null>(null);

  onMount(() => {
    // Parse URL parameters
    const params = new URLSearchParams(window.location.search);

    context = params.get('context') || '';
    namespace = params.get('namespace') || '';
    podName = params.get('pod') || '';
    const containerParam = params.get('container');
    container = containerParam || undefined;

    if (!podName || !namespace) {
      loadError = 'Missing pod name or namespace';
    } else {
      isLoaded = true;
    }

    // Update window title
    const containerSuffix = container ? ` (${container})` : '';
    document.title = `Terminal: ${podName}${containerSuffix}`;
  });
</script>

<div class="h-screen flex flex-col bg-bg-primary overflow-hidden">
  {#if loadError}
    <div class="flex items-center justify-center h-full">
      <div class="text-center">
        <svg class="w-16 h-16 text-accent-error mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <p class="text-text-muted">{loadError}</p>
      </div>
    </div>
  {:else if !isLoaded}
    <div class="flex items-center justify-center h-full">
      <div class="text-center">
        <svg class="w-8 h-8 text-accent-primary mx-auto mb-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        <p class="text-text-muted">Loading...</p>
      </div>
    </div>
  {:else}
    <Terminal
      {namespace}
      {podName}
      {container}
    />
  {/if}
</div>
