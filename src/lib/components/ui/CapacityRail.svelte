<script lang="ts">
  import { onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import { pulseRailOpen } from '../../stores/rail';
  import {
    pulseMetrics,
    clusterEvents,
    selectedNamespace,
    currentContext,
    loadPulseMetrics,
  } from '../../stores/kubernetes';

  // Keep capacity fresh while the rail is mounted.
  onMount(() => {
    loadPulseMetrics($selectedNamespace);
    const id = setInterval(() => loadPulseMetrics($selectedNamespace), 10000);
    return () => clearInterval(id);
  });

  $effect(() => {
    const ns = $selectedNamespace;
    const ctx = $currentContext;
    if (ctx) loadPulseMetrics(ns);
  });

  // Same unit handling as the Dashboard: CPU in millicores, memory in bytes.
  function formatCpu(millicores: number): string {
    if (millicores >= 1000) return `${(millicores / 1000).toFixed(1)} cores`;
    return `${Math.round(millicores)}m`;
  }
  function formatMemory(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) return `${gb.toFixed(1)} Gi`;
    return `${(bytes / (1024 * 1024)).toFixed(0)} Mi`;
  }

  const cpuPct = $derived(() => {
    const m = $pulseMetrics;
    if (!m?.metrics_available || !m.cpu_usage || !m.cpu_allocatable) return 0;
    return Math.min(100, (m.cpu_usage / m.cpu_allocatable) * 100);
  });
  const memPct = $derived(() => {
    const m = $pulseMetrics;
    if (!m?.metrics_available || !m.memory_usage || !m.memory_allocatable) return 0;
    return Math.min(100, (m.memory_usage / m.memory_allocatable) * 100);
  });

  // clusterEvents is maintained newest-first by the watch.
  const recentEvents = $derived(() => $clusterEvents.slice(0, 14));

  function evTag(type: string): string {
    return type === 'Warning' ? '[WARN]' : '[ ok ]';
  }
  function evColor(type: string): string {
    return type === 'Warning' ? 'text-accent-warning' : 'text-accent-success';
  }
</script>

{#if $pulseRailOpen}
  <aside
    transition:fly={{ x: 280, duration: 160 }}
    class="fixed right-0 top-14 bottom-0 w-72 z-30 flex flex-col
           bg-bg-secondary border-l border-border-subtle overflow-hidden"
  >
    <!-- header -->
    <div class="flex items-center justify-between px-3 h-9 border-b border-border-subtle">
      <span class="text-[10px] tracking-[0.2em] text-text-muted font-mono">▚ PULSE</span>
      <button
        onclick={() => pulseRailOpen.set(false)}
        title="Close (⌘E)"
        class="text-text-muted hover:text-text-primary transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-3 py-3 flex flex-col gap-5">
      <!-- CAPACITY -->
      <section>
        <div class="text-[10px] tracking-[0.18em] text-text-muted font-mono mb-2">CAPACITY</div>

        {#if !$pulseMetrics}
          <div class="text-xs text-text-muted font-mono">reading metrics…</div>
        {:else if $pulseMetrics.metrics_available}
          <!-- CPU -->
          <div class="mb-3">
            <div class="flex justify-between text-xs text-text-secondary mb-1 font-mono">
              <span>cpu</span>
              <span class="text-text-primary tabular-nums">
                {formatCpu($pulseMetrics.cpu_usage ?? 0)} / {formatCpu($pulseMetrics.cpu_allocatable)}
              </span>
            </div>
            <div class="h-1.5 rounded-sm bg-bg-tertiary border border-border-subtle overflow-hidden">
              <div class="h-full bg-accent-primary" style="width:{cpuPct()}%"></div>
            </div>
          </div>
          <!-- MEM -->
          <div>
            <div class="flex justify-between text-xs text-text-secondary mb-1 font-mono">
              <span>mem</span>
              <span class="text-text-primary tabular-nums">
                {formatMemory($pulseMetrics.memory_usage ?? 0)} / {formatMemory($pulseMetrics.memory_allocatable)}
              </span>
            </div>
            <div class="h-1.5 rounded-sm bg-bg-tertiary border border-border-subtle overflow-hidden">
              <div class="h-full bg-accent-secondary" style="width:{memPct()}%"></div>
            </div>
          </div>
        {:else}
          <!-- No metrics-server: still show allocatable capacity + a hint. -->
          <div class="text-xs text-accent-warning font-mono mb-2">metrics-server not detected</div>
          <div class="text-xs text-text-secondary font-mono leading-relaxed">
            <div class="flex justify-between"><span>cpu</span><span class="text-text-primary tabular-nums">{formatCpu($pulseMetrics.cpu_allocatable)}</span></div>
            <div class="flex justify-between"><span>mem</span><span class="text-text-primary tabular-nums">{formatMemory($pulseMetrics.memory_allocatable)}</span></div>
            <div class="text-text-muted mt-1">allocatable · install metrics-server for live usage</div>
          </div>
        {/if}
      </section>

      <!-- EVENTS -->
      <section class="flex-1 min-h-0">
        <div class="text-[10px] tracking-[0.18em] text-text-muted font-mono mb-2">EVENTS ›_</div>
        {#if recentEvents().length === 0}
          <div class="text-xs text-text-muted font-mono">no events · cluster's quiet</div>
        {:else}
          <div class="flex flex-col gap-1.5 font-mono text-[11px] leading-snug">
            {#each recentEvents() as ev}
              <div class="truncate" title="{ev.reason}: {ev.message}">
                <span class={evColor(ev.type)}>{evTag(ev.type)}</span>
                <span class="text-text-secondary">{ev.involved_kind?.toLowerCase()}/{ev.involved_object}</span>
                <span class="text-text-muted">{ev.reason}</span>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>
  </aside>
{/if}
