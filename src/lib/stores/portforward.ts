import { writable, derived } from 'svelte/store';

// Lazy invoke helper to ensure Tauri is ready
async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    throw new Error('Not running in Tauri environment');
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

export type PortForwardStatus = 'starting' | 'active' | 'error' | 'stopped';
export type ResourceType = 'pod' | 'service';

export interface PortForwardInfo {
  id: string;
  context: string;
  namespace: string;
  resource_type: ResourceType;
  resource_name: string;
  pod_name: string;
  local_port: number;
  remote_port: number;
  status: PortForwardStatus;
  // Connection stats
  active_connections: number;
  total_connections: number;
  bytes_sent: number;
  bytes_received: number;
}

export interface AvailablePort {
  port: number;
  name: string | null;
  protocol: string;
}

// Stores
export const portForwards = writable<PortForwardInfo[]>([]);
export const isLoadingPortForwards = writable<boolean>(false);
export const portForwardError = writable<string | null>(null);
export const portForwardPanelOpen = writable<boolean>(false);

// Derived store for active count (for badge)
export const activePortForwardCount = derived(
  portForwards,
  $pf => $pf.filter(p => p.status === 'active' || p.status === 'starting').length
);

// Load all port forwards
export async function loadPortForwards(): Promise<void> {
  try {
    isLoadingPortForwards.set(true);
    portForwardError.set(null);
    const forwards = await tauriInvoke<PortForwardInfo[]>('list_port_forwards');
    portForwards.set(forwards);
  } catch (e) {
    console.error('Failed to load port forwards:', e);
    portForwardError.set(String(e));
  } finally {
    isLoadingPortForwards.set(false);
  }
}

// Start a new port forward
export async function startPortForward(
  context: string,
  namespace: string,
  resourceType: ResourceType,
  resourceName: string,
  localPort: number,
  remotePort: number
): Promise<PortForwardInfo> {
  try {
    portForwardError.set(null);
    const info = await tauriInvoke<PortForwardInfo>('start_port_forward', {
      context,
      namespace,
      resourceType,
      resourceName,
      localPort,
      remotePort
    });
    // Refresh the list
    await loadPortForwards();
    return info;
  } catch (e) {
    console.error('Failed to start port forward:', e);
    portForwardError.set(String(e));
    throw e;
  }
}

// Stop a port forward
export async function stopPortForward(id: string): Promise<void> {
  try {
    portForwardError.set(null);
    await tauriInvoke<void>('stop_port_forward', { id });
    // Refresh the list
    await loadPortForwards();
  } catch (e) {
    console.error('Failed to stop port forward:', e);
    portForwardError.set(String(e));
    throw e;
  }
}

// Stop all port forwards
export async function stopAllPortForwards(): Promise<void> {
  try {
    portForwardError.set(null);
    await tauriInvoke<void>('stop_all_port_forwards');
    // Refresh the list
    await loadPortForwards();
  } catch (e) {
    console.error('Failed to stop all port forwards:', e);
    portForwardError.set(String(e));
    throw e;
  }
}

// Get available ports for a resource
export async function getResourcePorts(
  context: string,
  namespace: string,
  resourceType: ResourceType,
  resourceName: string
): Promise<AvailablePort[]> {
  try {
    return await tauriInvoke<AvailablePort[]>('get_resource_ports', {
      context,
      namespace,
      resourceType,
      resourceName
    });
  } catch (e) {
    console.error('Failed to get resource ports:', e);
    return [];
  }
}

// Poll for updates (status changes)
let pollInterval: ReturnType<typeof setInterval> | null = null;

export function startPolling(intervalMs: number = 2000): void {
  if (pollInterval) return;
  pollInterval = setInterval(() => {
    loadPortForwards();
  }, intervalMs);
}

export function stopPolling(): void {
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
}
