// src/lib/stores/production.ts
// Production guard logic (spec 4.6). Visual markers land in PR6; this module
// is the single source of truth for "is this context production?", and
// ConfirmDialog reads it itself so no call site can forget it.
import { derived, get, type Readable } from "svelte/store";
import { settings } from "./settings";
import { currentContext } from "./kubernetes";

/**
 * Context names containing `prod` or `prd` (substring, case-insensitive)
 * are auto-flagged. Missing a production cluster is the unsafe direction, so
 * glued names (`eksprod`, `preprod`, `gke-prod1`) count. Only the common
 * false positive "product" is excluded (but "production" still counts);
 * anything else is fixed with an unmark.
 */
export const AUTO_PROD_PATTERN = /prod(?!uct(?!ion))|prd/i;

export type ProdSource = "manual" | "auto" | "none";

export function isAutoProd(contextName: string): boolean {
  return AUTO_PROD_PATTERN.test(contextName);
}

/** Pure resolver: a manual override wins, else the name pattern. */
export function resolveProd(
  contextName: string | null | undefined,
  overrides: Record<string, boolean>
): boolean {
  if (!contextName) return false;
  const override = overrides[contextName];
  if (typeof override === "boolean") return override;
  return isAutoProd(contextName);
}

/** Why a context is (or isn't) production, for the Settings list (PR6). */
export function prodSource(
  contextName: string,
  overrides: Record<string, boolean> = get(settings).prodOverrides
): ProdSource {
  if (typeof overrides[contextName] === "boolean") return "manual";
  return isAutoProd(contextName) ? "auto" : "none";
}

/** Snapshot check against the current settings. */
export function isProdContext(contextName: string | null | undefined): boolean {
  return resolveProd(contextName, get(settings).prodOverrides);
}

/** Reactive check for a fixed context (detail windows use their locked context). */
export function prodContextStore(contextName: string): Readable<boolean> {
  return derived(settings, ($s) => resolveProd(contextName, $s.prodOverrides));
}

export const prodOverrides: Readable<Record<string, boolean>> = derived(
  settings,
  ($s) => $s.prodOverrides
);

export const currentContextIsProd: Readable<boolean> = derived(
  [currentContext, settings],
  ([$ctx, $s]) => resolveProd($ctx, $s.prodOverrides)
);

function setOverride(contextName: string, value: boolean | undefined) {
  settings.update((s) => {
    const next = { ...s.prodOverrides };
    if (value === undefined) delete next[contextName];
    else next[contextName] = value;
    return { ...s, prodOverrides: next };
  });
}

export function markProd(contextName: string) {
  setOverride(contextName, true);
}

/** Callers confirm first when the context is currently production (spec 4.6, tier 1). */
export function unmarkProd(contextName: string) {
  setOverride(contextName, false);
}

export function resetProd(contextName: string) {
  setOverride(contextName, undefined);
}
