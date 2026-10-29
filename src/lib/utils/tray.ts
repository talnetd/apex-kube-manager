// src/lib/utils/tray.ts
// Menu bar popover display logic (spec 13.5). The dot itself is reduced in
// Rust (src-tauri/src/tray.rs); this turns a snapshot into text.

export type Dot = 'grey' | 'green' | 'amber' | 'red';

export interface ClusterStatus {
  context: string;
  cluster: string;
  current: boolean;
  dot: Dot;
  reachable: boolean | null;
  critical: number;
  warning: number;
  detail: boolean;
  error: string | null;
}

export interface TraySnapshot {
  enabled: boolean;
  clusters: ClusterStatus[];
  checked_at: string | null;
}

export const DOT_LABEL: Record<Dot, string> = {
  grey: 'checking',
  green: 'connected',
  amber: 'has issues',
  red: 'needs attention',
};

/** The one-line summary under the popover title. */
export function traySummary(clusters: readonly ClusterStatus[]): string {
  if (!clusters.length) return 'No clusters found';
  if (clusters.every((c) => c.dot === 'grey')) return 'Checking clusters…';
  const bad = clusters.filter((c) => c.dot === 'red' || c.dot === 'amber').length;
  if (bad === 0) return clusters.length === 1 ? 'Cluster healthy' : `All ${clusters.length} clusters healthy`;
  return bad === 1 ? '1 cluster needs attention' : `${bad} clusters need attention`;
}

/** Right-hand text for a row: counts for the current context, state for others. */
export function clusterNote(c: ClusterStatus): string {
  if (c.reachable === null) return 'checking…';
  if (!c.reachable) return 'unreachable';
  if (!c.detail) return 'reachable';
  const parts: string[] = [];
  if (c.critical) parts.push(`${c.critical} critical`);
  if (c.warning) parts.push(`${c.warning} ${c.warning === 1 ? 'issue' : 'issues'}`);
  return parts.join(' · ') || 'no issues';
}

/** "4s", "2m": time since the last sweep. */
export function sinceLabel(checkedAt: string | null, now: number): string | null {
  if (!checkedAt) return null;
  const t = Date.parse(checkedAt);
  if (!Number.isFinite(t)) return null;
  const s = Math.max(0, Math.round((now - t) / 1000));
  return s < 60 ? `${s}s` : `${Math.round(s / 60)}m`;
}
