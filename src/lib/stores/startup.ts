import { writable, derived } from 'svelte/store';

export interface StartupCheck {
  id: string;
  label: string;
  status: 'pending' | 'running' | 'success' | 'error';
  message?: string;
}

export const startupChecks = writable<StartupCheck[]>([
  { id: 'kubeconfig', label: 'Locating kubeconfig', status: 'pending' },
  { id: 'parse', label: 'Parsing configuration', status: 'pending' },
  { id: 'contexts', label: 'Loading contexts', status: 'pending' },
  { id: 'connection', label: 'Connecting to cluster', status: 'pending' },
  { id: 'namespaces', label: 'Fetching namespaces', status: 'pending' },
]);

export const isInitialized = writable(false);
export const initError = writable<string | null>(null);

function updateCheck(id: string, updates: Partial<StartupCheck>) {
  startupChecks.update((checks) =>
    checks.map((check) => (check.id === id ? { ...check, ...updates } : check))
  );
}

async function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function runStartupChecks(): Promise<boolean> {
  try {
    // Check if running in Tauri environment
    if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
      throw new Error('Not running in Tauri environment. Use "npm run tauri dev" to start the app.');
    }

    // Dynamically import to ensure Tauri is ready
    const { invoke } = await import('@tauri-apps/api/core');

    // Check 1: Kubeconfig exists
    updateCheck('kubeconfig', { status: 'running' });
    await delay(300);
    const configPath = await invoke<string>('check_kubeconfig');
    updateCheck('kubeconfig', { status: 'success', message: configPath });

    // Check 2: Parse configuration
    updateCheck('parse', { status: 'running' });
    await delay(200);
    await invoke('validate_kubeconfig');
    updateCheck('parse', { status: 'success' });

    // Check 3: Load contexts
    updateCheck('contexts', { status: 'running' });
    await delay(200);
    const contexts = await invoke<string[]>('get_context_names');
    updateCheck('contexts', { status: 'success', message: `${contexts.length} contexts found` });

    // Check 4: Connect to cluster
    updateCheck('connection', { status: 'running' });
    const currentContext = await invoke<string>('get_current_context');
    await invoke('test_cluster_connection');
    updateCheck('connection', { status: 'success', message: currentContext });

    // Check 5: Fetch namespaces
    updateCheck('namespaces', { status: 'running' });
    const namespaces = await invoke<string[]>('get_namespaces');
    updateCheck('namespaces', { status: 'success', message: `${namespaces.length} namespaces` });

    await delay(500);
    isInitialized.set(true);
    return true;
  } catch (error) {
    const errorMessage = String(error);
    initError.set(errorMessage);

    // Mark current running check as error
    startupChecks.update((checks) =>
      checks.map((check) =>
        check.status === 'running' ? { ...check, status: 'error', message: errorMessage } : check
      )
    );

    return false;
  }
}

export const startupProgress = derived(startupChecks, ($checks) => {
  const completed = $checks.filter((c) => c.status === 'success').length;
  return Math.round((completed / $checks.length) * 100);
});

export const currentCheck = derived(startupChecks, ($checks) => {
  return $checks.find((c) => c.status === 'running')?.label || '';
});
