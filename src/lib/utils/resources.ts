// src/lib/utils/resources.ts
// Pure helpers for the extra resource views (spec 12): Kubernetes quantity
// parsing, ResourceQuota usage, PodDisruptionBudget health, and the type id
// that carries a custom resource's group/version/plural to its detail window.
import type { HealthResult } from './health';
import { HEALTH_RANK } from './health';

const BINARY: Record<string, number> = { Ki: 2 ** 10, Mi: 2 ** 20, Gi: 2 ** 30, Ti: 2 ** 40, Pi: 2 ** 50, Ei: 2 ** 60 };
const DECIMAL: Record<string, number> = { n: 1e-9, u: 1e-6, m: 1e-3, '': 1, k: 1e3, M: 1e6, G: 1e9, T: 1e12, P: 1e15, E: 1e18 };

/**
 * Kubernetes quantity in base units ("500m" -> 0.5, "2Gi" -> 2147483648,
 * "1.5k" -> 1500, "1e3" -> 1000). Null when it doesn't parse: never a
 * confident 0 (same rule as node metrics in CLAUDE.md).
 */
export function parseQuantity(q: string | null | undefined): number | null {
  if (q == null) return null;
  const m = /^([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)(Ki|Mi|Gi|Ti|Pi|Ei|[numkMGTPE])?$/.exec(q.trim());
  if (!m) return null;
  const n = Number(m[1]);
  if (!Number.isFinite(n)) return null;
  const suffix = m[2] ?? '';
  const mult = BINARY[suffix] ?? DECIMAL[suffix];
  return mult === undefined ? null : n * mult;
}

export interface QuotaRow {
  resource: string;
  used: string;
  hard: string;
  /** used / hard * 100, or null when either side doesn't parse or hard is 0. */
  pct: number | null;
}

/** One row per hard limit, fullest first (unparseable ones last). */
export function quotaUsage(hard: Record<string, string>, used: Record<string, string>): QuotaRow[] {
  const rows = Object.entries(hard).map(([resource, h]) => {
    const u = used[resource] ?? '0';
    const hn = parseQuantity(h);
    const un = parseQuantity(u);
    const pct = hn && un !== null ? (un / hn) * 100 : hn === 0 && un === 0 ? 0 : null;
    return { resource, used: u, hard: h, pct };
  });
  return rows.sort((a, b) => (b.pct ?? -1) - (a.pct ?? -1) || a.resource.localeCompare(b.resource));
}

/** Quota health from its fullest resource: 100% is failed (new pods are refused), 90% a warning. */
export function quotaHealth(rows: readonly QuotaRow[]): HealthResult {
  const worst = rows.find((r) => r.pct !== null);
  if (!worst || worst.pct === null) return { health: 'neutral', label: rows.length ? 'No usage' : 'No limits', rank: HEALTH_RANK.neutral };
  const label = `${worst.resource} ${Math.round(worst.pct)}%`;
  if (worst.pct >= 100) return { health: 'failed', label, rank: HEALTH_RANK.failed };
  if (worst.pct >= 90) return { health: 'warning', label, rank: HEALTH_RANK.warning };
  return { health: 'healthy', label, rank: HEALTH_RANK.healthy };
}

export interface PdbLike {
  allowed_disruptions: number;
  current_healthy: number;
  desired_healthy: number;
  expected_pods: number;
}

/**
 * Healthy when at least one disruption is allowed. Zero allowed blocks node
 * drains (warning); fewer healthy pods than the budget wants is failed.
 * A budget that selects no pods is neutral, not a problem.
 */
export function pdbHealth(p: PdbLike): HealthResult {
  if (p.expected_pods === 0) return { health: 'neutral', label: 'No matching pods', rank: HEALTH_RANK.neutral };
  if (p.current_healthy < p.desired_healthy) {
    return { health: 'failed', label: `${p.current_healthy}/${p.desired_healthy} healthy`, rank: HEALTH_RANK.failed };
  }
  if (p.allowed_disruptions <= 0) return { health: 'warning', label: 'Blocks disruptions', rank: HEALTH_RANK.warning };
  return { health: 'healthy', label: `${p.allowed_disruptions} allowed`, rank: HEALTH_RANK.healthy };
}

// ---------- Custom resource type ids ----------

export interface CustomTarget {
  group: string;
  version: string;
  kind: string;
  plural: string;
  namespaced: boolean;
}

const CR_PREFIX = 'cr|';

/**
 * Detail windows get one `type` string, so a custom resource packs its API
 * coordinates into it: `cr|<group>|<version>|<plural>|<Kind>|<n|c>`.
 */
export function encodeCrType(t: CustomTarget): string {
  return [CR_PREFIX + t.group, t.version, t.plural, t.kind, t.namespaced ? 'n' : 'c'].join('|');
}

export function parseCrType(type: string): CustomTarget | null {
  if (!type.startsWith(CR_PREFIX)) return null;
  const parts = type.slice(CR_PREFIX.length).split('|');
  if (parts.length !== 5) return null;
  const [group, version, plural, kind, scope] = parts;
  if (!version || !plural || !kind || (scope !== 'n' && scope !== 'c')) return null;
  return { group, version, plural, kind, namespaced: scope === 'n' };
}

export interface CrdLike {
  group: string;
  kind: string;
  plural: string;
  scope: string;
  version: string;
}

export const crdTarget = (c: CrdLike): CustomTarget => ({
  group: c.group,
  version: c.version,
  kind: c.kind,
  plural: c.plural,
  namespaced: c.scope === 'Namespaced',
});
