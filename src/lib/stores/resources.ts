// src/lib/stores/resources.ts
// Stores for the extra resource types (spec 12). Each is a polled list with
// the same guards as the watches in kubernetes.ts: a generation counter so
// an older request can't overwrite a newer one, and a context check so a
// late answer from the previous cluster is dropped. Lists clear on a
// context switch so names from one cluster never show under another.
import { writable, get, type Readable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { currentContext, isForbiddenError } from './kubernetes';
import type { CustomTarget } from '../utils/resources';

export type ListStatus = 'idle' | 'loading' | 'ready' | 'error' | 'forbidden';
export interface ListState {
  status: ListStatus;
  error?: string;
}

export interface ListStore<T> {
  items: Readable<T[]>;
  state: Readable<ListState>;
  /**
   * Namespaced lists take the namespace filter; `extra` adds command args.
   * `poll: true` is a background tick: skipped while a load is in flight,
   * so a slow cluster isn't hit again before it answers and the slow
   * answer isn't thrown away. Other loads supersede an in-flight one.
   */
  load: (namespace?: string | null, extra?: Record<string, unknown>, opts?: LoadOptions) => Promise<void>;
  reset: () => void;
  namespaced: boolean;
}

export interface LoadOptions {
  poll?: boolean;
  /** Override the store's scope, e.g. a cluster-scoped CRD in the custom resource browser. */
  namespaced?: boolean;
}

export function listStore<T>(command: string, namespaced: boolean): ListStore<T> {
  const items = writable<T[]>([]);
  const state = writable<ListState>({ status: 'idle' });
  let gen = 0;
  let inFlight = false;

  async function load(namespace?: string | null, extra: Record<string, unknown> = {}, opts: LoadOptions = {}) {
    const ctx = get(currentContext);
    if (!ctx) return;
    if (opts.poll && inFlight) return;
    const scoped = opts.namespaced ?? namespaced;
    const g = ++gen;
    inFlight = true;
    // Keep showing rows during a refresh; only the first load shows the skeleton.
    state.update((s) => (s.status === 'ready' ? s : { status: 'loading' }));
    try {
      const args = scoped ? { namespace: namespace ?? null, ...extra } : extra;
      const rows = await invoke<T[]>(command, args);
      if (g !== gen || get(currentContext) !== ctx) return;
      items.set(rows);
      state.set({ status: 'ready' });
    } catch (e) {
      if (g !== gen || get(currentContext) !== ctx) return;
      state.set({ status: isForbiddenError(e) ? 'forbidden' : 'error', error: String(e) });
    } finally {
      if (g === gen) inFlight = false;
    }
  }

  function reset() {
    gen++;
    inFlight = false;
    items.set([]);
    state.set({ status: 'idle' });
  }

  let ctx = get(currentContext);
  currentContext.subscribe((c) => {
    if (c !== ctx) {
      ctx = c;
      reset();
    }
  });

  return { items, state, load, reset, namespaced };
}

// ---------- Row types (match src-tauri/src/resources.rs) ----------

export interface RoleInfo {
  name: string;
  namespace: string;
  rules: number;
  age: string;
}
export interface ClusterRoleInfo {
  name: string;
  rules: number;
  aggregated: boolean;
  age: string;
}
export interface RoleBindingInfo {
  name: string;
  namespace: string;
  role_ref: string;
  subjects: string;
  subject_count: number;
  age: string;
}
export interface ClusterRoleBindingInfo {
  name: string;
  role_ref: string;
  subjects: string;
  subject_count: number;
  age: string;
}
export interface StorageClassInfo {
  name: string;
  provisioner: string;
  reclaim_policy: string;
  binding_mode: string;
  allow_expansion: boolean;
  is_default: boolean;
  age: string;
}
export interface EndpointSliceInfo {
  name: string;
  namespace: string;
  service: string;
  address_type: string;
  ready: number;
  total: number;
  ports: string;
  age: string;
}
export interface IngressClassInfo {
  name: string;
  controller: string;
  is_default: boolean;
  parameters: string;
  age: string;
}
export interface PdbInfo {
  name: string;
  namespace: string;
  min_available: string;
  max_unavailable: string;
  allowed_disruptions: number;
  current_healthy: number;
  desired_healthy: number;
  expected_pods: number;
  age: string;
}
export interface ResourceQuotaInfo {
  name: string;
  namespace: string;
  hard: Record<string, string>;
  used: Record<string, string>;
  age: string;
}
export interface LimitRangeInfo {
  name: string;
  namespace: string;
  types: string;
  limits: number;
  age: string;
}
export interface PriorityClassInfo {
  name: string;
  value: number;
  global_default: boolean;
  preemption_policy: string;
  description: string;
  age: string;
}
export interface CrdInfo {
  name: string;
  group: string;
  kind: string;
  plural: string;
  scope: string;
  versions: string[];
  version: string;
  age: string;
}
export interface CustomResourceInfo {
  name: string;
  namespace: string;
  age: string;
}

// ---------- Stores ----------

export const roles = listStore<RoleInfo>('get_roles', true);
export const clusterRoles = listStore<ClusterRoleInfo>('get_cluster_roles', false);
export const roleBindings = listStore<RoleBindingInfo>('get_role_bindings', true);
export const clusterRoleBindings = listStore<ClusterRoleBindingInfo>('get_cluster_role_bindings', false);
export const storageClasses = listStore<StorageClassInfo>('get_storage_classes', false);
export const endpointSlices = listStore<EndpointSliceInfo>('get_endpoint_slices', true);
export const ingressClasses = listStore<IngressClassInfo>('get_ingress_classes', false);
export const pdbs = listStore<PdbInfo>('get_pdbs', true);
export const resourceQuotas = listStore<ResourceQuotaInfo>('get_resource_quotas', true);
export const limitRanges = listStore<LimitRangeInfo>('get_limit_ranges', true);
export const priorityClasses = listStore<PriorityClassInfo>('get_priority_classes', false);
export const crds = listStore<CrdInfo>('get_crds', false);
/** Instances of the CRD picked in the browser; `load(ns, { target })`. */
export const customResources = listStore<CustomResourceInfo>('get_custom_resources', true);

export type { CustomTarget };

/** CRD picked in the custom resource browser (session only). */
export const selectedCrdName = writable<string | null>(null);
