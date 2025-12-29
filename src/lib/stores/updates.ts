import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface UpdateInfo {
  current_version: string;
  latest_version: string;
  update_available: boolean;
  release_url: string;
  release_notes: string | null;
  published_at: string | null;
}

export const updateInfo = writable<UpdateInfo | null>(null);
export const updateCheckError = writable<string | null>(null);
export const isCheckingUpdate = writable<boolean>(false);

export async function checkForUpdates(): Promise<void> {
  isCheckingUpdate.set(true);
  updateCheckError.set(null);

  try {
    const info = await invoke<UpdateInfo>('check_for_updates');
    updateInfo.set(info);
  } catch (e) {
    console.error('Failed to check for updates:', e);
    updateCheckError.set(String(e));
    updateInfo.set(null);
  } finally {
    isCheckingUpdate.set(false);
  }
}

// Check for updates on app start (with a small delay to not block startup)
export function initUpdateCheck(): void {
  setTimeout(() => {
    checkForUpdates();
  }, 3000); // Check 3 seconds after startup
}
