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
  // More resources (spec 12)
  import RoleList from './views/more/RoleList.svelte';
  import ClusterRoleList from './views/more/ClusterRoleList.svelte';
  import RoleBindingList from './views/more/RoleBindingList.svelte';
  import ClusterRoleBindingList from './views/more/ClusterRoleBindingList.svelte';
  import StorageClassList from './views/more/StorageClassList.svelte';
  import EndpointSliceList from './views/more/EndpointSliceList.svelte';
  import IngressClassList from './views/more/IngressClassList.svelte';
  import PdbList from './views/more/PdbList.svelte';
  import ResourceQuotaList from './views/more/ResourceQuotaList.svelte';
  import LimitRangeList from './views/more/LimitRangeList.svelte';
  import PriorityClassList from './views/more/PriorityClassList.svelte';
  import CrdList from './views/more/CrdList.svelte';
  import CustomResourceBrowser from './views/more/CustomResourceBrowser.svelte';

  interface Props {
    view: View;
  }

  let { view }: Props = $props();
</script>

<main class="shell-main overflow-hidden bg-app min-w-0 min-h-0">
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
  {:else if view === 'network_policies'}
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
  <!-- More resources -->
  {:else if view === 'roles'}
    <RoleList />
  {:else if view === 'clusterroles'}
    <ClusterRoleList />
  {:else if view === 'rolebindings'}
    <RoleBindingList />
  {:else if view === 'clusterrolebindings'}
    <ClusterRoleBindingList />
  {:else if view === 'storageclasses'}
    <StorageClassList />
  {:else if view === 'endpointslices'}
    <EndpointSliceList />
  {:else if view === 'ingressclasses'}
    <IngressClassList />
  {:else if view === 'pdbs'}
    <PdbList />
  {:else if view === 'resourcequotas'}
    <ResourceQuotaList />
  {:else if view === 'limitranges'}
    <LimitRangeList />
  {:else if view === 'priorityclasses'}
    <PriorityClassList />
  {:else if view === 'crds'}
    <CrdList />
  {:else if view === 'customresources'}
    <CustomResourceBrowser />
  {:else}
    <div class="flex items-center justify-center h-full">
      <div class="text-center">
        <h2 class="text-lg text-text-primary mb-2 capitalize">{view}</h2>
        <p class="text-text-muted">Coming soon...</p>
      </div>
    </div>
  {/if}
</main>

<style>
  .shell-main {
    grid-column: 2;
    grid-row: 2;
  }
  @media (max-width: 639.98px) {
    .shell-main {
      grid-column: 1;
    }
  }
</style>
