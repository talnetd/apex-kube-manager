// src/lib/utils/poll.ts
/**
 * List refresh interval (CLAUDE.md, Architecture Note 3). ListView polls
 * with `{ poll: true }`, so a tick is skipped while a load is in flight.
 */
export const LIST_POLL_MS = 10000;
