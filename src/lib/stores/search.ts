import { writable, derived } from 'svelte/store';
import {
  pods,
  deployments,
  statefulsets,
  daemonsets,
  replicasets,
  jobs,
  cronjobs,
  services,
  ingresses,
  networkPolicies,
  configmaps,
  secrets,
  hpas,
  pvs,
  pvcs,
  namespacesInfo,
  nodes,
  serviceAccounts,
  type PodInfo,
  type DeploymentInfo,
  type StatefulSetInfo,
  type DaemonSetInfo,
  type ReplicaSetInfo,
  type JobInfo,
  type CronJobInfo,
  type ServiceInfo,
  type IngressInfo,
  type NetworkPolicyInfo,
  type ConfigMapInfo,
  type SecretInfo,
  type HPAInfo,
  type PersistentVolumeInfo,
  type PersistentVolumeClaimInfo,
  type NamespaceInfo,
  type NodeInfo,
  type ServiceAccountInfo,
} from './kubernetes';

// Search state
export const searchQuery = writable<string>('');
export const globalSearchOpen = writable<boolean>(false);

// Generic filter function for any resource type
export function filterBySearch<T extends Record<string, unknown>>(
  items: T[],
  query: string,
  fields: (keyof T)[]
): T[] {
  if (!query.trim()) return items;
  const q = query.toLowerCase();
  return items.filter((item) =>
    fields.some((field) => {
      const value = item[field];
      if (value === null || value === undefined) return false;
      if (Array.isArray(value)) {
        return value.some((v) => String(v).toLowerCase().includes(q));
      }
      return String(value).toLowerCase().includes(q);
    })
  );
}

// Search result interface
export interface SearchResults {
  pods: PodInfo[];
  deployments: DeploymentInfo[];
  statefulsets: StatefulSetInfo[];
  daemonsets: DaemonSetInfo[];
  replicasets: ReplicaSetInfo[];
  jobs: JobInfo[];
  cronjobs: CronJobInfo[];
  services: ServiceInfo[];
  ingresses: IngressInfo[];
  networkPolicies: NetworkPolicyInfo[];
  configmaps: ConfigMapInfo[];
  secrets: SecretInfo[];
  hpas: HPAInfo[];
  pvs: PersistentVolumeInfo[];
  pvcs: PersistentVolumeClaimInfo[];
  namespaces: NamespaceInfo[];
  nodes: NodeInfo[];
  serviceAccounts: ServiceAccountInfo[];
}

// Derived store for global search results
export const globalSearchResults = derived(
  [
    searchQuery,
    pods,
    deployments,
    statefulsets,
    daemonsets,
    replicasets,
    jobs,
    cronjobs,
    services,
    ingresses,
    networkPolicies,
    configmaps,
    secrets,
    hpas,
    pvs,
    pvcs,
    namespacesInfo,
    nodes,
    serviceAccounts,
  ],
  ([
    $query,
    $pods,
    $deployments,
    $statefulsets,
    $daemonsets,
    $replicasets,
    $jobs,
    $cronjobs,
    $services,
    $ingresses,
    $networkPolicies,
    $configmaps,
    $secrets,
    $hpas,
    $pvs,
    $pvcs,
    $namespaces,
    $nodes,
    $serviceAccounts,
  ]): SearchResults | null => {
    if (!$query.trim()) return null;

    return {
      pods: filterBySearch($pods, $query, ['name', 'namespace', 'status', 'node']),
      deployments: filterBySearch($deployments, $query, ['name', 'namespace']),
      statefulsets: filterBySearch($statefulsets, $query, ['name', 'namespace']),
      daemonsets: filterBySearch($daemonsets, $query, ['name', 'namespace']),
      replicasets: filterBySearch($replicasets, $query, ['name', 'namespace', 'owner']),
      jobs: filterBySearch($jobs, $query, ['name', 'namespace']),
      cronjobs: filterBySearch($cronjobs, $query, ['name', 'namespace', 'schedule']),
      services: filterBySearch($services, $query, ['name', 'namespace', 'service_type', 'cluster_ip']),
      ingresses: filterBySearch($ingresses, $query, ['name', 'namespace', 'hosts']),
      networkPolicies: filterBySearch($networkPolicies, $query, ['name', 'namespace']),
      configmaps: filterBySearch($configmaps, $query, ['name', 'namespace']),
      secrets: filterBySearch($secrets, $query, ['name', 'namespace', 'secret_type']),
      hpas: filterBySearch($hpas, $query, ['name', 'namespace', 'reference']),
      pvs: filterBySearch($pvs, $query, ['name', 'status', 'storage_class']),
      pvcs: filterBySearch($pvcs, $query, ['name', 'namespace', 'status', 'volume']),
      namespaces: filterBySearch($namespaces, $query, ['name', 'status']),
      nodes: filterBySearch($nodes, $query, ['name', 'status', 'roles']),
      serviceAccounts: filterBySearch($serviceAccounts, $query, ['name', 'namespace']),
    };
  }
);

// Helper to count total results
export function countResults(results: SearchResults | null): number {
  if (!results) return 0;
  return (
    results.pods.length +
    results.deployments.length +
    results.statefulsets.length +
    results.daemonsets.length +
    results.replicasets.length +
    results.jobs.length +
    results.cronjobs.length +
    results.services.length +
    results.ingresses.length +
    results.networkPolicies.length +
    results.configmaps.length +
    results.secrets.length +
    results.hpas.length +
    results.pvs.length +
    results.pvcs.length +
    results.namespaces.length +
    results.nodes.length +
    results.serviceAccounts.length
  );
}

// Resource type display names for bento cards
export const resourceDisplayNames: Record<keyof SearchResults, string> = {
  pods: 'Pods',
  deployments: 'Deployments',
  statefulsets: 'StatefulSets',
  daemonsets: 'DaemonSets',
  replicasets: 'ReplicaSets',
  jobs: 'Jobs',
  cronjobs: 'CronJobs',
  services: 'Services',
  ingresses: 'Ingresses',
  networkPolicies: 'Network Policies',
  configmaps: 'ConfigMaps',
  secrets: 'Secrets',
  hpas: 'HPAs',
  pvs: 'Persistent Volumes',
  pvcs: 'Persistent Volume Claims',
  namespaces: 'Namespaces',
  nodes: 'Nodes',
  serviceAccounts: 'Service Accounts',
};

// Map search result keys to resource types for detail window
export const resourceTypeMap: Record<keyof SearchResults, string> = {
  pods: 'pod',
  deployments: 'deployment',
  statefulsets: 'statefulset',
  daemonsets: 'daemonset',
  replicasets: 'replicaset',
  jobs: 'job',
  cronjobs: 'cronjob',
  services: 'service',
  ingresses: 'ingress',
  networkPolicies: 'networkpolicy',
  configmaps: 'configmap',
  secrets: 'secret',
  hpas: 'hpa',
  pvs: 'pv',
  pvcs: 'pvc',
  namespaces: 'namespace',
  nodes: 'node',
  serviceAccounts: 'serviceaccount',
};
