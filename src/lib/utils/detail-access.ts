// src/lib/utils/detail-access.ts
// Detail windows show one object on the context they were opened for
// (spec 13.3). When the shared context moves away, the window stays open
// but read-only until the user switches back. Pure, so it's unit-tested.

export type DetailAction = 'restart' | 'delete' | 'scale' | 'exec' | 'cordon' | 'taint' | 'edit' | 'apply';

export const DETAIL_ACTIONS: readonly DetailAction[] = ['restart', 'delete', 'scale', 'exec', 'cordon', 'taint', 'edit', 'apply'];

export interface DetailAccess {
  readOnly: boolean;
  /** The context the object lives on (locked at open). */
  locked: string;
  /** The shared app context, once known. */
  shared: string | null;
}

/** Read-only only once we know the shared context and it differs. */
export function detailAccess(locked: string, shared: string | null | undefined): DetailAccess {
  const s = shared || null;
  return { readOnly: !!locked && s !== null && s !== locked, locked, shared: s };
}

/** Every mutating action is blocked while read-only; reading never is. */
export function canRun(access: DetailAccess, _action: DetailAction): boolean {
  return !access.readOnly;
}

const VERB: Record<DetailAction, string> = {
  restart: 'restart',
  delete: 'delete',
  scale: 'scale',
  exec: 'exec into',
  cordon: 'cordon',
  taint: 'change taints on',
  edit: 'edit',
  apply: 'apply changes to',
};

export function blockedMessage(access: DetailAccess, action: DetailAction): string {
  return `Switch back to ${access.locked} to ${VERB[action]} this resource`;
}
