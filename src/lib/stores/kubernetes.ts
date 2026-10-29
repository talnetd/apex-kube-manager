import { writable, derived, get, type Readable, type Writable } from 'svelte/store';
import { settings } from './settings';
import { verdictToHealth, type Verdict } from '../utils/health';

export type { Verdict };

// Lazy invoke helper to ensure Tauri is ready. The import is shared, so
// concurrent first calls resolve the same module.
let tauriCore: Promise<typeof import('@tauri-apps/api/core')> | null = null;
async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  // Check if running in Tauri environment
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    throw new Error('Not running in Tauri environment');
  }
  const { invoke } = await (tauriCore ??= import('@tauri-apps/api/core'));
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
  /** Health from the shared triage rules (Rust, spec/triage/rules.md). Never re-derive it here. */
  verdict: Verdict;
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

export interface PodWatchEvent {
  event_type: 'applied' | 'deleted';
  pod: PodInfo;
  timestamp: string;
}

export interface DeploymentWatchEvent {
  event_type: 'applied' | 'deleted';
  deployment: DeploymentInfo;
  timestamp: string;
}

export interface JobWatchEvent {
  event_type: 'applied' | 'deleted';
  job: JobInfo;
  timestamp: string;
}

export interface NodeWatchEvent {
  event_type: 'applied' | 'deleted';
  node: NodeInfo;
  timestamp: string;
}

export interface DeploymentInfo {
  /** Health from the shared triage rules (Rust, spec/triage/rules.md). Never re-derive it here. */
  verdict: Verdict;
  name: string;
  namespace: string;
  ready: string;
  /** spec.replicas (desired). `ready` counts surge pods during rollouts. */
  desired: number;
  up_to_date: number;
  available: number;
  age: string;
}

export interface StatefulSetInfo {
  /** Health from the shared triage rules (Rust, spec/triage/rules.md). Never re-derive it here. */
  verdict: Verdict;
  name: string;
  namespace: string;
  ready: string;
  replicas: number;
  age: string;
  service_name: string | null;
}

export interface DaemonSetInfo {
  /** Health from the shared triage rules (Rust, spec/triage/rules.md). Never re-derive it here. */
  verdict: Verdict;
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
  /** Health from the shared triage rules (Rust, spec/triage/rules.md). Never re-derive it here. */
  verdict: Verdict;
  name: string;
  namespace: string;
  desired: number;
  current: number;
  ready: number;
  age: string;
  owner: string | null;
}

