import { describe, it, expect } from 'vitest';
import { conditionHealth } from '../src/lib/utils/conditions';

describe('conditionHealth', () => {
  it('treats True as healthy for normal conditions', () => {
    expect(conditionHealth('Available', 'True')).toBe('healthy');
    expect(conditionHealth('Ready', 'False')).toBe('failed');
  });
  it('inverts the node pressure conditions', () => {
    expect(conditionHealth('DiskPressure', 'False')).toBe('healthy');
    expect(conditionHealth('MemoryPressure', 'True')).toBe('failed');
  });
  it('is unknown for anything else', () => {
    expect(conditionHealth('Ready', 'Unknown')).toBe('unknown');
    expect(conditionHealth('Ready', '')).toBe('unknown');
  });
});
