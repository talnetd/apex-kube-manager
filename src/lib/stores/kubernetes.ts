import { writable, derived } from 'svelte/store';

// Lazy invoke helper to ensure Tauri is ready
async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // Check if running in Tauri environment
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    throw new Error('Not running in Tauri environment');
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

export interface KubeContext {
  name: string;
  cluster: string;
  user: string;
  namespace: string | null;
  is_current: boolean;
}

export interface PodInfo {
  name: string;
  namespace: string;
  status: string;
  ready: string;
  restarts: number;
  age: string;
  node: string | null;
  ip: string | null;
  containers: ContainerInfo[];
}

export interface ContainerInfo {
  name: string;
  image: string;
  ready: boolean;
  restart_count: number;
  state: string;
}

export interface DeploymentInfo {
  name: string;
  namespace: string;
  ready: string;
  up_to_date: number;
  available: number;
  age: string;
}

export interface StatefulSetInfo {
  name: string;
  namespace: string;
  ready: string;
  replicas: number;
  age: string;
  service_name: string | null;
}

export interface DaemonSetInfo {
  name: string;
  namespace: string;
  desired: number;
  current: number;
  ready: number;
  up_to_date: number;
  available: number;
  node_selector: string | null;
  age: string;
}

export interface ReplicaSetInfo {
  name: string;
  namespace: string;
  desired: number;
  current: number;
  ready: number;
  age: string;
  owner: string | null;
}

export interface JobInfo {
  name: string;
  namespace: string;
  completions: string;
  duration: string | null;
  age: string;
  status: string;
}

export interface CronJobInfo {
  name: string;
  namespace: string;
  schedule: string;
  suspend: boolean;
  active: number;
  last_schedule: string | null;
  age: string;
}

export interface ServiceInfo {
  name: string;
  namespace: string;
  service_type: string;
  cluster_ip: string | null;
  external_ip: string | null;
  ports: string[];
  age: string;
}

// Network resources
export interface IngressInfo {
  name: string;
  namespace: string;
  class: string | null;
  hosts: string[];
  address: string | null;
  ports: string;
  age: string;
}

export interface NetworkPolicyInfo {
  name: string;
  namespace: string;
  pod_selector: string;
  policy_types: string[];
  age: string;
}

// Config resources
export interface ConfigMapInfo {
  name: string;
  namespace: string;
  data_count: number;
  age: string;
}

export interface SecretInfo {
  name: string;
  namespace: string;
  secret_type: string;
  data_count: number;
  age: string;
}

export interface HPAInfo {
  name: string;
  namespace: string;
  reference: string;
  targets: string;
  min_pods: number;
  max_pods: number;
  replicas: number;
  age: string;
}

// Storage resources
export interface PersistentVolumeInfo {
  name: string;
  capacity: string;
  access_modes: string[];
  reclaim_policy: string;
  status: string;
  claim: string | null;
  storage_class: string | null;
  age: string;
}

export interface PersistentVolumeClaimInfo {
  name: string;
  namespace: string;
  status: string;
  volume: string | null;
  capacity: string | null;
  access_modes: string[];
  storage_class: string | null;
  age: string;
}

// Cluster resources
export interface NamespaceInfo {
  name: string;
  status: string;
  age: string;
}

export interface NodeInfo {
  name: string;
  status: string;
  roles: string[];
  age: string;
  version: string;
  internal_ip: string | null;
  os_image: string;
  kernel: string;
  container_runtime: string;
}

export interface ServiceAccountInfo {
  name: string;
  namespace: string;
  secrets: number;
  age: string;
}

export interface ClusterMetrics {
  total_pods: number;
  running_pods: number;
  pending_pods: number;
  failed_pods: number;
  total_deployments: number;
  total_services: number;
  total_namespaces: number;
}

export interface ResourceCount {
  ok: number;
  fail: number;
}

export interface PulseMetrics {
  context: string;
  cluster: string;
  user: string;
  k8s_version: string;
  // Resources with OK/FAIL status
  pods: ResourceCount;
  deployments: ResourceCount;
  statefulsets: ResourceCount;
  daemonsets: ResourceCount;
  replicasets: ResourceCount;
  jobs: ResourceCount;
  // Resources without fail state (just counts)
  cronjobs: number;
  services: number;
  configmaps: number;
  secrets: number;
  pvs: number;
  pvcs: number;
  hpas: number;
  ingresses: number;
  network_policies: number;
  service_accounts: number;
  namespaces: number;
  nodes: number;
  // Resource metrics
  cpu_capacity: number;
  cpu_allocatable: number;
  memory_capacity: number;
  memory_allocatable: number;
}

