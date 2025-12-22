<script lang="ts">
  import type { View } from '../stores/navigation';
  import Dashboard from './views/Dashboard.svelte';
  // Workloads
  import PodList from './views/PodList.svelte';
  import DeploymentList from './views/DeploymentList.svelte';
  import StatefulSetList from './views/StatefulSetList.svelte';
  import DaemonSetList from './views/DaemonSetList.svelte';
  import ReplicaSetList from './views/ReplicaSetList.svelte';
  import JobList from './views/JobList.svelte';
  import CronJobList from './views/CronJobList.svelte';
  // Network
  import ServiceList from './views/ServiceList.svelte';
  import IngressList from './views/IngressList.svelte';
  import NetworkPolicyList from './views/NetworkPolicyList.svelte';
  // Config
  import ConfigMapList from './views/ConfigMapList.svelte';
  import SecretList from './views/SecretList.svelte';
  import HPAList from './views/HPAList.svelte';
  // Storage
  import PVList from './views/PVList.svelte';
  import PVCList from './views/PVCList.svelte';
  // Cluster
  import NamespaceList from './views/NamespaceList.svelte';
  import NodeList from './views/NodeList.svelte';
  import ServiceAccountList from './views/ServiceAccountList.svelte';
  import EventList from './views/EventList.svelte';

  interface Props {
    view: View;
  }

  let { view }: Props = $props();
</script>

<main class="flex-1 overflow-hidden bg-bg-primary">
  {#if view === 'dashboard'}
    <Dashboard />
  <!-- Workloads -->
  {:else if view === 'pods'}
    <PodList />
  {:else if view === 'deployments'}
    <DeploymentList />
  {:else if view === 'statefulsets'}
    <StatefulSetList />
  {:else if view === 'daemonsets'}
    <DaemonSetList />
  {:else if view === 'replicasets'}
    <ReplicaSetList />
  {:else if view === 'jobs'}
    <JobList />
  {:else if view === 'cronjobs'}
    <CronJobList />
  <!-- Network -->
  {:else if view === 'services'}
    <ServiceList />
  {:else if view === 'ingresses'}
    <IngressList />
  {:else if view === 'networkpolicies'}
    <NetworkPolicyList />
  <!-- Config -->
  {:else if view === 'configmaps'}
    <ConfigMapList />
  {:else if view === 'secrets'}
    <SecretList />
  {:else if view === 'hpas'}
    <HPAList />
  <!-- Storage -->
  {:else if view === 'pvs'}
    <PVList />
  {:else if view === 'pvcs'}
    <PVCList />
  <!-- Cluster -->
  {:else if view === 'namespaces'}
    <NamespaceList />
  {:else if view === 'nodes'}
    <NodeList />
  {:else if view === 'serviceaccounts'}
    <ServiceAccountList />
  {:else if view === 'events'}
    <EventList />
  {:else}
    <div class="flex items-center justify-center h-full">
      <div class="text-center">
        <h2 class="text-xl text-text-primary mb-2 capitalize">{view}</h2>
        <p class="text-text-muted">Coming soon...</p>
      </div>
    </div>
  {/if}
</main>
