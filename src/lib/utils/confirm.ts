// src/lib/utils/confirm.ts
// Pure tier logic for ConfirmDialog (spec 7.4), kept out of the component so
// it is unit-testable and every dialog escalates the same way on production.

export type ConfirmTier = 1 | 2 | 3;

export interface ConfirmResource {
  kind: string;
  name: string;
  namespace?: string;
  context: string;
}

export interface ConfirmInput {
  resource: ConfirmResource;
  confirmLabel: string;
  tone?: 'default' | 'danger';
  /** Type-to-confirm text (tier 3). */
  requireTyping?: string;
  /** Explicit tier; otherwise derived from tone / requireTyping. */
  tier?: ConfirmTier;
  isProd: boolean;
  /** Default true. Only the "unmark production" confirm turns it off. */
  prodEscalation?: boolean;
}

export interface ConfirmBehavior {
  /** Tier as declared by the call site. */
  baseTier: ConfirmTier;
  /** Tier actually applied; 4 = "tier 3+" (type context/name) on production. */
  effectiveTier: 1 | 2 | 3 | 4;
  prod: boolean;
  danger: boolean;
  /** Where focus lands when the dialog opens. */
  initialFocus: 'confirm' | 'cancel' | 'input';
  /** Plain Enter (focus not on a button) confirms. ⌘/Ctrl+Enter always does once allowed. */
  enterConfirms: boolean;
  /** Text the user must type before the confirm button enables, or null. */
  typeToConfirm: string | null;
  confirmLabel: string;
  role: 'dialog' | 'alertdialog';
}

export function baseTierOf(input: Pick<ConfirmInput, 'tier' | 'tone' | 'requireTyping'>): ConfirmTier {
  if (input.tier) return input.tier;
  if (input.requireTyping) return 3;
  if (input.tone === 'danger') return 2;
  return 1;
}

export function resolveConfirm(input: ConfirmInput): ConfirmBehavior {
  const baseTier = baseTierOf(input);
  const prod = input.isProd && input.prodEscalation !== false;
  const effectiveTier = (prod ? baseTier + 1 : baseTier) as 1 | 2 | 3 | 4;
  const danger = input.tone === 'danger' || baseTier >= 2;
  const { name, context } = input.resource;

  let typeToConfirm: string | null = null;
  if (effectiveTier === 3) typeToConfirm = input.requireTyping ?? name;
  if (effectiveTier === 4) typeToConfirm = `${context}/${input.requireTyping ?? name}`;

  return {
    baseTier,
    effectiveTier,
    prod,
    danger,
    initialFocus: effectiveTier === 1 ? 'confirm' : effectiveTier === 2 ? 'cancel' : 'input',
    enterConfirms: effectiveTier === 1,
    typeToConfirm,
    confirmLabel: prod ? `${input.confirmLabel} on ${context}` : input.confirmLabel,
    role: effectiveTier >= 2 ? 'alertdialog' : 'dialog',
  };
}

/** Whether the typed text satisfies type-to-confirm (exact, surrounding whitespace ignored). */
export function typedMatches(behavior: Pick<ConfirmBehavior, 'typeToConfirm'>, typed: string): boolean {
  return behavior.typeToConfirm === null || typed.trim() === behavior.typeToConfirm;
}
