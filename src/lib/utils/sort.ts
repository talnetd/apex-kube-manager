// Generic sorting utility for table data

export type SortDirection = 'asc' | 'desc';

export interface SortState {
  field: string;
  direction: SortDirection;
}

export function toggleSort(current: SortState, field: string): SortState {
  if (current.field === field) {
    return { field, direction: current.direction === 'asc' ? 'desc' : 'asc' };
  }
  return { field, direction: 'asc' };
}

export function sortData<T>(data: T[], field: string, direction: SortDirection): T[] {
  return [...data].sort((a, b) => {
    const aVal = getNestedValue(a, field);
    const bVal = getNestedValue(b, field);

    // Handle null/undefined
    if (aVal == null && bVal == null) return 0;
    if (aVal == null) return direction === 'asc' ? 1 : -1;
    if (bVal == null) return direction === 'asc' ? -1 : 1;

    // Handle numbers
    if (typeof aVal === 'number' && typeof bVal === 'number') {
      return direction === 'asc' ? aVal - bVal : bVal - aVal;
    }

    // Handle age strings (e.g., "5d", "2h", "30m", "10s")
    if (field === 'age' || field.endsWith('_age')) {
      const aSeconds = parseAge(String(aVal));
      const bSeconds = parseAge(String(bVal));
      return direction === 'asc' ? aSeconds - bSeconds : bSeconds - aSeconds;
    }

    // Handle strings
    const aStr = String(aVal).toLowerCase();
    const bStr = String(bVal).toLowerCase();
    if (aStr < bStr) return direction === 'asc' ? -1 : 1;
    if (aStr > bStr) return direction === 'asc' ? 1 : -1;
    return 0;
  });
}

function getNestedValue(obj: any, path: string): any {
  return path.split('.').reduce((acc, part) => acc?.[part], obj);
}

function parseAge(age: string): number {
  // Parse age strings like "5d", "2h30m", "45s", etc.
  let totalSeconds = 0;

  const days = age.match(/(\d+)d/);
  const hours = age.match(/(\d+)h/);
  const minutes = age.match(/(\d+)m/);
  const seconds = age.match(/(\d+)s/);

  if (days) totalSeconds += parseInt(days[1]) * 86400;
  if (hours) totalSeconds += parseInt(hours[1]) * 3600;
  if (minutes) totalSeconds += parseInt(minutes[1]) * 60;
  if (seconds) totalSeconds += parseInt(seconds[1]);

  return totalSeconds;
}