export interface JobInfo {
  /** Health from the shared triage rules (Rust, spec/triage/rules.md). Never re-derive it here. */
  verdict: Verdict;
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
  /** Health from the shared triage rules (Rust, spec/triage/rules.md). Never re-derive it here. */
  verdict: Verdict;
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

export interface NodeTaintInfo {
  key: string;
  value: string | null;
  effect: string;
}

export interface NodeInfo {
  /** Health from the shared triage rules (Rust, spec/triage/rules.md). Never re-derive it here. */
  verdict: Verdict;
  name: string;
  status: string;
  roles: string[];
  age: string;
  version: string;
  internal_ip: string | null;
  os_image: string;
  kernel: string;
  container_runtime: string;
  taints: NodeTaintInfo[];
  // Denominators for the usage columns
  cpu_allocatable: number;    // millicores
  memory_allocatable: number; // bytes
}

export interface NodeMetricsInfo {
  name: string;
  // CPU in millicores
  cpu_usage: number | null;
  // Memory in bytes
  memory_usage: number | null;
  // Root filesystem in bytes
  disk_usage: number | null;
  disk_capacity: number | null;
}

export interface ServiceAccountInfo {
  name: string;
  namespace: string;
  secrets: number;
  age: string;
}

export interface ClusterEventInfo {
  name: string;
  namespace: string;
  type: string;
  reason: string;
  message: string;
  involved_object: string;
  involved_kind: string;
  source: string;
  count: number;
  first_seen: string | null;
  last_seen: string | null;
  age: string;
}

export interface ClusterEventWatchEvent {
  event_type: 'applied' | 'deleted';
  event: ClusterEventInfo;
  timestamp: string;
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
  // Actual usage from metrics-server (null if unavailable)
  cpu_usage: number | null;
  memory_usage: number | null;
  metrics_available: boolean;
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
// Keyed by node name — merged into the node list rows.
export const nodeMetrics = writable<Record<string, NodeMetricsInfo>>({});
export const serviceAccounts = writable<ServiceAccountInfo[]>([]);
export const clusterEvents = writable<ClusterEventInfo[]>([]);

export const clusterMetrics = writable<ClusterMetrics | null>(null);
export const pulseMetrics = writable<PulseMetrics | null>(null);
export const isLoading = writable<boolean>(false);
export const isSwitchingContext = writable<boolean>(false);
export const error = writable<string | null>(null);

// ---- Per-resource load state (spec 5.4) ----
// DataTable needs to tell "still connecting" from "empty" from "RBAC said no".
export type ResourceStatus = 'idle' | 'loading' | 'ready' | 'error' | 'forbidden';
export interface ResourceState {
  status: ResourceStatus;
  error?: string;
  /** ms epoch of the last successful list or watch event. */
  lastSyncedAt?: number;
}

const resourceStates = new Map<string, ReturnType<typeof writable<ResourceState>>>();

/** Load state for one resource list (e.g. 'pods'). Created on first use. */
export function resourceState(name: string) {
  let s = resourceStates.get(name);
  if (!s) {
    s = writable<ResourceState>({ status: 'idle' });
    resourceStates.set(name, s);
  }
  return s;
}

/** Last successful sync from any resource (status bar freshness, spec 4.4). */
export const lastSyncedAt = writable<number | null>(null);

export function markLoading(name: string) {
  resourceState(name).update((s) => ({ ...s, status: 'loading', error: undefined }));
}

export function markSynced(name: string, at = Date.now()) {
  resourceState(name).set({ status: 'ready', lastSyncedAt: at });
  lastSyncedAt.set(at);
}

/** 403s come through as error text until the error type carries a code. */
export function isForbiddenError(e: unknown): boolean {
  const msg = String(e);
  return /\b403\b|forbidden/i.test(msg);
}

export function markFailed(name: string, e: unknown) {
  resourceState(name).update((s) => ({
    ...s,
    status: isForbiddenError(e) ? 'forbidden' : 'error',
    error: String(e),
  }));
}

// Connection status
export const connectionStatus = writable<'connected' | 'disconnected' | 'connecting'>('disconnected');

// Watch stream state (module-level to persist across component mounts)
type UnlistenFn = () => void;
let podWatchUnlisten: UnlistenFn | null = null;
let currentPodWatchId: string | null = null;
// Context the `pods` store currently holds.
let podsContext: string | null = null;

// Global refresh trigger - increment to trigger refresh in all views
export const refreshTrigger = writable<number>(0);

export function triggerRefresh() {
  refreshTrigger.update(n => n + 1);
}

// ---------- Shared context (spec 13.3) ----------
// Rust owns the context: a switch from any window, the menu bar, or an
// outside kubectx change is broadcast once as `context-changed { context,
// seq }`, and every window applies it here. `seq` drops out-of-order events.
export interface ContextChangedEvent {
  context: string;
  seq: number;
}

let lastContextSeq = 0;
let contextUnlisten: UnlistenFn | null = null;

/** Accept an event only if it's newer than the last one applied (pure, tested). */
export function acceptContextEvent(lastSeq: number, ev: ContextChangedEvent): boolean {
  return ev.seq > lastSeq && !!ev.context;
}

/**
 * Bring this window's stores onto `contextName`: stop the old cluster's
 * watches, drop anything that could collide across clusters, reload.
 * Namespace selection is per window and resets on a switch.
 */
async function applyContextChange(contextName: string) {
  if (!contextName || contextName === _currentContext) return;
  isSwitchingContext.set(true);
  try {
    pulseMetrics.set(null);
    apiProbes.set({ ok: 0, total: 0, since: Date.now() });
    nodeMetrics.set({});
    await stopPodWatch();
    await stopDeploymentWatch();
    await stopJobWatch();
    await stopNodeWatch();
    await stopEventWatch();
    // Node names collide across clusters: never show the old cluster's nodes.
    nodes.set([]);
    currentContext.set(contextName);
    selectedNamespace.set(null);
    await loadContexts();
    await loadNamespaces();
    await loadClusterMetrics();
    triggerRefresh();
  } finally {
    isSwitchingContext.set(false);
  }
}

/** Listen for shared context changes (main windows). Idempotent. */
export async function startContextPolling() {
  if (contextUnlisten || typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return;
  const { listen } = await import('@tauri-apps/api/event');
  contextUnlisten = await listen<ContextChangedEvent>('context-changed', (event) => {
    if (!acceptContextEvent(lastContextSeq, event.payload)) return;
    lastContextSeq = event.payload.seq;
    applyContextChange(event.payload.context).catch((e) => error.set(String(e)));
  });
}

export function stopContextPolling() {
  contextUnlisten?.();
  contextUnlisten = null;
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

/**
 * Switch the shared context. Rust writes the kubeconfig once and tells
 * every window; this window applies it right away (the broadcast that
 * follows is then a no-op here, by name).
 */
export async function switchContext(contextName: string) {
  try {
    isLoading.set(true);
    isSwitchingContext.set(true);
    await tauriInvoke('switch_context', { contextName });
    await applyContextChange(contextName);
  } catch (e) {
    error.set(String(e));
  } finally {
    isLoading.set(false);
    isSwitchingContext.set(false);
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

// Pod Watch Stream Functions
//
// Every start/stop bumps `podWatchGen`. Each await in the setup re-checks
// it, and so does the listener: a superseded setup tears down whatever it
// already created (watch id, listener) and never writes to `pods`. That
// covers a namespace switch racing a context switch, and a slow first list
// resolving after a newer watch started.
let podWatchGen = 0;

// Watch events are applied in batches (every 100ms) so a burst of updates
// costs one copy/regroup/sort instead of one per event.
const POD_BATCH_MS = 100;
let podEventQueue: PodWatchEvent[] = [];
let podFlushTimer: ReturnType<typeof setTimeout> | null = null;

const podKey = (p: Pick<PodInfo, 'namespace' | 'name'>) => `${p.namespace}/${p.name}`;

/** Apply queued watch events to a pod list in order (last event per pod wins). Pure. */
export function applyPodEvents(current: PodInfo[], events: PodWatchEvent[]): PodInfo[] {
  if (events.length === 0) return current;
  const index = new Map(current.map((p, i) => [podKey(p), i]));
  const next = [...current];
  const removed = new Set<number>();
  for (const { event_type, pod } of events) {
    const k = podKey(pod);
    const i = index.get(k);
    if (event_type === 'applied') {
      if (i !== undefined && !removed.has(i)) next[i] = pod;
      else {
        index.set(k, next.length);
        next.push(pod);
      }
    } else if (event_type === 'deleted' && i !== undefined) {
      removed.add(i);
      index.delete(k);
    }
  }
  return removed.size ? next.filter((_, i) => !removed.has(i)) : next;
}

function flushPodEvents(gen: number) {
  podFlushTimer = null;
  const events = podEventQueue;
  podEventQueue = [];
  if (gen !== podWatchGen || events.length === 0) return;
  pods.update((current) => applyPodEvents(current, events));
  markSynced('pods');
}

function queuePodEvent(gen: number, event: PodWatchEvent) {
  if (gen !== podWatchGen) return; // superseded watch: drop
  podEventQueue.push(event);
  if (!podFlushTimer) podFlushTimer = setTimeout(() => flushPodEvents(gen), POD_BATCH_MS);
}

/** Tear down the current listener + backend watch and drop queued events. */
async function teardownPodWatch() {
  const unlisten = podWatchUnlisten;
  const watchId = currentPodWatchId;
  podWatchUnlisten = null;
  currentPodWatchId = null;
  podEventQueue = [];
  if (podFlushTimer) {
    clearTimeout(podFlushTimer);
    podFlushTimer = null;
  }
  unlisten?.();
  if (watchId) {
    try {
      await tauriInvoke('stop_watch', { watchId });
      console.log(`Pod watch stopped: ${watchId}`);
    } catch (e) {
      console.error('Failed to stop pod watch:', e);
    }
  }
}

export async function startPodWatch(namespace?: string | null) {
  const gen = ++podWatchGen;
  await teardownPodWatch();
  if (gen !== podWatchGen) return;

  if (!isContextReady()) return;

  // Pod names collide across clusters: never show the old context's pods
  // under the new one while the first list is in flight.
  const ctx = _currentContext;
  if (podsContext !== ctx) {
    pods.set([]);
    podsContext = ctx;
  }
  markLoading('pods');

  // A setup step finished after a newer start/stop: undo what we made.
  const superseded = () => gen !== podWatchGen || _currentContext !== ctx;

  try {
    const initialPods = await tauriInvoke<PodInfo[]>('get_pods', { namespace });
    if (superseded()) return;
    pods.set(initialPods);
    markSynced('pods');

    const watchId = await tauriInvoke<string>('watch_pods', { namespace });
    if (superseded()) {
      tauriInvoke('stop_watch', { watchId }).catch(() => {});
      return;
    }
    currentPodWatchId = watchId;

    const { listen } = await import('@tauri-apps/api/event');
    if (superseded()) {
      // currentPodWatchId is ours only if nobody tore down in between.
      if (currentPodWatchId === watchId) {
        currentPodWatchId = null;
        tauriInvoke('stop_watch', { watchId }).catch(() => {});
      }
      return;
    }

    const unlisten = await listen<PodWatchEvent>(`pod-watch-${watchId}`, (event) =>
      queuePodEvent(gen, event.payload)
    );
    if (superseded()) {
      unlisten();
      if (currentPodWatchId === watchId) {
        currentPodWatchId = null;
        tauriInvoke('stop_watch', { watchId }).catch(() => {});
      }
      return;
    }
    podWatchUnlisten = unlisten;

    console.log(`Pod watch started: ${watchId}`);
  } catch (e) {
    if (superseded()) return;
    error.set(String(e));
    markFailed('pods', e);
    console.error('Failed to start pod watch:', e);
  }
}

export async function stopPodWatch() {
  podWatchGen++;
  await teardownPodWatch();
}

// ---- Lists: one model for every list view (spec 5.4) ----
//
// A list source is rows + load state + the context those rows came from.
// Rows clear the moment the context changes, so a name from one cluster is
// never shown, opened or edited under another. Polled lists add a loader
// (`listStore`); watched lists add a watch (`listWatch`) on the same source.
// Both use a generation counter so an older answer never overwrites a newer
// one, and a context check so a late answer from the previous cluster is
// dropped.

export interface ListSource<T> {
  /** resourceState key, e.g. 'deployments'. */
  name: string;
  items: Writable<T[]>;
  state: Readable<ResourceState>;
  /** Context the rows on screen were loaded from ('' when there are none). */
  context: Readable<string>;
  namespaced: boolean;
  /** Clears rows and state; also runs on every context change. */
  reset: () => void;
}

export interface ListLoadOptions {
  /**
   * A background tick: skipped while a load is in flight, so a slow cluster
   * isn't asked again before it answers and the slow answer still lands.
   * Any other load (mount, `r`, namespace change) supersedes the in-flight one.
   */
  poll?: boolean;
  /** Override the scope, e.g. a cluster-scoped CRD in the custom resource browser. */
  namespaced?: boolean;
}

export interface ListStore<T> extends ListSource<T> {
  load: (namespace?: string | null, extra?: Record<string, unknown>, opts?: ListLoadOptions) => Promise<void>;
}

// Context setters, kept off the public ListSource shape.
const sourceContext = new WeakMap<ListSource<unknown>, (c: string) => void>();

function listSource<T>(name: string, items: Writable<T[]>, namespaced: boolean, onReset?: () => void): ListSource<T> {
  const state = resourceState(name);
  const context = writable('');
  function reset() {
    onReset?.();
    items.set([]);
    context.set('');
    state.set({ status: 'idle' });
  }
  let seen = _currentContext;
  currentContext.subscribe((c) => {
    if (c === seen) return;
    seen = c;
    if (get(context) !== c) reset();
  });
  const src: ListSource<T> = {
    name,
    items,
    state: { subscribe: state.subscribe },
    context: { subscribe: context.subscribe },
    namespaced,
    reset,
  };
  sourceContext.set(src as ListSource<unknown>, context.set);
  return src;
}

const setSourceContext = (src: ListSource<unknown>, ctx: string) => sourceContext.get(src)?.(ctx);

/** A polled list. `name` keys its resourceState; `items` reuses an existing store. */
export function listStore<T>(
  command: string,
  namespaced: boolean,
  opts: { name?: string; items?: Writable<T[]> } = {}
): ListStore<T> {
  const name = opts.name ?? command;
  let gen = 0;
  let inFlight = false;
  let scope: string | null = null;
  const src = listSource<T>(name, opts.items ?? writable<T[]>([]), namespaced, () => {
    gen++;
    inFlight = false;
    scope = null;
  });

  async function load(namespace?: string | null, extra: Record<string, unknown> = {}, o: ListLoadOptions = {}) {
    if (!isContextReady()) return;
    if (o.poll && inFlight) return;
    const ctx = _currentContext;
    const scoped = o.namespaced ?? namespaced;
    const args = scoped ? { namespace: namespace ?? null, ...extra } : { ...extra };
    const key = JSON.stringify(args);
    const g = ++gen;
    inFlight = true;
    // Rows stay up on a refresh. A new scope (namespace) shows the loading
    // bar over them; a plain re-poll doesn't.
    if (key !== scope || get(src.state).status !== 'ready') markLoading(name);
    scope = key;
    const live = () => g === gen && _currentContext === ctx;
    try {
      const rows = await tauriInvoke<T[]>(command, args);
      if (!live()) return;
      src.items.set(rows);
      setSourceContext(src as ListSource<unknown>, ctx);
      markSynced(name);
    } catch (e) {
      if (!live()) return;
      markFailed(name, e);
    } finally {
      if (g === gen) inFlight = false;
    }
  }

  const store = src as ListStore<T>;
  store.load = load;
  return store;
}

/**
 * Watch on a list source: initial list, then applied/deleted events. Every
 * start/stop bumps the generation, and each await re-checks it (and the
 * context), so a start that finishes after a newer one undoes its own work
 * and never writes rows. Double starts (Retry, `r`) can't leak a listener.
 */
function listWatch<T, E extends { event_type: 'applied' | 'deleted' }>(opts: {
  label: string;
  list: string;
  watch: string;
  channel: string;
  source: ListSource<T>;
  key: (x: T) => string;
  item: (e: E) => T;
  /** New rows go first (events: newest at the top before sorting). */
  prepend?: boolean;
  /** Called for each streamed (not initial) applied row. */
  onApplied?: (x: T, isNew: boolean) => void;
}) {
  const { source } = opts;
  let gen = 0;
  let unlisten: UnlistenFn | null = null;
  let watchId: string | null = null;

  const stopRemote = (id: string) =>
    tauriInvoke('stop_watch', { watchId: id }).catch((e) => console.error(`Failed to stop ${opts.label} watch:`, e));

  async function teardown() {
    const u = unlisten;
    const id = watchId;
    unlisten = null;
    watchId = null;
    u?.();
    if (id) await stopRemote(id);
  }

  async function start(args?: Record<string, unknown>) {
    const g = ++gen;
    await teardown();
    if (g !== gen || !isContextReady()) return;
    const ctx = _currentContext;
    const superseded = () => g !== gen || _currentContext !== ctx;
    // Undo a watch we created if someone else took over meanwhile.
    const drop = (id: string) => {
      if (watchId === id) watchId = null;
      stopRemote(id);
    };

    markLoading(source.name);
    try {
      const initial = await tauriInvoke<T[]>(opts.list, args);
      if (superseded()) return;
      source.items.set(initial);
      setSourceContext(source as ListSource<unknown>, ctx);
      markSynced(source.name);

      const id = await tauriInvoke<string>(opts.watch, args);
      if (superseded()) return void stopRemote(id);
      watchId = id;

      const { listen } = await import('@tauri-apps/api/event');
      if (superseded()) return drop(id);

      const u = await listen<E>(`${opts.channel}-${id}`, (event) => {
        if (superseded()) return; // superseded stream or another cluster: ignore
        const x = opts.item(event.payload);
        const k = opts.key(x);
        let isNew = false;
        source.items.update((current) => {
          const idx = current.findIndex((c) => opts.key(c) === k);
          if (event.payload.event_type === 'deleted') return idx >= 0 ? current.filter((_, i) => i !== idx) : current;
          if (idx >= 0) {
            const next = [...current];
            next[idx] = x;
            return next;
          }
          isNew = true;
          return opts.prepend ? [x, ...current] : [...current, x];
        });
        if (event.payload.event_type === 'applied') opts.onApplied?.(x, isNew);
      });
      if (superseded()) {
        u();
        return drop(id);
      }
      unlisten = u;
    } catch (e) {
      if (superseded()) return;
      markFailed(source.name, e);
      console.error(`Failed to start ${opts.label} watch:`, e);
    }
  }

  async function stop() {
    gen++;
    await teardown();
  }

  return { start, stop };
}

// Polled lists (and the list half of the watched ones).
export const deploymentList = listStore<DeploymentInfo>('get_deployments', true, { name: 'deployments', items: deployments });
export const statefulSetList = listStore<StatefulSetInfo>('get_statefulsets', true, { name: 'statefulsets', items: statefulsets });
export const daemonSetList = listStore<DaemonSetInfo>('get_daemonsets', true, { name: 'daemonsets', items: daemonsets });
export const replicaSetList = listStore<ReplicaSetInfo>('get_replicasets', true, { name: 'replicasets', items: replicasets });
export const jobList = listStore<JobInfo>('get_jobs', true, { name: 'jobs', items: jobs });
export const cronJobList = listStore<CronJobInfo>('get_cronjobs', true, { name: 'cronjobs', items: cronjobs });
export const serviceList = listStore<ServiceInfo>('get_services', true, { name: 'services', items: services });
export const ingressList = listStore<IngressInfo>('get_ingresses', true, { name: 'ingresses', items: ingresses });
export const networkPolicyList = listStore<NetworkPolicyInfo>('get_network_policies', true, { name: 'networkpolicies', items: networkPolicies });
export const configMapList = listStore<ConfigMapInfo>('get_configmaps', true, { name: 'configmaps', items: configmaps });
export const secretList = listStore<SecretInfo>('get_secrets', true, { name: 'secrets', items: secrets });
export const hpaList = listStore<HPAInfo>('get_hpas', true, { name: 'hpas', items: hpas });
export const pvList = listStore<PersistentVolumeInfo>('get_pvs', false, { name: 'pvs', items: pvs });
export const pvcList = listStore<PersistentVolumeClaimInfo>('get_pvcs', true, { name: 'pvcs', items: pvcs });
export const namespaceList = listStore<NamespaceInfo>('get_namespaces_info', false, { name: 'namespaces', items: namespacesInfo });
export const nodeList = listStore<NodeInfo>('get_nodes', false, { name: 'nodes', items: nodes });
export const serviceAccountList = listStore<ServiceAccountInfo>('get_service_accounts', true, { name: 'serviceaccounts', items: serviceAccounts });
export const eventList = listStore<ClusterEventInfo>('get_events', true, { name: 'events', items: clusterEvents });

const deploymentWatch = listWatch<DeploymentInfo, DeploymentWatchEvent>({
  label: 'Deployment',
  list: 'get_deployments',
  watch: 'watch_deployments',
  channel: 'deployment-watch',
  source: deploymentList,
  key: (d) => `${d.namespace}/${d.name}`,
  item: (e) => e.deployment,
});

const jobWatch = listWatch<JobInfo, JobWatchEvent>({
  label: 'Job',
  list: 'get_jobs',
  watch: 'watch_jobs',
  channel: 'job-watch',
  source: jobList,
  key: (j) => `${j.namespace}/${j.name}`,
  item: (e) => e.job,
});

// Cluster-scoped, no namespace
const nodeWatch = listWatch<NodeInfo, NodeWatchEvent>({
  label: 'Node',
  list: 'get_nodes',
  watch: 'watch_nodes',
  channel: 'node-watch',
  source: nodeList,
  key: (n) => n.name,
  item: (e) => e.node,
});

// Started once in App.svelte (not by EventList) so Warning notifications
// fire from any view. A new Warning event notifies once; the initial list
// never notifies.
const eventWatch = listWatch<ClusterEventInfo, ClusterEventWatchEvent>({
  label: 'Event',
  list: 'get_events',
  watch: 'watch_events',
  channel: 'cluster-event-watch',
  source: eventList,
  key: (e) => `${e.namespace}/${e.name}`,
  item: (e) => e.event,
  prepend: true,
  onApplied: (e, isNew) => {
    if (isNew && e.type === 'Warning' && get(settings).notificationsEnabled) sendEventNotification(e);
  },
});

export const startDeploymentWatch = (namespace?: string | null) => deploymentWatch.start({ namespace });
export const stopDeploymentWatch = () => deploymentWatch.stop();
export const startJobWatch = (namespace?: string | null) => jobWatch.start({ namespace });
export const stopJobWatch = () => jobWatch.stop();
export const startNodeWatch = () => nodeWatch.start();
export const stopNodeWatch = () => nodeWatch.stop();
export const startEventWatch = (namespace?: string | null) => eventWatch.start({ namespace });
export const stopEventWatch = () => eventWatch.stop();

// Helper to send native notification for Warning events
async function sendEventNotification(event: ClusterEventInfo) {
  try {
    const { isPermissionGranted, requestPermission, sendNotification } = await import('@tauri-apps/plugin-notification');

    let permissionGranted = await isPermissionGranted();
    if (!permissionGranted) {
      const permission = await requestPermission();
      permissionGranted = permission === 'granted';
    }

    if (permissionGranted) {
      sendNotification({
        title: `${event.reason}: ${event.involved_kind}/${event.involved_object}`,
        body: event.message,
      });
    }
  } catch (e) {
    console.error('Failed to send notification:', e);
  }
}

export const loadEvents = (namespace?: string | null) => eventList.load(namespace);
export const loadDeployments = (namespace?: string | null) => deploymentList.load(namespace);
export const loadStatefulSets = (namespace?: string | null) => statefulSetList.load(namespace);

export async function scaleStatefulSet(namespace: string, name: string, replicas: number) {
  if (!isContextReady()) return;
  await tauriInvoke('scale_statefulset', { namespace, name, replicas });
}

export async function restartStatefulSet(namespace: string, name: string) {
  if (!isContextReady()) return;
  await tauriInvoke('restart_statefulset', { namespace, name });
}

export const loadDaemonSets = (namespace?: string | null) => daemonSetList.load(namespace);
export const loadReplicaSets = (namespace?: string | null) => replicaSetList.load(namespace);
export const loadJobs = (namespace?: string | null) => jobList.load(namespace);
export const loadCronJobs = (namespace?: string | null) => cronJobList.load(namespace);
export const loadServices = (namespace?: string | null) => serviceList.load(namespace);
export const loadIngresses = (namespace?: string | null) => ingressList.load(namespace);
export const loadNetworkPolicies = (namespace?: string | null) => networkPolicyList.load(namespace);
export const loadConfigMaps = (namespace?: string | null) => configMapList.load(namespace);
export const loadSecrets = (namespace?: string | null) => secretList.load(namespace);
export const loadHPAs = (namespace?: string | null) => hpaList.load(namespace);
export const loadPVs = () => pvList.load();
export const loadPVCs = (namespace?: string | null) => pvcList.load(namespace);
export const loadNamespacesInfo = () => namespaceList.load();
export const loadNodes = () => nodeList.load();

// Polled separately from the node watch: usage changes constantly, and a
// missing metrics-server or `nodes/proxy` permission must not surface as an
// app-wide error — rows just show no bars.
let nodeMetricsInFlight = false;

export async function loadNodeMetrics() {
  if (!isContextReady()) return;
  // A slow poll (unreachable kubelets) can outlast the interval. Without this,
  // polls stack up and a late response overwrites a newer one.
  if (nodeMetricsInFlight) return;

  // Node names collide across clusters, so a poll that resolves after a context
  // switch would paint the old cluster's usage onto the new cluster's rows.
  const context = _currentContext;
  const nodeNames = get(nodes).map((n) => n.name);
  if (nodeNames.length === 0) return;

  nodeMetricsInFlight = true;
  try {
    const metrics = await tauriInvoke<NodeMetricsInfo[]>('get_node_metrics', {
      contextName: context,
      nodeNames,
    });
    if (_currentContext !== context) return;
    nodeMetrics.set(Object.fromEntries(metrics.map((m) => [m.name, m])));
  } catch (e) {
    console.warn('Failed to load node metrics:', e);
  } finally {
    nodeMetricsInFlight = false;
  }
}

export const loadServiceAccounts = (namespace?: string | null) => serviceAccountList.load(namespace);

export async function loadClusterMetrics() {
  if (!isContextReady()) return;
  try {
    const metrics = await tauriInvoke<ClusterMetrics>('get_cluster_metrics');
    clusterMetrics.set(metrics);
  } catch (e) {
    error.set(String(e));
  }
}

/** Context and namespace the current `pulseMetrics` were loaded for. */
export const pulseScope = writable<{ context: string; namespace: string | null } | null>(null);

/**
 * API server reachability over the session (Pulse "Uptime" tile): every
 * metrics poll counts as one probe. Reset on context switch.
 */
export const apiProbes = writable<{ ok: number; total: number; since: number }>({ ok: 0, total: 0, since: Date.now() });

export async function loadPulseMetrics(namespace?: string | null) {
  if (!isContextReady()) return;
  const context = _currentContext;
  try {
    const metrics = await tauriInvoke<PulseMetrics>('get_pulse_metrics', { namespace });
    if (_currentContext !== context) return;
    pulseMetrics.set(metrics);
    pulseScope.set({ context, namespace: namespace ?? null });
    markSynced('pulse');
    // The status bar's "Connected" reflects the last real API call.
    connectionStatus.set('connected');
    apiProbes.update((p) => ({ ...p, ok: p.ok + 1, total: p.total + 1 }));
  } catch (e) {
    error.set(String(e));
    // 401/403 means the server answered: not downtime, and still connected.
    if (_currentContext !== context) return;
    const answered = isForbiddenError(e) || /\b401\b|unauthorized/i.test(String(e));
    if (!answered) connectionStatus.set('disconnected');
    apiProbes.update((p) => ({ ...p, ok: p.ok + (answered ? 1 : 0), total: p.total + 1 }));
  }
}

// Load all resources for global search
export async function loadAllResources(namespace?: string | null) {
  if (!isContextReady()) return;

  // Load all resources in parallel
  await Promise.all([
    loadPods(namespace),
    loadDeployments(namespace),
    loadStatefulSets(namespace),
    loadDaemonSets(namespace),
    loadReplicaSets(namespace),
    loadJobs(namespace),
    loadCronJobs(namespace),
    loadServices(namespace),
    loadIngresses(namespace),
    loadNetworkPolicies(namespace),
    loadConfigMaps(namespace),
    loadSecrets(namespace),
    loadHPAs(namespace),
    loadPVs(),
    loadPVCs(namespace),
    loadNamespacesInfo(),
    loadNodes(),
    loadServiceAccounts(namespace),
  ]);
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

export interface HealthBuckets<T> {
  all: T[];
  failed: T[];
  warning: T[];
  progressing: T[];
  healthy: T[];
  /** Neutral verdicts: Completed pods, Scaled to 0, Suspended. */
  completed: T[];
  /** No usable data; only reachable through `all`. */
  unknown: T[];
}

/**
 * Bucket rows by their Rust verdict (spec 5.1). A CrashLoopBackOff pod has
 * phase Running but lands in `failed` here, which is the whole point.
 */
export function bucketByHealth<T extends { verdict: Verdict }>(items: T[]): HealthBuckets<T> {
  const b: HealthBuckets<T> = {
    all: items,
    failed: [],
    warning: [],
    progressing: [],
    healthy: [],
    completed: [],
    unknown: [],
  };
  for (const item of items) {
    const h = verdictToHealth(item.verdict);
    if (h === 'neutral') b.completed.push(item);
    else b[h].push(item);
  }
  return b;
}

export const podsByStatus = derived(pods, ($pods) => bucketByHealth($pods));
