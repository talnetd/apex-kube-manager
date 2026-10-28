// src/lib/utils/health.ts
// Verdict -> display mapping (spec 5.1, 11.1). No detection rules live here:
// verdicts are computed in Rust by the shared triage rules (spec/triage/).

export type Health = 'failed' | 'warning' | 'progressing' | 'healthy' | 'neutral' | 'unknown';

export interface Verdict {
  status: 'ok' | 'warn' | 'fail' | 'unknown';
  reason: string;
  ruleId: string;
  since?: string;
  severity?: 'critical' | 'error' | 'warning';
  tone?: 'progressing' | 'neutral';
}

export interface HealthResult {
  health: Health;
  label: string;
  /** Sort key per 11.1: lower is worse. */
  rank: number;
  severity?: Verdict['severity'];
}

/** Rank per spec 11.1. Drives every "problems first" sort and "worst of" aggregation. */
export const HEALTH_RANK: Record<Health, number> = {
  failed: 0,
  warning: 1,
  progressing: 2,
  unknown: 3,
  healthy: 4,
  neutral: 5,
};

export const HEALTH_NAME: Record<Health, string> = {
  failed: 'failed',
  warning: 'warning',
  progressing: 'progressing',
  unknown: 'unknown',
  healthy: 'healthy',
  neutral: 'neutral',
};

export function verdictToHealth(v: Pick<Verdict, 'status' | 'tone'>): Health {
  switch (v.status) {
    case 'fail':
      return 'failed';
    case 'warn':
      return 'warning';
    case 'unknown':
      return 'unknown';
    case 'ok':
      if (v.tone === 'progressing') return 'progressing';
      if (v.tone === 'neutral') return 'neutral';
      return 'healthy';
  }
}

export function toDisplay(v: Verdict): HealthResult {
  const health = verdictToHealth(v);
  return {
    health,
    label: v.reason || (health === 'unknown' ? 'Unknown' : ''),
    rank: HEALTH_RANK[health],
    severity: v.severity,
  };
}

/** Worst of several states (11.3 rule 6). Returns null for an empty list. */
export function worstHealth(states: Health[]): Health | null {
  let worst: Health | null = null;
  for (const h of states) {
    if (worst === null || HEALTH_RANK[h] < HEALTH_RANK[worst]) worst = h;
  }
  return worst;
}

/** Accessible name pattern (11.3 rule 7): "{label}, {state}". */
export function healthAriaLabel(health: Health, label: string): string {
  return label ? `${label}, ${HEALTH_NAME[health]}` : HEALTH_NAME[health];
}

/**
 * Class names per state (11.2). Written out literally so Tailwind's scanner
 * picks them up. Unknown and neutral use the neutral text tokens, no tint.
 */
export const HEALTH_CLASSES: Record<Health, { fg: string; solid: string; tint: string; icon: string }> = {
  failed: { fg: 'text-danger', solid: 'bg-danger-solid', tint: 'bg-danger-tint', icon: 'text-danger-solid' },
  warning: { fg: 'text-warning', solid: 'bg-warning-solid', tint: 'bg-warning-tint', icon: 'text-warning-solid' },
  progressing: { fg: 'text-info', solid: 'bg-info-solid', tint: 'bg-info-tint', icon: 'text-info-solid' },
  healthy: { fg: 'text-success', solid: 'bg-success-solid', tint: 'bg-success-tint', icon: 'text-success-solid' },
  unknown: { fg: 'text-secondary', solid: 'bg-muted', tint: '', icon: 'text-muted' },
  neutral: { fg: 'text-secondary', solid: 'bg-muted', tint: '', icon: 'text-muted' },
};
