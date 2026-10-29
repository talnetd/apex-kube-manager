// Age parsing for sorting ("5d", "2h30m", "45s" -> seconds).

export function parseAge(age: string): number {
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
