<script lang="ts">
  // Confirmation dialog (spec 7.4). Replaces native confirm(). Reads the
  // production guard itself, so no call site can forget to escalate:
  //   tier 1 (restart, scale, cordon)   -> Enter confirms
  //   tier 2 (delete pod / secret)      -> focus on Cancel, needs Tab+Enter or ⌘↵
  //   tier 3 (delete namespace / PV)    -> type the name
  // On a production context every tier moves up one (tier 3 -> type ctx/name).
  import type { Snippet } from 'svelte';
  import { tick } from 'svelte';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import Button from './Button.svelte';
  import { settings } from '../../stores/settings';
  import { resolveProd } from '../../stores/production';
  import { toast } from '../../stores/toast';
  import {
    resolveConfirm,
    typedMatches,
    type ConfirmResource,
    type ConfirmTier,
  } from '../../utils/confirm';

  let {
    open = $bindable(false),
    title,
    resource,
    body,
    confirmLabel,
    tone = 'default',
    tier,
    requireTyping,
    prodEscalation = true,
    successMessage,
    onConfirm,
    onClose,
  }: {
    open?: boolean;
    title: string;
    resource: ConfirmResource;
    body?: string | Snippet;
    confirmLabel: string;
    tone?: 'default' | 'danger';
    tier?: ConfirmTier;
    requireTyping?: string;
    prodEscalation?: boolean;
    /** Toast shown after onConfirm resolves. */
    successMessage?: string;
    onConfirm: () => Promise<void> | void;
    onClose?: () => void;
  } = $props();

  const uid = Math.random().toString(36).slice(2, 9);
  const titleId = `confirm-title-${uid}`;
  const bodyId = `confirm-body-${uid}`;

  const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
  const modKey = isMac ? '⌘' : 'Ctrl+';

  const isProd = $derived(resolveProd(resource.context, $settings.prodOverrides));
  const behavior = $derived(
    resolveConfirm({ resource, confirmLabel, tone, tier, requireTyping, isProd, prodEscalation })
  );

  let typed = $state('');
  let busy = $state(false);
  let error = $state<string | null>(null);
  let dialogEl = $state<HTMLDivElement>();
  let cancelBtn = $state<HTMLButtonElement>();
  let confirmBtn = $state<HTMLButtonElement>();
  let inputEl = $state<HTMLInputElement>();
  let returnFocus: HTMLElement | null = null;

  const canConfirm = $derived(!busy && typedMatches(behavior, typed));

  // Open / close side effects: reset state, move focus in, restore on close.
  let wasOpen = false;
  $effect(() => {
    if (open && !wasOpen) {
      wasOpen = true;
      returnFocus = (document.activeElement as HTMLElement) ?? null;
      typed = '';
      error = null;
      busy = false;
      tick().then(() => {
        const target =
          behavior.initialFocus === 'input' ? inputEl : behavior.initialFocus === 'cancel' ? cancelBtn : confirmBtn;
        target?.focus();
      });
    } else if (!open && wasOpen) {
      wasOpen = false;
      const el = returnFocus;
      returnFocus = null;
      if (el && typeof el.focus === 'function' && document.contains(el)) el.focus();
    }
  });

  function close() {
    if (busy) return;
    open = false;
    onClose?.();
  }

  async function confirm() {
    if (!canConfirm) return;
    busy = true;
    error = null;
    try {
      await onConfirm();
      busy = false;
      open = false;
      onClose?.();
      if (successMessage) toast(successMessage, { health: 'healthy' });
    } catch (e) {
      busy = false;
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function focusables(): HTMLElement[] {
    if (!dialogEl) return [];
    return Array.from(
      dialogEl.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex="-1"])')
    );
  }

  function onKeydown(e: KeyboardEvent) {
    // Nothing inside the dialog should reach the app's global/list shortcuts.
    e.stopPropagation();

    if (e.key === 'Escape') {
      e.preventDefault();
      close();
      return;
    }

    if (e.key === 'Enter') {
      const target = e.target as HTMLElement;
      if (e.metaKey || e.ctrlKey) {
        e.preventDefault();
        confirm();
        return;
      }
      // A focused button handles Enter natively (Tab to the danger button + Enter).
      if (target.tagName === 'BUTTON') return;
      e.preventDefault();
      if (behavior.enterConfirms) confirm();
      return;
    }

    if (e.key === 'Tab') {
      const items = focusables();
      if (items.length === 0) return;
      const first = items[0];
      const last = items[items.length - 1];
      const active = document.activeElement;
      if (e.shiftKey && (active === first || !dialogEl?.contains(active))) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && (active === last || !dialogEl?.contains(active))) {
        e.preventDefault();
        first.focus();
      }
    }
  }

  const resourceLine = $derived(
    [resource.name, resource.namespace, resource.context].filter(Boolean) as string[]
  );
