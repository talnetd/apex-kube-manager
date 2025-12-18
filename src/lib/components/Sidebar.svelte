<script lang="ts">
  import { currentView, sidebarCollapsed, type View } from '../stores/navigation';
  import { connectionStatus } from '../stores/kubernetes';

  interface NavItem {
    id: View;
    label: string;
    icon: string;
  }

  interface NavSection {
    title: string;
    items: NavItem[];
  }

  const navSections: NavSection[] = [
    {
      title: '',
      items: [
        { id: 'dashboard', label: 'Pulse', icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6' },
      ],
    },
    {
      title: 'Workloads',
      items: [
        { id: 'pods', label: 'Pods', icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10' },
        { id: 'deployments', label: 'Deployments', icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z' },
        { id: 'statefulsets', label: 'StatefulSets', icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10' },
        { id: 'daemonsets', label: 'DaemonSets', icon: 'M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z' },
        { id: 'replicasets', label: 'ReplicaSets', icon: 'M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z' },
        { id: 'jobs', label: 'Jobs', icon: 'M21 13.255A23.931 23.931 0 0112 15c-3.183 0-6.22-.62-9-1.745M16 6V4a2 2 0 00-2-2h-4a2 2 0 00-2 2v2m4 6h.01M5 20h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z' },
        { id: 'cronjobs', label: 'CronJobs', icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z' },
      ],
    },
    {
      title: 'Network',
      items: [
        { id: 'services', label: 'Services', icon: 'M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9' },
        { id: 'ingresses', label: 'Ingresses', icon: 'M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064' },
        { id: 'network_policies', label: 'NetPolicies', icon: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z' },
      ],
    },
    {
      title: 'Config',
      items: [
        { id: 'configmaps', label: 'ConfigMaps', icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z' },
        { id: 'secrets', label: 'Secrets', icon: 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z' },
      ],
    },
    {
      title: 'Storage',
      items: [
        { id: 'pvs', label: 'PersistentVolumes', icon: 'M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4' },
        { id: 'pvcs', label: 'PVCs', icon: 'M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4' },
      ],
    },
    {
      title: 'Cluster',
      items: [
        { id: 'nodes', label: 'Nodes', icon: 'M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01' },
        { id: 'namespaces', label: 'Namespaces', icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10' },
        { id: 'hpas', label: 'HPAs', icon: 'M13 7h8m0 0v8m0-8l-8 8-4-4-6 6' },
        { id: 'service_accounts', label: 'ServiceAccounts', icon: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z' },
      ],
    },
  ];

  function toggleSidebar() {
    sidebarCollapsed.update((v) => !v);
  }

  function navigateTo(view: View) {
    currentView.set(view);
  }
</script>

<aside
  class="flex flex-col bg-bg-secondary border-r border-border-subtle transition-all duration-200 {$sidebarCollapsed ? 'w-14' : 'w-52'}"
>
  <!-- Logo / Brand / Drag Region -->
  <div class="flex items-center h-14 px-3 border-b border-border-subtle cursor-grab active:cursor-grabbing select-none" data-tauri-drag-region>
    {#if !$sidebarCollapsed}
      <div class="flex items-center gap-2" data-tauri-drag-region>
        <div class="w-7 h-7 bg-accent-primary rounded-lg flex items-center justify-center" data-tauri-drag-region>
          <span class="text-bg-primary font-bold text-xs" data-tauri-drag-region>K8</span>
        </div>
        <span class="text-text-primary font-semibold text-sm" data-tauri-drag-region>Apex Kube</span>
      </div>
    {:else}
      <div class="w-7 h-7 bg-accent-primary rounded-lg flex items-center justify-center mx-auto" data-tauri-drag-region>
        <span class="text-bg-primary font-bold text-xs" data-tauri-drag-region>K8</span>
      </div>
    {/if}
  </div>

  <!-- Navigation -->
  <nav class="flex-1 py-2 overflow-y-auto">
    {#each navSections as section, sectionIndex}
      <!-- Section divider with title -->
      {#if section.title}
        <div class="px-3 pt-3 pb-1">
          {#if !$sidebarCollapsed}
            <span class="text-[10px] font-medium text-text-muted uppercase tracking-wider">{section.title}</span>
          {:else}
            <div class="border-t border-border-subtle"></div>
          {/if}
        </div>
      {/if}

      <!-- Items -->
      <ul class="space-y-0.5 px-2">
        {#each section.items as item}
          <li>
            <button
              onclick={() => navigateTo(item.id)}
              title={$sidebarCollapsed ? item.label : ''}
              class="w-full flex items-center gap-2.5 px-2 py-1.5 rounded-md transition-colors
                {$currentView === item.id
                  ? 'bg-accent-primary/10 text-accent-primary'
                  : 'text-text-secondary hover:bg-bg-tertiary hover:text-text-primary'}"
            >
              <svg
                class="w-4 h-4 flex-shrink-0"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="1.5"
                  d={item.icon}
                />
              </svg>
              {#if !$sidebarCollapsed}
                <span class="text-xs">{item.label}</span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/each}
  </nav>

  <!-- Connection Status & Collapse Toggle -->
  <div class="border-t border-border-subtle p-2">
    <div class="flex items-center {$sidebarCollapsed ? 'justify-center' : 'justify-between'}">
      {#if !$sidebarCollapsed}
        <div class="flex items-center gap-2">
          <div
            class="w-2 h-2 rounded-full {$connectionStatus === 'connected'
              ? 'bg-accent-success'
              : $connectionStatus === 'connecting'
              ? 'bg-accent-warning animate-pulse'
              : 'bg-accent-error'}"
          ></div>
          <span class="text-[10px] text-text-muted capitalize">{$connectionStatus}</span>
        </div>
      {/if}
      <button
        onclick={toggleSidebar}
        class="p-1 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
        title={$sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      >
        <svg
          class="w-4 h-4 transition-transform {$sidebarCollapsed ? 'rotate-180' : ''}"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M11 19l-7-7 7-7m8 14l-7-7 7-7"
          />
        </svg>
      </button>
    </div>
  </div>
</aside>
