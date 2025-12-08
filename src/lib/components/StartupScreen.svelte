<script lang="ts">
  import { onMount } from 'svelte';
  import {
    startupChecks,
    startupProgress,
    currentCheck,
    initError,
    runStartupChecks,
  } from '../stores/startup';

  let mounted = $state(false);
  let showRetry = $state(false);

  onMount(() => {
    mounted = true;
    startChecks();
  });

  async function startChecks() {
    showRetry = false;
    const success = await runStartupChecks();
    if (!success) {
      showRetry = true;
    }
  }

  function getStatusIcon(status: string) {
    switch (status) {
      case 'success':
        return 'M5 13l4 4L19 7';
      case 'error':
        return 'M6 18L18 6M6 6l12 12';
      case 'running':
        return 'M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15';
      default:
        return 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z';
    }
  }

  function getStatusColor(status: string) {
    switch (status) {
      case 'success':
        return 'text-accent-success';
      case 'error':
        return 'text-accent-error';
      case 'running':
        return 'text-accent-primary';
      default:
        return 'text-text-muted';
    }
  }
</script>

<div class="fixed inset-0 bg-bg-primary flex items-center justify-center">
  <div class="w-full max-w-md px-8">
    <!-- Logo -->
    <div class="text-center mb-12 {mounted ? 'animate-fade-in' : 'opacity-0'}">
      <div class="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-accent-primary to-accent-primary/50 rounded-2xl mb-6 shadow-lg shadow-accent-primary/20">
        <svg class="w-10 h-10 text-bg-primary" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
      <h1 class="text-2xl font-bold text-text-primary mb-2">Apex Kube Manager</h1>
      <p class="text-text-muted text-sm">Initializing your Kubernetes environment</p>
    </div>

    <!-- Progress Bar -->
    <div class="mb-8 {mounted ? 'animate-fade-in-delay' : 'opacity-0'}">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm text-text-secondary">{$currentCheck || 'Starting...'}</span>
        <span class="text-sm text-text-muted">{$startupProgress}%</span>
      </div>
      <div class="h-1.5 bg-bg-tertiary rounded-full overflow-hidden">
        <div
          class="h-full bg-gradient-to-r from-accent-primary to-accent-success transition-all duration-500 ease-out"
          style="width: {$startupProgress}%"
        ></div>
      </div>
    </div>

    <!-- Check List - Terminal Style Box -->
    <div class="rounded-lg border border-border-subtle bg-bg-secondary overflow-hidden {mounted ? 'animate-fade-in-delay-2' : 'opacity-0'}">
      <!-- Terminal Header -->
      <div class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary border-b border-border-subtle">
        <div class="flex gap-1.5">
          <div class="w-3 h-3 rounded-full bg-accent-error/60"></div>
          <div class="w-3 h-3 rounded-full bg-accent-warning/60"></div>
          <div class="w-3 h-3 rounded-full bg-accent-success/60"></div>
        </div>
        <span class="text-xs text-text-muted ml-2 font-mono">system checks</span>
      </div>
      <!-- Terminal Content -->
      <div class="h-48 overflow-y-auto p-3 font-mono text-sm scroll-smooth" id="check-log">
        {#each $startupChecks as check, i}
          <div
            class="flex items-start gap-2 py-1.5 transition-all duration-300"
            style="animation-delay: {i * 100}ms"
          >
            <span class="text-text-muted select-none shrink-0">$</span>
            <div class="flex items-center gap-2 min-w-0 flex-1">
              {#if check.status === 'running'}
                <svg class="w-4 h-4 text-accent-primary animate-spin shrink-0" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              {:else if check.status === 'success'}
                <svg class="w-4 h-4 text-accent-success shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              {:else if check.status === 'error'}
                <svg class="w-4 h-4 text-accent-error shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              {:else}
                <span class="w-4 h-4 shrink-0"></span>
              {/if}
              <div class="min-w-0 flex-1">
                <span class="{check.status === 'pending' ? 'text-text-muted' : check.status === 'error' ? 'text-accent-error' : 'text-text-primary'}">
                  {check.label}
                </span>
                {#if check.message && check.status !== 'pending'}
                  <span class="text-xs {check.status === 'error' ? 'text-accent-error/70' : 'text-text-muted'} block truncate">
                    → {check.message}
                  </span>
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Error State -->
    {#if $initError && showRetry}
      <div class="mt-8 p-4 bg-accent-error/10 border border-accent-error/30 rounded-lg">
        <p class="text-sm text-accent-error mb-4">{$initError}</p>
        <button
          onclick={startChecks}
          class="w-full px-4 py-2 bg-accent-error text-white rounded-lg hover:bg-accent-error/80 transition-colors"
        >
          Retry Connection
        </button>
      </div>
    {/if}

    <!-- Footer -->
    <div class="mt-12 text-center {mounted ? 'animate-fade-in-delay-3' : 'opacity-0'}">
      <p class="text-xs text-text-muted">
        Reading configuration from ~/.kube/config
      </p>
    </div>
  </div>
</div>

<style>
  @keyframes fade-in {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .animate-fade-in {
    animation: fade-in 0.5s ease-out forwards;
  }

  .animate-fade-in-delay {
    animation: fade-in 0.5s ease-out 0.2s forwards;
    opacity: 0;
  }

  .animate-fade-in-delay-2 {
    animation: fade-in 0.5s ease-out 0.4s forwards;
    opacity: 0;
  }

  .animate-fade-in-delay-3 {
    animation: fade-in 0.5s ease-out 0.6s forwards;
    opacity: 0;
  }

  /* Terminal scrollbar */
  #check-log::-webkit-scrollbar {
    width: 6px;
  }
  #check-log::-webkit-scrollbar-track {
    background: transparent;
  }
  #check-log::-webkit-scrollbar-thumb {
    background: #333333;
    border-radius: 3px;
  }
  #check-log::-webkit-scrollbar-thumb:hover {
    background: #444444;
  }
</style>
