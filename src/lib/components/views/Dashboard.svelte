<script lang="ts">
  import { onMount } from 'svelte';
  import { pulseMetrics, selectedNamespace, currentContext, loadPulseMetrics, type ResourceCount } from '../../stores/kubernetes';
  import { currentView, type View } from '../../stores/navigation';

  let lastRefresh = $state(new Date());
  let refreshInterval: ReturnType<typeof setInterval>;

  // Calculate real CPU/Memory usage percentages from metrics
  const cpuUsage = $derived(() => {
    if (!$pulseMetrics?.metrics_available || !$pulseMetrics.cpu_usage) return 0;
    const usage = $pulseMetrics.cpu_usage;
    const allocatable = $pulseMetrics.cpu_allocatable;
    if (allocatable <= 0) return 0;
    return Math.min(100, (usage / allocatable) * 100);
  });

  const memoryUsage = $derived(() => {
    if (!$pulseMetrics?.metrics_available || !$pulseMetrics.memory_usage) return 0;
    const usage = $pulseMetrics.memory_usage;
    const allocatable = $pulseMetrics.memory_allocatable;
    if (allocatable <= 0) return 0;
    return Math.min(100, (usage / allocatable) * 100);
  });

  // React to namespace and context changes
  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    const ns = $selectedNamespace;
    loadPulseMetrics(ns);
    lastRefresh = new Date();
  });

  onMount(() => {
    refreshInterval = setInterval(() => {
      loadPulseMetrics($selectedNamespace);
      lastRefresh = new Date();
    }, 5000);

    return () => {
      clearInterval(refreshInterval);
    };
  });

  // Resources with OK/FAIL status
  interface StatusCard {
    label: string;
    key: 'pods' | 'deployments' | 'statefulsets' | 'daemonsets' | 'replicasets' | 'jobs';
    icon: string;
    color: string;
    view: View;
    hasStatus: true;
  }

  // Resources with just count
  interface CountCard {
    label: string;
    key: 'cronjobs' | 'services' | 'configmaps' | 'secrets' | 'pvs' | 'pvcs' | 'hpas' | 'ingresses' | 'network_policies' | 'service_accounts';
    icon: string;
    color: string;
    view: View;
    hasStatus: false;
  }

  type ResourceCard = StatusCard | CountCard;

  const resourceCards: ResourceCard[] = [
    { label: 'Pods', key: 'pods', icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10', color: 'accent-primary', view: 'pods', hasStatus: true },
    { label: 'Deployments', key: 'deployments', icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z', color: 'accent-success', view: 'deployments', hasStatus: true },
    { label: 'StatefulSets', key: 'statefulsets', icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10', color: 'accent-warning', view: 'statefulsets', hasStatus: true },
    { label: 'DaemonSets', key: 'daemonsets', icon: 'M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z', color: 'purple-400', view: 'daemonsets', hasStatus: true },
    { label: 'ReplicaSets', key: 'replicasets', icon: 'M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z', color: 'blue-400', view: 'replicasets', hasStatus: true },
    { label: 'Jobs', key: 'jobs', icon: 'M21 13.255A23.931 23.931 0 0112 15c-3.183 0-6.22-.62-9-1.745M16 6V4a2 2 0 00-2-2h-4a2 2 0 00-2 2v2m4 6h.01M5 20h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z', color: 'cyan-400', view: 'jobs', hasStatus: true },
    { label: 'CronJobs', key: 'cronjobs', icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z', color: 'pink-400', view: 'cronjobs', hasStatus: false },
    { label: 'Services', key: 'services', icon: 'M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9', color: 'accent-primary', view: 'services', hasStatus: false },
    { label: 'ConfigMaps', key: 'configmaps', icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z', color: 'amber-400', view: 'configmaps', hasStatus: false },
    { label: 'Secrets', key: 'secrets', icon: 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z', color: 'accent-error', view: 'secrets', hasStatus: false },
    { label: 'PVs', key: 'pvs', icon: 'M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4', color: 'indigo-400', view: 'pvs', hasStatus: false },
    { label: 'PVCs', key: 'pvcs', icon: 'M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4', color: 'violet-400', view: 'pvcs', hasStatus: false },
    { label: 'HPAs', key: 'hpas', icon: 'M13 7h8m0 0v8m0-8l-8 8-4-4-6 6', color: 'emerald-400', view: 'hpas', hasStatus: false },
    { label: 'Ingresses', key: 'ingresses', icon: 'M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064', color: 'orange-400', view: 'ingresses', hasStatus: false },
    { label: 'NetPolicies', key: 'network_policies', icon: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z', color: 'rose-400', view: 'network_policies', hasStatus: false },
    { label: 'ServiceAccounts', key: 'service_accounts', icon: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z', color: 'teal-400', view: 'serviceaccounts', hasStatus: false },
  ];

  function navigateToView(view: View) {
    currentView.set(view);
  }

  // Geist: neutral cards; only a failure count is coloured.
  function getColorClass(_color: string, _hasFail: boolean = false) {
    return 'text-secondary border-line bg-app';
  }

  function getValue(card: ResourceCard): { ok: number; fail: number } | number {
    if (!$pulseMetrics) return card.hasStatus ? { ok: 0, fail: 0 } : 0;

    if (card.hasStatus) {
      return $pulseMetrics[card.key] as ResourceCount;
    } else {
      return $pulseMetrics[card.key] as number;
    }
  }

  function formatCpu(millicores: number): string {
    if (millicores >= 1000) {
      return `${(millicores / 1000).toFixed(1)}`;
    }
    return `${millicores}m`;
  }

  function formatMemory(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) {
      return `${gb.toFixed(1)}`;
    }
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(0)}`;
  }

  function getStrokeDasharray(percentage: number, radius: number): string {
    const circumference = 2 * Math.PI * radius;
    const filled = (percentage / 100) * circumference;
    return `${filled} ${circumference}`;
  }
</script>

<div class="h-full overflow-y-auto bg-bg-primary">
  {#if $pulseMetrics}
    <!-- Pulse Title -->
    <div class="px-6 py-4 border-b border-border-subtle">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <h1 class="text-lg font-semibold text-text-primary">Pulse</h1>
          {#if $selectedNamespace}
            <span class="px-2 py-0.5 border border-line-strong text-secondary text-base rounded-sm">
              {$selectedNamespace}
            </span>
          {/if}
        </div>
        <div class="text-xs text-text-muted">
          Last updated: {lastRefresh.toLocaleTimeString()}
        </div>
      </div>
    </div>

    <!-- CPU & Memory Gauges -->
    <div class="px-6 pt-6">
      {#if $pulseMetrics.metrics_available}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
          <!-- CPU Gauge -->
          <div class="relative bg-app rounded-lg border border-line p-6 overflow-hidden">
            <div class="relative flex items-center gap-6">
              <!-- Ring Gauge -->
              <div class="relative w-32 h-32 shrink-0">
                <svg class="w-full h-full -rotate-90" viewBox="0 0 120 120">
                  <!-- Background ring -->
                  <circle
                    cx="60"
                    cy="60"
                    r="52"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="12"
                    class="text-active"
                  />
                  <!-- Progress ring -->
                  <circle
                    cx="60"
                    cy="60"
                    r="52"
                    fill="none"
                    stroke="var(--ui-accent)"
                    stroke-width="12"
                    stroke-linecap="round"
                    stroke-dasharray={getStrokeDasharray(cpuUsage(), 52)}
                    class="transition-all duration-300"
                  />
                </svg>
                <!-- Center text -->
                <div class="absolute inset-0 flex flex-col items-center justify-center">
                  <span class="text-xl font-bold text-text-primary">{cpuUsage().toFixed(0)}%</span>
                  <span class="text-xs text-text-muted">usage</span>
                </div>
              </div>

              <!-- Info -->
              <div class="flex-1">
                <div class="flex items-center gap-2 mb-2">
                  <svg class="w-5 h-5 text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                  </svg>
                  <h3 class="text-[18px]/7 font-semibold text-text-primary">CPU</h3>
                </div>
                <div class="space-y-1">
                  <div class="flex justify-between text-base">
                    <span class="text-text-muted">Usage</span>
                    <span class="text-muted font-mono">{formatCpu($pulseMetrics.cpu_usage ?? 0)} cores</span>
                  </div>
                  <div class="flex justify-between text-base">
                    <span class="text-text-muted">Allocatable</span>
                    <span class="text-text-primary font-mono">{formatCpu($pulseMetrics.cpu_allocatable)} cores</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Memory Gauge -->
          <div class="relative bg-app rounded-lg border border-line p-6 overflow-hidden">
            <div class="relative flex items-center gap-6">
              <!-- Ring Gauge -->
              <div class="relative w-32 h-32 shrink-0">
                <svg class="w-full h-full -rotate-90" viewBox="0 0 120 120">
                  <!-- Background ring -->
                  <circle
                    cx="60"
                    cy="60"
                    r="52"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="12"
                    class="text-active"
                  />
                  <!-- Progress ring -->
                  <circle
                    cx="60"
                    cy="60"
                    r="52"
                    fill="none"
                    stroke="var(--ui-accent)"
                    stroke-width="12"
                    stroke-linecap="round"
                    stroke-dasharray={getStrokeDasharray(memoryUsage(), 52)}
                    class="transition-all duration-300"
                  />
                </svg>
                <!-- Center text -->
                <div class="absolute inset-0 flex flex-col items-center justify-center">
                  <span class="text-xl font-bold text-text-primary">{memoryUsage().toFixed(0)}%</span>
                  <span class="text-xs text-text-muted">usage</span>
                </div>
              </div>

              <!-- Info -->
              <div class="flex-1">
                <div class="flex items-center gap-2 mb-2">
                  <svg class="w-5 h-5 text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
                  </svg>
                  <h3 class="text-[18px]/7 font-semibold text-text-primary">Memory</h3>
                </div>
                <div class="space-y-1">
                  <div class="flex justify-between text-base">
                    <span class="text-text-muted">Usage</span>
                    <span class="text-muted font-mono">{formatMemory($pulseMetrics.memory_usage ?? 0)} Gi</span>
                  </div>
                  <div class="flex justify-between text-base">
                    <span class="text-text-muted">Allocatable</span>
                    <span class="text-text-primary font-mono">{formatMemory($pulseMetrics.memory_allocatable)} Gi</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      {:else}
        <!-- Metrics unavailable message -->
        <div class="mb-6 bg-bg-secondary rounded-lg border border-border-subtle p-6">
          <div class="flex items-center gap-4">
            <div class="p-3 rounded-lg bg-transparent">
              <svg class="w-6 h-6 text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div class="flex-1">
              <h3 class="text-base font-medium text-text-primary">Metrics Unavailable</h3>
              <p class="text-base text-text-muted">Install metrics-server to view cluster CPU and memory usage</p>
            </div>
            <div class="text-right text-base">
              <div class="text-text-muted">Allocatable</div>
              <div class="text-text-primary font-mono">{formatCpu($pulseMetrics.cpu_allocatable)} cores, {formatMemory($pulseMetrics.memory_allocatable)} Gi</div>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Resource Grid -->
    <div class="px-6 pb-6">
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-3">
        {#each resourceCards as card}
          {@const value = getValue(card)}
          {@const hasStatus = card.hasStatus && typeof value === 'object'}
          {@const hasFail = hasStatus && (value as ResourceCount).fail > 0}
          <button
            onclick={() => navigateToView(card.view)}
            class="group relative overflow-hidden rounded-lg border {getColorClass(card.color, hasFail)} p-3 transition-colors hover:bg-hover text-left cursor-pointer"
          >

            <div class="relative">
              <div class="flex items-center justify-between mb-1">
                <div class="opacity-50">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={card.icon} />
                  </svg>
                </div>
              </div>
              {#if hasStatus}
                {@const statusValue = value as ResourceCount}
                <div class="text-lg font-bold">
                  <span class="text-primary">{statusValue.ok}</span>
                  {#if statusValue.fail > 0}
                    <span class="text-text-muted">:</span>
                    <span class="text-danger">{statusValue.fail}</span>
                  {/if}
                </div>
              {:else}
                <div class="text-xl font-bold">{value}</div>
              {/if}
              <div class="text-xs opacity-70 mt-0.5">{card.label}</div>
            </div>
          </button>
        {/each}
      </div>
    </div>
  {:else}
    <!-- Loading State -->
    <div class="flex items-center justify-center h-full">
      <div class="text-center">
        <div class="inline-flex items-center justify-center w-16 h-16 mb-4">
          <svg class="w-10 h-10 text-accent-text animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>
        <p class="text-text-muted">Loading cluster metrics...</p>
      </div>
    </div>
  {/if}
</div>
