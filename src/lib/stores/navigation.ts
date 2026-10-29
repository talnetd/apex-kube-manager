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
  | 'serviceaccounts'
  | 'nodes'
  | 'namespaces'
  | 'events';

export const currentView = writable<View>('dashboard');
export const sidebarCollapsed = writable<boolean>(false);

/** sm (< 640px) only: the sidebar is an off-canvas drawer (spec 13.2). */
export const navOpen = writable<boolean>(false);
currentView.subscribe(() => navOpen.set(false));
