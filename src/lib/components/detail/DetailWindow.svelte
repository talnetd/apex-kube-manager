<script lang="ts">
  import { onMount } from 'svelte';
  // Complex components with special features (logs, exec, scaling)
  import PodDetail from './PodDetail.svelte';
  import DeploymentDetail from './DeploymentDetail.svelte';
  import StatefulSetDetail from './StatefulSetDetail.svelte';
  // Unified component for all other resource types
  import ResourceDetail from './ResourceDetail.svelte';

  // Resource types that can be displayed
  type ResourceType = 'pod' | 'deployment' | 'statefulset' | 'daemonset' | 'replicaset' |
                      'job' | 'cronjob' | 'service' | 'ingress' | 'configmap' | 'secret' |
                      'pv' | 'pvc' | 'node' | 'namespace' | 'serviceaccount' | 'hpa' | 'networkpolicy';

  // Resource types that use the unified ResourceDetail component
  const unifiedResourceTypes = new Set<ResourceType>([
    'daemonset', 'replicaset', 'job', 'cronjob', 'service', 'ingress',
    'configmap', 'secret', 'networkpolicy', 'hpa', 'pv', 'pvc',
    'namespace', 'node', 'serviceaccount'
  ]);

  // Locked context from URL params - never changes after mount
  let lockedContext = $state<string>('');
  let lockedNamespace = $state<string>('');
  let resourceType = $state<ResourceType>('pod');
  let resourceName = $state<string>('');
  let isLoaded = $state<boolean>(false);
  let loadError = $state<string | null>(null);

  onMount(() => {
    // Parse URL parameters - these are locked for the lifetime of this window
    const params = new URLSearchParams(window.location.search);

    lockedContext = params.get('context') || '';
    lockedNamespace = params.get('namespace') || '';
    resourceType = (params.get('type') as ResourceType) || 'pod';
    resourceName = params.get('name') || '';

    if (!resourceName) {
      loadError = 'No resource name specified';
    } else {
      isLoaded = true;
    }

    // Update window title (capitalize resource type only)
    const capitalizedType = resourceType.charAt(0).toUpperCase() + resourceType.slice(1);
    document.title = `${capitalizedType}: ${resourceName}`;
  });

  function getResourceIcon(type: ResourceType): string {
    const icons: Record<ResourceType, string> = {
      pod: 'M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4',
      deployment: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z',
      statefulset: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
      daemonset: 'M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z',
      replicaset: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6z',
      job: 'M21 13.255A23.931 23.931 0 0112 15c-3.183 0-6.22-.62-9-1.745M16 6V4a2 2 0 00-2-2h-4a2 2 0 00-2 2v2m4 6h.01M5 20h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z',
      cronjob: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
      service: 'M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9',
      ingress: 'M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064',
      configmap: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
      secret: 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z',
      pv: 'M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4',
      pvc: 'M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4',
      node: 'M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01',
      namespace: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
      serviceaccount: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z',
      hpa: 'M13 7h8m0 0v8m0-8l-8 8-4-4-6 6',
      networkpolicy: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z'
    };
    return icons[type] || icons.pod;
  }
</script>

<div class="h-screen flex flex-col bg-bg-primary overflow-hidden">
  <!-- Header with locked context info -->
  <header class="flex items-center justify-between px-4 py-3 bg-bg-secondary border-b border-border-subtle">
    <div class="flex items-center gap-3">
      <svg class="w-5 h-5 text-accent-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={getResourceIcon(resourceType)} />
      </svg>
      <div>
        <h1 class="text-lg font-semibold text-text-primary"><span class="capitalize">{resourceType}</span>: {resourceName}</h1>
        <div class="flex items-center gap-2 text-xs text-text-muted">
          <span class="flex items-center gap-1">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2" />
            </svg>
            {lockedContext}
          </span>
          {#if lockedNamespace}
            <span class="text-text-muted">/</span>
            <span class="flex items-center gap-1">
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
              {lockedNamespace}
            </span>
          {/if}
        </div>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-xs px-2 py-1 bg-accent-primary/10 text-accent-primary rounded">
        Locked View
      </span>
    </div>
  </header>

  <!-- Main content area -->
  <main class="flex-1 overflow-hidden">
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
    <!-- Special components with complex features -->
    {:else if resourceType === 'pod'}
      <PodDetail
        context={lockedContext}
        namespace={lockedNamespace}
        name={resourceName}
      />
    {:else if resourceType === 'deployment'}
      <DeploymentDetail
        context={lockedContext}
        namespace={lockedNamespace}
        name={resourceName}
      />
    {:else if resourceType === 'statefulset'}
      <StatefulSetDetail
        context={lockedContext}
        namespace={lockedNamespace}
        name={resourceName}
      />
    <!-- Unified component for all other resource types -->
    {:else if unifiedResourceTypes.has(resourceType)}
      <ResourceDetail
        resourceType={resourceType}
        context={lockedContext}
        namespace={lockedNamespace}
        name={resourceName}
      />
    {:else}
      <!-- Fallback for unknown resource types -->
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-text-muted mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d={getResourceIcon(resourceType)} />
          </svg>
          <h2 class="text-xl text-text-primary mb-2"><span class="capitalize">{resourceType}</span> Details</h2>
          <p class="text-text-muted">Detail view coming soon...</p>
        </div>
      </div>
    {/if}
  </main>
</div>
