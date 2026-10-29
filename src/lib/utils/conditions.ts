// src/lib/utils/conditions.ts
// Health of a Kubernetes condition row. True is good for most types; the
// node "pressure" types are good when False.
const INVERTED = new Set(['MemoryPressure', 'DiskPressure', 'PIDPressure', 'NetworkUnavailable']);

export function conditionHealth(type: string, status: string): 'healthy' | 'failed' | 'unknown' {
  if (status !== 'True' && status !== 'False') return 'unknown';
  const good = INVERTED.has(type) ? status === 'False' : status === 'True';
  return good ? 'healthy' : 'failed';
}