// Stores
export const contexts = writable<KubeContext[]>([]);
export const currentContext = writable<string>('');
export const namespaces = writable<string[]>([]);
export const selectedNamespace = writable<string | null>(null);
export const pods = writable<PodInfo[]>([]);
export const deployments = writable<DeploymentInfo[]>([]);
export const statefulsets = writable<StatefulSetInfo[]>([]);
export const daemonsets = writable<DaemonSetInfo[]>([]);
export const replicasets = writable<ReplicaSetInfo[]>([]);
export const jobs = writable<JobInfo[]>([]);
export const cronjobs = writable<CronJobInfo[]>([]);
export const services = writable<ServiceInfo[]>([]);
// Network
export const ingresses = writable<IngressInfo[]>([]);
export const networkPolicies = writable<NetworkPolicyInfo[]>([]);
// Config
export const configmaps = writable<ConfigMapInfo[]>([]);
export const secrets = writable<SecretInfo[]>([]);
export const hpas = writable<HPAInfo[]>([]);
// Storage
export const pvs = writable<PersistentVolumeInfo[]>([]);
export const pvcs = writable<PersistentVolumeClaimInfo[]>([]);
// Cluster
export const namespacesInfo = writable<NamespaceInfo[]>([]);
export const nodes = writable<NodeInfo[]>([]);
export const serviceAccounts = writable<ServiceAccountInfo[]>([]);

export const clusterMetrics = writable<ClusterMetrics | null>(null);
export const pulseMetrics = writable<PulseMetrics | null>(null);
export const isLoading = writable<boolean>(false);
export const error = writable<string | null>(null);

// Connection status
export const connectionStatus = writable<'connected' | 'disconnected' | 'connecting'>('disconnected');

// Global refresh trigger - increment to trigger refresh in all views
export const refreshTrigger = writable<number>(0);

export function triggerRefresh() {
  refreshTrigger.update(n => n + 1);
}

// Context polling for external changes (kubectx)
let contextPollInterval: ReturnType<typeof setInterval> | null = null;

export function startContextPolling() {
  if (contextPollInterval) return;

  contextPollInterval = setInterval(async () => {
    try {
      const current = await tauriInvoke<string>('get_current_context');
      if (current !== _currentContext && current) {
        // External context change detected
        currentContext.set(current);
        selectedNamespace.set(null);
        await loadContexts();
        await loadNamespaces();
        triggerRefresh();
      }
    } catch (e) {
      // Silently ignore polling errors
    }
  }, 3000); // Check every 3 seconds
}

export function stopContextPolling() {
  if (contextPollInterval) {
    clearInterval(contextPollInterval);
    contextPollInterval = null;
  }
}

// Helper to check if context is ready
let _currentContext = '';
currentContext.subscribe(value => { _currentContext = value; });

function isContextReady(): boolean {
  return _currentContext !== '';
}

// Actions
export async function loadContexts() {
  try {
    const ctx = await tauriInvoke<KubeContext[]>('get_contexts');
    contexts.set(ctx);
    const current = await tauriInvoke<string>('get_current_context');
    currentContext.set(current);
    connectionStatus.set('connected');
  } catch (e) {
    error.set(String(e));
    connectionStatus.set('disconnected');
  }
}

