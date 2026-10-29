// src/lib/stores/resources.ts
// Stores for the extra resource types (spec 12). They use the same list
// model as every other list (`listStore` in kubernetes.ts): poll ticks skip
// while a load is in flight, a newer load supersedes an older one, late
// answers from the previous cluster are dropped, and rows clear on a
// context switch.
import { writable } from 'svelte/store';
import { listStore, type ListStore, type ListLoadOptions, type ResourceState } from './kubernetes';
import type { CustomTarget } from '../utils/resources';

export { listStore };
export type { ListStore, ListLoadOptions as LoadOptions };
export type ListState = ResourceState;
export type ListStatus = ResourceState['status'];

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
