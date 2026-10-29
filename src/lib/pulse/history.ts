// src/lib/pulse/history.ts
// In-app history for Pulse sparklines (spec 6.12): a fixed-size ring buffer
// of timestamped samples, kept in memory only. Sampling rides on the existing
// metrics poll, so it adds no requests.

export interface Sample {
  t: number;
  v: number | null;
}

export class RingBuffer<T> {
  private buf: (T | undefined)[];
  private start = 0;
  private size = 0;

  constructor(readonly capacity: number) {
    if (!Number.isInteger(capacity) || capacity < 1) throw new Error('capacity must be a positive integer');
    this.buf = new Array(capacity);
  }

  get length(): number {
    return this.size;
  }

  push(item: T): void {
    const end = (this.start + this.size) % this.capacity;
    this.buf[end] = item;
    if (this.size < this.capacity) this.size++;
    else this.start = (this.start + 1) % this.capacity;
  }

  /** Oldest first. */
  toArray(): T[] {
    const out: T[] = [];
    for (let i = 0; i < this.size; i++) out.push(this.buf[(this.start + i) % this.capacity] as T);
    return out;
  }

  last(): T | undefined {
    return this.size ? this.buf[(this.start + this.size - 1) % this.capacity] : undefined;
  }

  clear(): void {
    this.buf = new Array(this.capacity);
    this.start = 0;
    this.size = 0;
  }
}

export const HOUR_MS = 60 * 60 * 1000;

/** Samples inside the window ending at `now`, oldest first. */
export function windowed(samples: readonly Sample[], now: number, windowMs = HOUR_MS): Sample[] {
  const from = now - windowMs;
  return samples.filter((s) => s.t >= from && s.t <= now);
}

/**
 * Change between the first and last non-null sample in the window, or null
 * when there aren't two points to compare.
 */
export function delta(samples: readonly Sample[]): number | null {
  const vals = samples.map((s) => s.v).filter((v): v is number => v !== null);
  if (vals.length < 2) return null;
  return vals[vals.length - 1] - vals[0];
}

/** One metric's history: push on each poll, read the last hour. */
export class MetricHistory {
  private ring: RingBuffer<Sample>;

  /**
   * Samples are deduped to one per second, and several stores can trigger a
   * poll, so the worst case is one a second: 3600 covers the full hour.
   */
  constructor(capacity = 3600) {
    this.ring = new RingBuffer<Sample>(capacity);
  }

  add(v: number | null, t = Date.now()): void {
    const prev = this.ring.last();
    // Several stores can trigger the same poll result; keep one sample per second.
    if (prev && t - prev.t < 1000) return;
    this.ring.push({ t, v });
  }

  lastHour(now = Date.now()): Sample[] {
    return windowed(this.ring.toArray(), now);
  }

  clear(): void {
    this.ring.clear();
  }
}