export async function switchContext(contextName: string) {
  try {
    isLoading.set(true);
    await tauriInvoke('switch_context', { contextName });
    currentContext.set(contextName);
    // Reset namespace selection when switching clusters
    selectedNamespace.set(null);
    // Reload contexts to update is_current flags
    await loadContexts();
    // Reload namespaces for the new cluster
    await loadNamespaces();
    await loadClusterMetrics();
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadNamespaces() {
  if (!isContextReady()) return;
  try {
    const ns = await tauriInvoke<string[]>('get_namespaces');
    namespaces.set(ns);
  } catch (e) {
    error.set(String(e));
  }
}

export async function loadPods(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const podList = await tauriInvoke<PodInfo[]>('get_pods', { namespace });
    pods.set(podList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadDeployments(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const deployList = await tauriInvoke<DeploymentInfo[]>('get_deployments', { namespace });
    deployments.set(deployList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadStatefulSets(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const stsList = await tauriInvoke<StatefulSetInfo[]>('get_statefulsets', { namespace });
    statefulsets.set(stsList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function scaleStatefulSet(namespace: string, name: string, replicas: number) {
  if (!isContextReady()) return;
  await tauriInvoke('scale_statefulset', { namespace, name, replicas });
}

export async function restartStatefulSet(namespace: string, name: string) {
  if (!isContextReady()) return;
  await tauriInvoke('restart_statefulset', { namespace, name });
}

export async function loadDaemonSets(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const dsList = await tauriInvoke<DaemonSetInfo[]>('get_daemonsets', { namespace });
    daemonsets.set(dsList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadReplicaSets(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const rsList = await tauriInvoke<ReplicaSetInfo[]>('get_replicasets', { namespace });
    replicasets.set(rsList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadJobs(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const jobList = await tauriInvoke<JobInfo[]>('get_jobs', { namespace });
    jobs.set(jobList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadCronJobs(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const cjList = await tauriInvoke<CronJobInfo[]>('get_cronjobs', { namespace });
    cronjobs.set(cjList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadServices(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const svcList = await tauriInvoke<ServiceInfo[]>('get_services', { namespace });
    services.set(svcList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

// Network resources
export async function loadIngresses(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const ingList = await tauriInvoke<IngressInfo[]>('get_ingresses', { namespace });
    ingresses.set(ingList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadNetworkPolicies(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const npList = await tauriInvoke<NetworkPolicyInfo[]>('get_network_policies', { namespace });
    networkPolicies.set(npList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

// Config resources
export async function loadConfigMaps(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const cmList = await tauriInvoke<ConfigMapInfo[]>('get_configmaps', { namespace });
    configmaps.set(cmList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadSecrets(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const secList = await tauriInvoke<SecretInfo[]>('get_secrets', { namespace });
    secrets.set(secList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadHPAs(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const hpaList = await tauriInvoke<HPAInfo[]>('get_hpas', { namespace });
    hpas.set(hpaList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

// Storage resources
export async function loadPVs() {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const pvList = await tauriInvoke<PersistentVolumeInfo[]>('get_pvs');
    pvs.set(pvList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadPVCs(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const pvcList = await tauriInvoke<PersistentVolumeClaimInfo[]>('get_pvcs', { namespace });
    pvcs.set(pvcList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

// Cluster resources
export async function loadNamespacesInfo() {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const nsList = await tauriInvoke<NamespaceInfo[]>('get_namespaces_info');
    namespacesInfo.set(nsList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadNodes() {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const nodeList = await tauriInvoke<NodeInfo[]>('get_nodes');
    nodes.set(nodeList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadServiceAccounts(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    isLoading.set(true);
    const saList = await tauriInvoke<ServiceAccountInfo[]>('get_service_accounts', { namespace });
    serviceAccounts.set(saList);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
  }
}

export async function loadClusterMetrics() {
  if (!isContextReady()) return;
  try {
    const metrics = await tauriInvoke<ClusterMetrics>('get_cluster_metrics');
    clusterMetrics.set(metrics);
  } catch (e) {
    error.set(String(e));
  }
}

export async function loadPulseMetrics(namespace?: string | null) {
  if (!isContextReady()) return;
  try {
    const metrics = await tauriInvoke<PulseMetrics>('get_pulse_metrics', { namespace });
    pulseMetrics.set(metrics);
  } catch (e) {
    error.set(String(e));
  }
}

export async function deletePod(namespace: string, podName: string) {
  if (!isContextReady()) return;
  try {
    await tauriInvoke('delete_pod', { namespace, podName });
    await loadPods(namespace);
  } catch (e) {
    error.set(String(e));
  }
}

export async function getPodLogs(
  namespace: string,
  podName: string,
  container?: string,
  tailLines?: number,
  previous?: boolean
): Promise<string> {
  if (!isContextReady()) return '';
  try {
    return await tauriInvoke<string>('get_pod_logs', {
      namespace,
      podName,
      container,
      tailLines,
      previous,
    });
  } catch (e) {
    error.set(String(e));
    return '';
  }
}

export async function openTerminalWindow(
  namespace: string,
  podName: string,
  container?: string
) {
  if (!isContextReady()) return;
  try {
    await tauriInvoke('open_terminal_window', {
      podName,
      namespace,
      context: _currentContext,
      container: container || null,
    });
  } catch (e) {
    error.set(String(e));
  }
}

// Derived stores
export const filteredPods = derived(
  [pods, selectedNamespace],
  ([$pods, $selectedNamespace]) => {
    if (!$selectedNamespace) return $pods;
    return $pods.filter((pod) => pod.namespace === $selectedNamespace);
  }
);

export const podsByStatus = derived(pods, ($pods) => {
  return {
    all: $pods,
    running: $pods.filter((p) => p.status === 'Running'),
    pending: $pods.filter((p) => p.status === 'Pending'),
    failed: $pods.filter((p) => p.status === 'Failed'),
    succeeded: $pods.filter((p) => p.status === 'Succeeded'),
  };
});
