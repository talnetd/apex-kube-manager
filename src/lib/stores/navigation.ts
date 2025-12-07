import { writable } from 'svelte/store';

export type View =
  | 'dashboard'
  | 'pods'
  | 'deployments'
  | 'statefulsets'
  | 'daemonsets'
  | 'replicasets'
  | 'jobs'
  | 'cronjobs'
  | 'services'
  | 'configmaps'
  | 'secrets'
  | 'pvs'
  | 'pvcs'
  | 'hpas'
  | 'ingresses'
  | 'network_policies'
  | 'service_accounts'
  | 'nodes'
  | 'namespaces';

export const currentView = writable<View>('dashboard');
export const sidebarCollapsed = writable<boolean>(false);
