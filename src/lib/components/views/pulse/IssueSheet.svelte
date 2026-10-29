<script lang="ts">
  // Issue side sheet (preview `sheetHtml`): impact, possible trigger,
  // affected pods and evidence events. No delete here: that lives in the
  // detail window only (CLAUDE.md, Architecture Note 7).
  import StatusPill from '../../ui/StatusPill.svelte';
  import Icon from '../../shell/Icon.svelte';
  import { toDisplay } from '../../../utils/health';
  import type { PodInfo, ClusterEventInfo } from '../../../stores/kubernetes';
  import { KIND_SHORT, SEV_LABEL, shortDuration, type Issue, type Trigger } from '../../../pulse/model';

  let {
    issue,
    impact,
    pods,
    events,
    trigger,
    prod,
    now,
    canRestart,
    canScale,
    onClose,
    onLogs,
    onOpen,
    onOpenPod,
    onRestart,
    onScale,
    onCopy,
  }: {
    issue: Issue;
    impact: string;
    pods: PodInfo[];
    events: ClusterEventInfo[];
    trigger: Trigger | null;
    prod: boolean;
    now: number;
    canRestart: boolean;
    canScale: boolean;
    onClose: () => void;
    onLogs?: () => void;
    onOpen: () => void;
    onOpenPod: (p: PodInfo) => void;
    onRestart: () => void;
    onScale: () => void;
    onCopy: () => void;
  } = $props();

  const health = $derived(toDisplay(issue.verdict));
  let closeBtn = $state<HTMLButtonElement>();
  $effect(() => {
    closeBtn?.focus();
  });
  const shown = $derived(pods.slice(0, 8));
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="sheetscrim" onclick={onClose}></div>
<div class="sheet" role="dialog" aria-modal="true" aria-labelledby="pulse-sheet-title">
  {#if prod}<div class="edge" aria-hidden="true"></div>{/if}
  <div class="sh-hd">
    <div class="min-w-0 flex-1">
      <div class="sh-k">
        <span class="sevtag {issue.sev}">{SEV_LABEL[issue.sev].toUpperCase()}</span>
        <span class="mono muted">{KIND_SHORT[issue.kind]}{issue.namespace ? ` · ${issue.namespace}` : ''}</span>
        {#if prod}<span class="prodtag">PROD</span>{/if}
      </div>
      <h2 id="pulse-sheet-title">{issue.name}</h2>
      <div class="sh-m">
        <StatusPill health={health.health} label={issue.reason} variant="dot" />
        <span class="muted tab">for {shortDuration(now - issue.startedAt)}</span>
      </div>
    </div>
    <button bind:this={closeBtn} type="button" class="ib" aria-label="Close" title="Close · Esc" onclick={onClose}><Icon name="close" /></button>
  </div>

  <div class="sh-acts">
    {#if onLogs}<button type="button" class="btn primary" onclick={onLogs}><Icon name="logs" size={14} />Logs</button>{/if}
    <button type="button" class="btn {onLogs ? '' : 'primary'}" onclick={onOpen}><Icon name="open" size={14} />Open</button>
    {#if canRestart}<button type="button" class="btn" onclick={onRestart}><Icon name="restart" size={14} />Restart…</button>{/if}
    {#if canScale}<button type="button" class="btn" onclick={onScale}><Icon name="scale" size={14} />Scale…</button>{/if}
    <button type="button" class="btn ghost" onclick={onCopy}><Icon name="copy" size={14} />Copy</button>
  </div>

  <div class="sh-b">
    <p class="sh-imp">{impact}</p>

    {#if trigger}
      <div class="sh-sec">
        <h4>Possible trigger</h4>
        <div class="trigc">
          <span class="dia" aria-hidden="true">◆</span>
          <div class="min-w-0">
            <b>{trigger.item.text}</b>
            <div class="sec">{trigger.item.detail}</div>
            <div class="muted">
              {trigger.item.by ? `by ${trigger.item.by} · ` : ''}{shortDuration(now - trigger.item.t)} ago · issue started {shortDuration(trigger.afterMs)} later
            </div>
          </div>
        </div>
      </div>
    {/if}

    {#if pods.length}
      <div class="sh-sec">
        <h4>Affected ({pods.length})</h4>
        <ul class="mem">
          {#each shown as p (p.namespace + '/' + p.name)}
            {@const d = toDisplay(p.verdict)}
            <li>
              <button type="button" onclick={() => onOpenPod(p)}>
                <span class="mono">{p.name}</span>
                <StatusPill health={d.health} label={d.label || p.status} variant="dot" />
                <span class="muted tab">{p.restarts} ↻</span>
              </button>
            </li>
          {/each}
        </ul>
        {#if pods.length > shown.length}<span class="muted" style="font-size:12px">{pods.length - shown.length} more in Pods</span>{/if}
      </div>
    {/if}

    {#if events.length}
      <div class="sh-sec">
        <h4>Evidence</h4>
        {#each events.slice(0, 6) as e (e.namespace + '/' + e.name)}
          <div class="ev">
            <StatusPill health="warning" label={`${e.reason} ×${e.count}`} variant="dot" />
            <span class="muted tab">{e.age}</span>
            <p>{e.message}</p>
          </div>
        {/each}
      </div>
    {/if}

    <p class="muted" style="font-size:12px;margin-top:20px">j/k next issue · Esc close. Delete is only available in the detail window.</p>
  </div>
</div>