</script>

{#if open}
  <div
    class="fixed inset-0 z-[800] bg-scrim flex items-start justify-center pt-[14vh] px-4"
    transition:fade={{ duration: 150 }}
    onmousedown={(e) => {
      if (e.target === e.currentTarget) close();
    }}
    role="presentation"
  >
    <div
      bind:this={dialogEl}
      class="w-[440px] max-w-full bg-surface text-primary shadow-dialog rounded-xl p-5"
      role={behavior.role}
      aria-modal="true"
      aria-labelledby={titleId}
      aria-describedby={bodyId}
      data-tier={behavior.effectiveTier}
      tabindex="-1"
      onkeydown={onKeydown}
      transition:scale={{ start: 0.97, duration: 200, easing: cubicOut }}
    >
      {#if behavior.prod}
        <div
          class="mb-4 flex items-center gap-2 bg-warning-tint border border-warning-solid/30 rounded-lg px-3 py-2 text-sm text-warning"
          data-testid="prod-banner"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linejoin="round" class="shrink-0" aria-hidden="true">
            <path d="M8 1.75l5.25 2v4.1c0 3.1-2.2 5.4-5.25 6.4-3.05-1-5.25-3.3-5.25-6.4v-4.1z" />
          </svg>
          <span class="inline-flex items-center h-4 px-1 rounded-sm border border-warning-solid/30 bg-warning-tint text-2xs font-semibold uppercase tracking-[0.04em]">Prod</span>
          <span class="truncate">Production context: {resource.context}</span>
        </div>
      {/if}

      <div class="flex items-start justify-between gap-3">
        <h2 id={titleId} class="text-md font-semibold tracking-[-0.01em]">{title}</h2>
        <button
          type="button"
          class="-mr-1.5 -mt-0.5 w-7 h-7 shrink-0 inline-flex items-center justify-center rounded-md text-muted hover:bg-hover hover:text-primary disabled:opacity-45"
          aria-label="Close"
          disabled={busy}
          onclick={close}
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M4 4l8 8M12 4l-8 8" />
          </svg>
        </button>
      </div>

      <p class="mt-1 text-sm text-muted truncate" data-testid="resource-line">
        <span class="font-mono text-2xs text-muted mr-1">{resource.kind}</span>
        {#each resourceLine as part, i}
          {#if i > 0}<span aria-hidden="true"> · </span>{/if}
          <span class={part === resource.context && behavior.prod ? 'text-warning' : i === 0 ? 'text-primary font-medium' : ''}>{part}</span>
        {/each}
      </p>

      <div id={bodyId} class="mt-4 text-base text-secondary">
        {#if typeof body === 'string'}
          <p>{body}</p>
        {:else if body}
          {@render body()}
        {/if}
      </div>

      {#if behavior.typeToConfirm !== null}
        <label class="mt-4 block text-sm text-secondary">
          Type <span class="font-mono text-primary select-all">{behavior.typeToConfirm}</span> to confirm
          <input
            bind:this={inputEl}
            bind:value={typed}
            type="text"
            autocomplete="off"
            autocapitalize="off"
            spellcheck="false"
            disabled={busy}
            class="mt-1.5 w-full h-8 px-2.5 rounded-md bg-app border border-line hover:border-line-strong text-sm font-mono text-primary focus-visible:outline-2 focus-visible:outline-accent focus-visible:outline-offset-0"
            data-testid="confirm-input"
          />
        </label>
      {/if}

      {#if error}
        <p class="mt-3 text-sm text-danger break-words" role="alert">{error}</p>
      {/if}

      <div class="mt-5 flex items-center justify-end gap-2">
        <Button bind:ref={cancelBtn} size="lg" variant="secondary" kbd="Esc" disabled={busy} onclick={close} data-testid="cancel">
          Cancel
        </Button>
        <Button
          bind:ref={confirmBtn}
          size="lg"
          variant={behavior.danger ? 'danger' : 'primary'}
          kbd={behavior.enterConfirms ? '↵' : `${modKey}↵`}
          loading={busy}
          disabled={!canConfirm}
          onclick={confirm}
          data-testid="confirm"
        >
          {behavior.confirmLabel}
        </Button>
      </div>
    </div>
  </div>
{/if}
