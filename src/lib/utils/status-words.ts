// src/lib/utils/status-words.ts
// Display health for the listed resources whose Rust structs don't carry a
// triage `verdict` yet (spec 11.5): Event, PV, Namespace, CronJob. Each maps
// a status word the API already reports straight to a display state; nothing
// here detects a problem. When Rust adds verdicts for one of these types,
// delete its function and use `toDisplay(row.verdict)` like the workloads.
import { HEALTH_RANK, toDisplay, type Health, type HealthResult, type Verdict } from './health';

/**
 * `toDisplay(row.verdict)`, cached per row object. Watch and poll updates
 * replace a row's object when it changes, so a stale entry can't survive.
 */
export function verdictHealth<T extends { verdict: Verdict }>(): (r: T) => HealthResult {
  const cache = new WeakMap<T, HealthResult>();
  return (r) => {
    let h = cache.get(r);
    if (!h) {
      h = toDisplay(r.verdict);
      cache.set(r, h);
    }
    return h;
  };
}

function result(health: Health, label: string): HealthResult {
  return { health, label, rank: HEALTH_RANK[health] };
}

/** Warning events carry a state; Normal events are neutral, not healthy (11.3 rule 4). */
export function eventHealth(type: string): HealthResult {
  return type === 'Warning' ? result('warning', 'Warning') : result('neutral', type || 'Normal');
}

/** PV phase: Failed / Released / Pending / Bound / Available. */
export function pvHealth(status: string): HealthResult {
  switch (status) {
    case 'Failed':
      return result('failed', status);
    case 'Released':
      return result('warning', status);
    case 'Pending':
      return result('progressing', status);
    case 'Bound':
      return result('healthy', status);
    case 'Available':
      return result('neutral', status);
    default:
      return result('unknown', status || 'Unknown');
  }
}

/** Namespace phase: Active / Terminating. */
export function namespaceHealth(status: string): HealthResult {
  if (status === 'Active') return result('healthy', status);
  if (status === 'Terminating') return result('progressing', status);
  return result('unknown', status || 'Unknown');
}

/** CronJob: suspended is neutral, running jobs are progressing, otherwise scheduled. */
export function cronJobHealth(cj: { suspend: boolean; active: number }): HealthResult {
  if (cj.suspend) return result('neutral', 'Suspended');
  if (cj.active > 0) return result('progressing', `${cj.active} active`);
  return result('healthy', 'Scheduled');
}
