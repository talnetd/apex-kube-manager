<script lang="ts">
  import { onMount } from 'svelte';
  import { pulseMetrics, selectedNamespace, currentContext, loadPulseMetrics, type ResourceCount } from '../../stores/kubernetes';
  import { currentView, type View } from '../../stores/navigation';

  let lastRefresh = $state(new Date());
  let refreshInterval: ReturnType<typeof setInterval>;

  // Simulated usage percentages (animating for visual effect)
  let cpuUsage = $state(0);
  let memoryUsage = $state(0);
  let targetCpuUsage = $state(45);
  let targetMemoryUsage = $state(62);

  // React to namespace and context changes
  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    const ns = $selectedNamespace;
    loadPulseMetrics(ns);
    lastRefresh = new Date();
  });

  onMount(() => {
    // Animate gauges on mount
    const animateGauges = () => {
      // Randomize target slightly for "live" feel
      targetCpuUsage = Math.min(85, Math.max(25, targetCpuUsage + (Math.random() - 0.5) * 8));
      targetMemoryUsage = Math.min(90, Math.max(35, targetMemoryUsage + (Math.random() - 0.5) * 6));
    };

    // Smooth animation towards target
    const smoothAnimation = setInterval(() => {
      cpuUsage += (targetCpuUsage - cpuUsage) * 0.1;
      memoryUsage += (targetMemoryUsage - memoryUsage) * 0.1;
    }, 50);

    // Update targets periodically for "live" effect
    const targetInterval = setInterval(animateGauges, 3000);

    refreshInterval = setInterval(() => {
      loadPulseMetrics($selectedNamespace);
      lastRefresh = new Date();
    }, 5000);

    // Initial animation
    setTimeout(() => {
      cpuUsage = 0;
      memoryUsage = 0;
      animateGauges();
    }, 100);

    return () => {
      clearInterval(refreshInterval);
      clearInterval(smoothAnimation);
      clearInterval(targetInterval);
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
    { label: 'ServiceAccounts', key: 'service_accounts', icon: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z', color: 'teal-400', view: 'service_accounts', hasStatus: false },
  ];

  function navigateToView(view: View) {
    currentView.set(view);
  }

  function getColorClass(color: string, hasFail: boolean = false) {
    const colorMap: Record<string, string> = {
      'accent-primary': 'text-accent-primary border-accent-primary/30 bg-accent-primary/10',
      'accent-success': 'text-accent-success border-accent-success/30 bg-accent-success/10',
      'accent-warning': 'text-accent-warning border-accent-warning/30 bg-accent-warning/10',
      'accent-error': 'text-accent-error border-accent-error/30 bg-accent-error/10',
      'purple-400': 'text-purple-400 border-purple-400/30 bg-purple-400/10',
      'blue-400': 'text-blue-400 border-blue-400/30 bg-blue-400/10',
      'cyan-400': 'text-cyan-400 border-cyan-400/30 bg-cyan-400/10',
      'pink-400': 'text-pink-400 border-pink-400/30 bg-pink-400/10',
      'amber-400': 'text-amber-400 border-amber-400/30 bg-amber-400/10',
      'indigo-400': 'text-indigo-400 border-indigo-400/30 bg-indigo-400/10',
      'violet-400': 'text-violet-400 border-violet-400/30 bg-violet-400/10',
      'emerald-400': 'text-emerald-400 border-emerald-400/30 bg-emerald-400/10',
      'orange-400': 'text-orange-400 border-orange-400/30 bg-orange-400/10',
      'rose-400': 'text-rose-400 border-rose-400/30 bg-rose-400/10',
      'teal-400': 'text-teal-400 border-teal-400/30 bg-teal-400/10',
    };
    if (hasFail) {
      return colorMap[color]?.replace(/border-[^\s]+/, 'border-accent-error/50') || colorMap['accent-primary'];
    }
    return colorMap[color] || colorMap['accent-primary'];
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
          <h1 class="text-xl font-semibold text-text-primary">Pulse</h1>
          {#if $selectedNamespace}
            <span class="px-2 py-0.5 bg-accent-primary/20 text-accent-primary text-sm rounded">
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
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
        <!-- CPU Gauge -->
        <div class="relative bg-bg-secondary rounded-xl border border-border-subtle p-6 overflow-hidden">
          <!-- Animated background glow -->
          <div class="absolute inset-0 opacity-20">
            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-48 h-48 bg-cyan-500 rounded-full blur-3xl animate-pulse"></div>
          </div>

          <div class="relative flex items-center gap-6">
            <!-- Ring Gauge -->
            <div class="relative w-32 h-32 flex-shrink-0">
              <svg class="w-full h-full -rotate-90" viewBox="0 0 120 120">
                <!-- Background ring -->
                <circle
                  cx="60"
                  cy="60"
                  r="52"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="12"
                  class="text-bg-tertiary"
                />
                <!-- Progress ring -->
                <circle
                  cx="60"
                  cy="60"
                  r="52"
                  fill="none"
                  stroke="url(#cpuGradient)"
                  stroke-width="12"
                  stroke-linecap="round"
                  stroke-dasharray={getStrokeDasharray(cpuUsage, 52)}
                  class="transition-all duration-100"
                />
                <defs>
                  <linearGradient id="cpuGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#06b6d4" />
                    <stop offset="100%" stop-color="#3b82f6" />
                  </linearGradient>
                </defs>
              </svg>
              <!-- Center text -->
              <div class="absolute inset-0 flex flex-col items-center justify-center">
                <span class="text-2xl font-bold text-text-primary">{cpuUsage.toFixed(0)}%</span>
                <span class="text-xs text-text-muted">usage</span>
              </div>
            </div>

            <!-- Info -->
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-2">
                <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
                </svg>
                <h3 class="text-lg font-semibold text-text-primary">CPU</h3>
              </div>
              <div class="space-y-1">
                <div class="flex justify-between text-sm">
                  <span class="text-text-muted">Allocatable</span>
                  <span class="text-text-primary font-mono">{formatCpu($pulseMetrics.cpu_allocatable)} cores</span>
                </div>
                <div class="flex justify-between text-sm">
                  <span class="text-text-muted">Capacity</span>
                  <span class="text-text-primary font-mono">{formatCpu($pulseMetrics.cpu_capacity)} cores</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Memory Gauge -->
        <div class="relative bg-bg-secondary rounded-xl border border-border-subtle p-6 overflow-hidden">
          <!-- Animated background glow -->
          <div class="absolute inset-0 opacity-20">
            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-48 h-48 bg-emerald-500 rounded-full blur-3xl animate-pulse"></div>
          </div>

          <div class="relative flex items-center gap-6">
            <!-- Ring Gauge -->
            <div class="relative w-32 h-32 flex-shrink-0">
              <svg class="w-full h-full -rotate-90" viewBox="0 0 120 120">
                <!-- Background ring -->
                <circle
                  cx="60"
                  cy="60"
                  r="52"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="12"
                  class="text-bg-tertiary"
                />
                <!-- Progress ring -->
                <circle
                  cx="60"
                  cy="60"
                  r="52"
                  fill="none"
                  stroke="url(#memGradient)"
                  stroke-width="12"
                  stroke-linecap="round"
                  stroke-dasharray={getStrokeDasharray(memoryUsage, 52)}
                  class="transition-all duration-100"
                />
                <defs>
                  <linearGradient id="memGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stop-color="#10b981" />
                    <stop offset="100%" stop-color="#8b5cf6" />
                  </linearGradient>
                </defs>
              </svg>
              <!-- Center text -->
              <div class="absolute inset-0 flex flex-col items-center justify-center">
                <span class="text-2xl font-bold text-text-primary">{memoryUsage.toFixed(0)}%</span>
                <span class="text-xs text-text-muted">usage</span>
              </div>
            </div>

            <!-- Info -->
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-2">
                <svg class="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
                </svg>
                <h3 class="text-lg font-semibold text-text-primary">Memory</h3>
              </div>
              <div class="space-y-1">
                <div class="flex justify-between text-sm">
                  <span class="text-text-muted">Allocatable</span>
                  <span class="text-text-primary font-mono">{formatMemory($pulseMetrics.memory_allocatable)} Gi</span>
                </div>
                <div class="flex justify-between text-sm">
                  <span class="text-text-muted">Capacity</span>
                  <span class="text-text-primary font-mono">{formatMemory($pulseMetrics.memory_capacity)} Gi</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
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
            class="group relative overflow-hidden rounded-lg border {getColorClass(card.color, hasFail)} p-3 transition-all duration-200 hover:scale-[1.02] hover:shadow-lg text-left cursor-pointer"
          >
            <!-- Background glow effect -->
            <div class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300 bg-gradient-to-br from-current/5 to-transparent"></div>

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
                <div class="text-xl font-bold">
                  <span class="text-accent-success">{statusValue.ok}</span>
                  {#if statusValue.fail > 0}
                    <span class="text-text-muted">:</span>
                    <span class="text-accent-error">{statusValue.fail}</span>
                  {/if}
                </div>
              {:else}
                <div class="text-2xl font-bold">{value}</div>
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
          <svg class="w-10 h-10 text-accent-primary animate-spin" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>
        <p class="text-text-muted">Loading cluster metrics...</p>
      </div>
    </div>
  {/if}
</div>
