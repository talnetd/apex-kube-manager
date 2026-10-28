# Apex Kube Manager: UI Redesign Spec

Status: proposal, v3 (2026-10-28). Owner decisions folded in (section 10). Pulse is the flagship (section 6).
Scope: Pulse (issues, timeline, capacity, fleet) and its backend, shell, all list views (~32 resource types incl. RBAC, policy and a generic CRD browser), detail windows
Direction: Linear / Raycast. Calm neutral greys, one accent, crisp type, 1px borders, no decoration for its own sake.
North star: **triage**. A broken thing should be visible within one glance of opening the app, from any screen.

This document is the contract for the redesign PRs. It does not change app code. Every value here is meant to be copied, not interpreted.

**Shared with mobile:** triage detection rules live in `spec/triage/` (specified in 6.10), and the status display model (states, labels, icon shapes, colors) lives in **section 11** and is the contract the mobile apps in `docs/mobile/architecture.md` follow too. Tokens in section 2 are shared the same way.

---

## 0. Decisions at a glance

| Topic | Decision |
|---|---|
| Accent | One accent, **indigo** (`#5E63E6` dark / `#4F52D9` light). Teal is retired because it reads as "healthy" next to green status. Owner-approved. |
| Neutrals | 12-step neutral grey scale per theme, slightly cool, mapped to CSS variables. |
| Status | 5 semantic states: `failed`, `warning`, `progressing`, `healthy`, `neutral` (+ `unknown`). Every state has **icon shape + text label + color**. Never color alone. Defined once in **section 11**, shared with mobile. |
| Density | `data-density="compact" \| "comfortable"` on `<html>` drives CSS variables. Default: **comfortable**. Toggle in Settings popover and `⌘⇧D`. |
| Themes | **Dark and Light** only (System follows the OS between the two). Neon is removed in PR1a. Owner-approved. |
| Production guard | Contexts matching `/prod\|prd/i` are auto-flagged, plus manual mark/unmark per context (persisted locally). Window edge line + `PROD` tag; destructive dialogs escalate one tier on prod (4.6, 7.4). |
| Resource coverage | 11 new built-in types (RBAC, StorageClasses, EndpointSlices, IngressClasses, PDBs, ResourceQuotas, LimitRanges, PriorityClasses) plus a generic CRD browser driven by `additionalPrinterColumns` (section 12). |
| Fonts | Inter Variable (bundled) for UI, Hack (bundled) for mono. No new fonts. |
| Styling stack | Tailwind CSS 4, CSS-first: tokens in `@theme` / `@theme inline` in `src/styles/app.css`, no JS config. Runtime `--ui-*` variables switch per theme class. Lands after the Tailwind 4 + Vite deps PR. |
| Shell | 44px header, 220px sidebar with **health badges**, collapsible sections, pins and `:kind` ⌘K jump, 24px **status bar** (new). |
| Tables | One `DataTable` component replaces 20 hand-rolled tables. Problems-first default sort on workload lists. |
| Pulse (flagship) | Rust engine with a ranked, root-cause-grouped Needs Attention feed, change timeline, capacity trends and multi-cluster fleet; 1h in-memory history with optional local persistence and a Prometheus seam (section 6). Built right after the foundation (PR2-PR5). |
| Pod health | Computed in Rust by the shared triage rules (`spec/triage/`, same as mobile), so `CrashLoopBackOff` stops showing as green "Running". |
| Destructive actions | Delete: detail windows only. Restart / scale: detail windows and Pulse quick actions, always through `ConfirmDialog` (prod-escalated). List views get none. |

---

## 1. Audit: top problems in the current UI

Ordered by triage impact first, then visual consistency.

### 1.1 Triage correctness (P0)

1. **CrashLooping pods display as healthy.** Pod `status` is the phase only (`src-tauri/src/kubernetes.rs:811` → `get_pod_phase`). A pod in `CrashLoopBackOff` has phase `Running`, so `StatusBadge` paints it green. `StatusBadge` already knows `crashloopbackoff` (`src/lib/components/ui/StatusBadge.svelte:16`) but never receives it. The reason is available in `PodInfo.containers[].state` (`kubernetes.rs:797`), it just isn't used.
2. **"Failed" pod filter misses most broken pods.** `podsByStatus.failed` is `status === 'Failed'` (`src/lib/stores/kubernetes.ts:1221`). CrashLoop, ImagePullBackOff, OOMKilled, CreateContainerConfigError all land in "Running" or "Pending".
3. **Pulse undercounts failing pods.** `pods_ok` counts any `Running` phase as OK (`kubernetes.rs:2863-2869`). Same root cause, now on the dashboard.
4. **Pulse has no node health.** `PulseMetrics.nodes` is a plain count (`kubernetes.ts:311`). A NotReady node is invisible until you open Nodes.
5. **Pulse shows inventory, not problems.** 16 equal-weight count cards (`src/lib/components/views/Dashboard.svelte:68-85`) where ConfigMaps get the same visual weight as failing Deployments. Warning events aren't on Pulse at all; they hide in a toggleable rail (`src/lib/components/ui/CapacityRail.svelte:122-138`).
6. **Normal events are colored green, as if "success".** `EventList.svelte:56-60`, `PodDetail.svelte:327`. Normal is neutral information. Green on 90% of rows dilutes the meaning of color.
7. **No loading vs empty vs error distinction in lists.** "No pods found" renders while the watch is still connecting, after a filter excludes everything, and on RBAC errors (`src/lib/components/views/PodList.svelte:310-319`; same pattern in all 20 views).

### 1.2 Consistency and safety (P1)

8. **Destructive actions live in list views**, against the project's own rule (CLAUDE.md, Architecture Note 7): pod delete button and `d` key (`PodList.svelte:131-136`, `294-302`), deployment scale/restart (`DeploymentList.svelte:267-277`), statefulset scale/restart (`StatefulSetList.svelte:173-183`), secret delete (`SecretList.svelte:190`).
9. **Native `confirm()` dialogs** for restart (`DeploymentDetail.svelte:217`, `StatefulSetDetail.svelte:215`, `DeploymentList.svelte:176`, `StatefulSetList.svelte:94`, `YamlEditorPanel.svelte:41`). Unstyled, no resource identity, no theme.
10. **Keyboard nav works in only 4 of 20 views** (Pod, Deployment, Service, Node lists wire `totalRows`). The other 16, including Events, ignore `j/k`. Help modal claims it works everywhere.
11. **Selection drift on re-sort** is fixed only in NodeList (`NodeList.svelte:97-120`). Pods and Deployments are watch-driven and re-order too, so `Enter` can open the wrong pod.
12. **20 copies of the same table markup.** Each view hand-writes toolbar, `<table>`, row classes and empty state (e.g. `PodList.svelte:215-321`, `NodeList.svelte:204-302`). Any change is a 20-file change.

### 1.3 Visual direction (P2)

13. **Rainbow palette.** Dashboard maps 15 Tailwind hues to resource types (`Dashboard.svelte:91-113`); header pills use blue/purple icons (`Header.svelte:180,189`). Color carries no meaning, so it can't carry status.
14. **Decorative effects**: gradient ring gauges (`Dashboard.svelte:200-203`, `264-267`), hover glow and `hover:scale-[1.02]` cards (`Dashboard.svelte:330-333`), CRT scanlines in neon theme (`src/styles/app.css:98-112`).
15. **Contrast failures (WCAG AA)**, measured:
    | Pair | Ratio | Needs |
    |---|---|---|
    | teal `#00d4aa` text on white (`app.css:42` used in light theme links/active nav) | 1.91 | 4.5 |
    | white on teal (primary buttons, port-forward badge `Header.svelte:246`) | 1.91 | 4.5 |
    | dark `text-muted #555` on `#0a0a0a` (`app.css:58`) | 2.66 | 4.5 |
    | light `text-muted #9ca3af` on white (`app.css:70`) | 2.54 | 4.5 |
    | amber `#f59e0b` text on white (warnings, restarts) | 2.15 | 4.5 |
    | green `#22c55e` text on white | 2.28 | 4.5 |
16. **No density control.** Fixed `py-3` cells and `text-sm` everywhere (`PodList.svelte:263-283`). Roughly 14 rows per 600px; k9s users expect 25+.
17. **Filter is hidden behind an icon** (`src/lib/components/ui/ViewFilter.svelte:78-88`) and clears itself when collapsed. Filtering is the #1 triage action; it should always be visible.
18. **Sidebar collapses on any click on empty space** (`Sidebar.svelte:78-81`, `onclick={toggleSidebar}` on the `<aside>`). Surprising, and the reason for the `stopPropagation` noise on every child.
19. **Header is overloaded and low signal**: version pill, nodes pill, NS pill, 7 icon buttons (`Header.svelte:171-319`). Namespace dropdown has no search and labels "all" as `*` (`Header.svelte:136,150`).
20. **Same data polled from three places**: `loadPulseMetrics` in Header (`Header.svelte:53`), Dashboard every 5s (`Dashboard.svelte:36-39`), CapacityRail every 10s (`CapacityRail.svelte:16`).
21. **Hardcoded colors bypass tokens**: logo `#20D4A9` (`Sidebar.svelte:87`), scrollbars (`app.css:139-174`), terminal palette (`Terminal.svelte:47-57`). 62 raw Tailwind palette classes (`purple-400`, `blue-400`...) across components.
22. **Micro text**: `text-[10px]` section labels and status (`Sidebar.svelte:107,160`) are below a comfortable minimum for a desktop app; 11px is the floor in this spec.

---

## 2. Design tokens

Target: **Tailwind CSS 4, CSS-first config.** This spec assumes the deps PR has already landed (Tailwind 4 through `@tailwindcss/vite`, latest Vite, `postcss.config.js` and `tailwind.config.js` deleted). All tokens live in `src/styles/app.css`. There is no JS config.

### 2.1 Approach (two layers)

1. **Runtime layer**: plain CSS custom properties named `--ui-*`, holding full color values (hex). They are defined on `:root, .dark` and overridden on `.light`. The theme class stays on `<html>`, set by `src/lib/stores/theme.ts:81-82`. Non-Tailwind code (xterm, canvas, inline styles) reads these.
2. **Tailwind layer**: `@theme inline { --color-app: var(--ui-app); … }` exposes each runtime token as a Tailwind color. `inline` makes utilities emit `var(--ui-app)` at the use site, so switching the class on `<html>` re-themes everything with no rebuild.

Opacity modifiers keep working. In v4, `bg-accent/20` compiles to `color-mix(in oklab, var(--ui-accent) 20%, transparent)`, which works with any color value, so the old `rgb(var(--x) / <alpha-value>)` triplet trick is no longer needed. Legacy classes like `bg-accent-primary/20` keep working the same way through the aliases in 2.8.

Runtime note: `color-mix()` needs Safari 16.2+ / Chromium 111+. Tailwind 4 already requires Safari 16.4+, so the deps PR sets that floor anyway (macOS 13.3+ WKWebView, current WebView2 on Windows, WebKitGTK 2.40+ on Linux).

**Naming.** Tailwind 4 has one flat `--color-*` namespace shared by `bg-`, `text-`, `border-`, `ring-`, `fill-` and so on. Names are chosen so they can't collide:

| Group | Token names | Utilities you write |
|---|---|---|
| Surfaces | `app, subtle, surface, hover, active, selected` | `bg-app`, `bg-hover`… |
| Lines | `divider, line, line-strong` | `border-divider`, `border-line`, `divide-divider`… |
| Foreground | `primary, secondary, muted, faint` | `text-primary`, `text-muted`… (foreground only; the one allowed background use is `bg-muted` for neutral bar fills) |
| Accent | `accent, accent-text, on-accent` | `bg-accent`, `text-accent-text`, `ring-accent`, `outline-accent` |
| Status | `{danger,warning,success,info}`, `-solid`, `-tint`; plus `danger-button` | `text-danger`, `bg-danger-solid`, `bg-danger-tint`, `border-danger/30` |

Everywhere else in this spec, class names refer to these utilities.

### 2.2 Neutral scale

Slightly cool greys. Components use roles, not raw values.

| Runtime var | Utility | Dark | Light | Use |
|---|---|---|---|---|
| `--ui-app` | `bg-app` | `#0E0E10` | `#FFFFFF` | Main content background |
| `--ui-subtle` | `bg-subtle` | `#141416` | `#F8F8F9` | Sidebar, header, status bar, table header |
| `--ui-surface` | `bg-surface` | `#18181B` | `#FFFFFF` | Cards, popovers, dialogs |
| `--ui-hover` | `bg-hover` | `#1F1F23` | `#F2F2F4` | Row / item hover |
| `--ui-active` | `bg-active` | `#26262B` | `#EAEAEE` | Pressed, active nav item, segmented "on" |
| `--ui-divider` | `border-divider` | `#232328` | `#ECECEF` | Row dividers |
| `--ui-line` | `border-line` | `#2E2E34` | `#DEDEE3` | Panels, inputs, cards |
| `--ui-line-strong` | `border-line-strong` | `#3F3F46` | `#C8C8CF` | Input hover, dividers that must read |
| `--ui-primary` | `text-primary` | `#EDEDF0` | `#18181B` | Body, names, values |
| `--ui-secondary` | `text-secondary` | `#B4B4BD` | `#46464E` | Secondary cells, labels |
| `--ui-muted` | `text-muted` | `#8D8D97` | `#6A6A73` | Column headers, meta, timestamps |
| `--ui-faint` | `text-faint` | `#6B6B75` | `#8E8E98` | Placeholders, disabled, `–` empties. **Never for information you need to read.** |

Measured contrast (text on `bg-app` / on `bg-active`):

| Token | Dark | Light |
|---|---|---|
| text-primary | 16.5 / 12.9 | 17.7 / 14.8 |
| text-secondary | 9.4 / 7.3 | 9.4 / 7.8 |
| text-muted | 5.9 / 4.6 | 5.4 / 4.5 |
| text-faint | 3.7 / 2.9 (non-essential only) | 3.2 / 2.7 (non-essential only) |

### 2.3 Accent (single)

| Runtime var | Utility | Dark | Light | Contrast |
|---|---|---|---|---|
| `--ui-accent` (fills, focus ring, selected rail, icons) | `bg-accent`, `ring-accent`, `outline-accent`, `text-accent` (icons only) | `#5E63E6` | `#4F52D9` | white on it: 4.77 dark / 5.93 light. vs bg-app (non-text): 4.0 / 5.9 |
| `--ui-accent-text` (links, active tab label, any accent **text**) | `text-accent-text` | `#9DA2FF` | `#4346C8` | on bg-app: 8.3 / 7.1 |
| `--ui-on-accent` | `text-on-accent` | `#FFFFFF` | `#FFFFFF` | |
| `--ui-selected` (keyboard-selected row) | `bg-selected` | accent at 16% over app | accent at 10% over app | primary text on it 14.3 / 15.4, muted 5.1 / 4.65 |

Rules:
- Accent means **"you are here / you can act"**: focus ring, selected row rail, active nav item marker, primary button, links. It never means status.
- `text-accent` is for icons only. Dark accent is 4.0:1 on bg-app, which passes for non-text but not for text. Text uses `text-accent-text`.
- Resource names in tables are **text-primary, not accent** (the current `text-accent-primary` names at `PodList.svelte:264` turn every row into a link-colored wall). Hover underlines the name.

### 2.4 Status color tokens

Status **values, meanings, icons and rules are defined in section 11** (the shared desktop + mobile contract). This subsection only names the desktop tokens.

- Per state there are three tokens: the bare name is the **fg** color (text, 4.5:1), `-solid` is for icons, dots and bars (3:1 non-text), `-tint` is a precomputed background (12% of solid in dark, 10% in light). Tints are tokens rather than `/12` modifiers, so the per-theme alpha lives in one place.
- State → token prefix: failed → `danger`, warning → `warning`, progressing → `info`, healthy → `success`. Unknown and neutral use the neutral text tokens.
- Utilities: `text-danger`, `bg-danger-solid`, `bg-danger-tint`, and the same for `warning`, `info`, `success`. Borders on status elements use the solid with a modifier, e.g. `border-danger-solid/30`.
- Danger **button** fill is `bg-danger-button`: `#CD2B31` dark (white text 5.27:1), `#DC2626` light (4.83:1).
- `progressing` (blue) is the only non-neutral color besides the accent. Its hue is deliberately different from the indigo accent (blue `#3B82F6` vs indigo `#5E63E6`), and it always comes with the half-circle icon. If it ever feels too close, swap progressing to neutral before touching the accent.

### 2.5 Typography

Fonts are already bundled at `src/assets/fonts/`. The `@font-face` blocks in `app.css:6-28` stay as they are.

Theme variables (in `@theme`, see 2.8):
- `--font-sans`: `'Inter', system-ui, -apple-system, 'Segoe UI', sans-serif`, with `--font-sans--font-feature-settings: 'cv11', 'ss01'`. Inter's straight-tail `l` and open digits make pod hashes easier to read.
- `--font-mono`: `'Hack', ui-monospace, 'SF Mono', Menlo, Consolas, monospace`.
- All numeric table cells and counters use `tabular-nums`.

Scale. The defaults are reset with `--text-*: initial;` so only these sizes exist (this stops drift like `text-[10px]`):

| Utility | Theme vars | Weight | Use |
|---|---|---|---|
| `text-2xs` | `--text-2xs: 11px; --text-2xs--line-height: 16px` | 500 | Section labels (uppercase, `tracking-[0.04em]`), kbd hints, badges in compact |
| `text-xs` | `12px / 16px` | 400 | Table cells (compact), meta, status bar |
| `text-sm` | `13px / 20px` | 400 | **Base UI size.** Table cells (comfortable), nav items, inputs, buttons |
| `text-base` | `14px / 20px` | 400 | Body text in detail panels, dialog copy |
| `text-md` | `16px / 24px` | 600 | Section headings in detail windows, dialog titles |
| `text-lg` | `20px / 28px` | 600 | Page title (comfortable) |
| `text-xl` | `24px / 32px` | 600 | Big numbers on Pulse health strip only |

Weights: 400 regular, 500 medium (labels, active nav, table header), 600 semibold (titles). No 700 in UI chrome.
Page title in compact is 16px (via `--title-font`, see 3.2).
Letter spacing: titles `-0.01em`, uppercase labels `0.04em`, everything else 0.

### 2.6 Spacing, radii, borders, shadows

Spacing: Tailwind 4's `--spacing: 0.25rem` base stays (any `p-N` works). Allowed steps in new code: `0.5 1 1.5 2 3 4 5 6 8` (2, 4, 6, 8, 12, 16, 20, 24, 32px). Density-dependent sizes use the variables in section 3 through v4's variable shorthand, e.g. `h-(--row-h)`, `px-(--cell-px)`.

Radii (defaults reset with `--radius-*: initial;`):

| Utility | Theme var | Value | Use |
|---|---|---|---|
| `rounded-sm` | `--radius-sm` | 4px | Badges, kbd, checkboxes, usage bars |
| `rounded-md` | `--radius-md` | 6px | Buttons, inputs, nav items, table row focus |
| `rounded-lg` | `--radius-lg` | 8px | Cards, panels, popovers, menus, banners |
| `rounded-xl` | `--radius-xl` | 12px | Dialogs, command palette |
| `rounded-full` | built in | 9999px | Status dots, sidebar count badges only |

Don't use bare `rounded`; always name the step.

Borders: always 1px. `border-divider` between rows, `border-line` around panels and inputs. No 2px borders except the 2px selected-row rail and the active tab underline.

Shadows (only for things that float). The values depend on the theme, so they live in the runtime layer and `@theme inline` maps them:

| Utility | Runtime var | Dark | Light |
|---|---|---|---|
| `shadow-popover` | `--ui-shadow-popover` | `0 0 0 1px var(--ui-line), 0 8px 24px rgb(0 0 0 / 0.45)` | `0 0 0 1px var(--ui-line), 0 8px 24px rgb(16 16 24 / 0.10)` |
| `shadow-dialog` | `--ui-shadow-dialog` | `0 0 0 1px var(--ui-line), 0 24px 64px rgb(0 0 0 / 0.6)` | `0 0 0 1px var(--ui-line), 0 24px 64px rgb(16 16 24 / 0.16)` |

Cards, tables and the sidebar get **no shadow**. Inside the window, elevation is just a background step plus a border.

Overlay scrim: `bg-scrim` (`--ui-scrim`: `rgb(0 0 0 / 0.5)` dark, `rgb(16 16 24 / 0.25)` light).

Focus ring (all interactive elements, `:focus-visible` only), set once in `@layer base`: `outline: 2px solid var(--ui-accent); outline-offset: 2px;`. Inside tables and lists, use `outline-offset: -2px` so it doesn't clip.

### 2.7 Motion

| Purpose | Utility | Value | Use |
|---|---|---|---|
| fast | `duration-100` | 100ms | Hover, pressed, color changes |
| base | `duration-150` | 150ms | Popovers, menus, tab underline |
| slow | `duration-200` | 200ms | Dialogs, rail slide, sidebar collapse |
| enter | `ease-out` (`--ease-out: cubic-bezier(0.2, 0, 0, 1)`) | | Everything entering |
| leave | `ease-in` (`--ease-in: cubic-bezier(0.4, 0, 1, 1)`) | | Everything leaving |

Tailwind 4 accepts any `duration-N`, so no theme variable is needed. Svelte transitions use the same numbers (`fly`/`fade` with `duration: 150`, easing `cubicOut`).

Rules:
- Animate opacity and transform only. No `scale` on hover. No pulse animations except the connection dot while `connecting`.
- Rows don't animate in or out when data changes. Watch streams update often, and motion there would be noise. Usage bars may transition width at 150ms.
- `@media (prefers-reduced-motion: reduce)`: all transitions and animations go to `0ms` (one rule in `@layer base`), and the context-switch progress bar (`Header.svelte:330-338`) becomes a static 2px accent bar.

### 2.8 Implementation: `src/styles/app.css`

The whole design-system layer is one file. Order matters: import, theme, runtime tokens, legacy aliases, base.

```css
@import "tailwindcss";

/* @font-face blocks for Inter and Hack: unchanged from today */

/* Legacy: keep the dark: variant pointing at the theme class, in case any view uses it */
@custom-variant dark (&:where(.dark, .dark *));

/* ---------- Tailwind layer ---------- */
@theme {
  --font-sans: 'Inter', system-ui, -apple-system, 'Segoe UI', sans-serif;
  --font-sans--font-feature-settings: 'cv11', 'ss01';
  --font-mono: 'Hack', ui-monospace, 'SF Mono', Menlo, Consolas, monospace;

  --text-*: initial;
  --text-2xs: 11px;  --text-2xs--line-height: 16px;
  --text-xs: 12px;   --text-xs--line-height: 16px;
  --text-sm: 13px;   --text-sm--line-height: 20px;
  --text-base: 14px; --text-base--line-height: 20px;
  --text-md: 16px;   --text-md--line-height: 24px;
  --text-lg: 20px;   --text-lg--line-height: 28px;
  --text-xl: 24px;   --text-xl--line-height: 32px;

  --radius-*: initial;
  --radius-sm: 4px; --radius-md: 6px; --radius-lg: 8px; --radius-xl: 12px;

  --ease-out: cubic-bezier(0.2, 0, 0, 1);
  --ease-in: cubic-bezier(0.4, 0, 1, 1);
}

@theme inline {
  --color-*: initial;            /* drop the default palette: no purple-400 etc. */
  --color-transparent: transparent;
  --color-current: currentColor;
  --color-white: #fff;
  --color-black: #000;

  --color-app: var(--ui-app);         --color-subtle: var(--ui-subtle);
  --color-surface: var(--ui-surface); --color-hover: var(--ui-hover);
  --color-active: var(--ui-active);   --color-selected: var(--ui-selected);
  --color-scrim: var(--ui-scrim);

  --color-divider: var(--ui-divider); --color-line: var(--ui-line);
  --color-line-strong: var(--ui-line-strong);

  --color-primary: var(--ui-primary);     --color-secondary: var(--ui-secondary);
  --color-muted: var(--ui-muted);         --color-faint: var(--ui-faint);

  --color-accent: var(--ui-accent);       --color-accent-text: var(--ui-accent-text);
  --color-on-accent: var(--ui-on-accent);

  --color-danger: var(--ui-danger);   --color-danger-solid: var(--ui-danger-solid);
  --color-danger-tint: var(--ui-danger-tint); --color-danger-button: var(--ui-danger-button);
  --color-warning: var(--ui-warning); --color-warning-solid: var(--ui-warning-solid);
  --color-warning-tint: var(--ui-warning-tint);
  --color-success: var(--ui-success); --color-success-solid: var(--ui-success-solid);
  --color-success-tint: var(--ui-success-tint);
  --color-info: var(--ui-info);       --color-info-solid: var(--ui-info-solid);
  --color-info-tint: var(--ui-info-tint);

  --shadow-popover: var(--ui-shadow-popover);
  --shadow-dialog: var(--ui-shadow-dialog);

  /* Legacy aliases: today's class names keep working until PR7b removes them */
  --color-bg-primary: var(--ui-app);      --color-bg-secondary: var(--ui-subtle);
  --color-bg-tertiary: var(--ui-hover);   --color-bg-card: var(--ui-surface);
  --color-border-subtle: var(--ui-line);
  --color-text-primary: var(--ui-primary);     --color-text-secondary: var(--ui-secondary);
  --color-text-muted: var(--ui-muted);
  --color-accent-primary: var(--ui-accent);    --color-accent-secondary: var(--ui-accent);
  --color-accent-error: var(--ui-danger-solid);
  --color-accent-warning: var(--ui-warning-solid);
  --color-accent-success: var(--ui-success-solid);
}

/* ---------- Runtime layer ---------- */
:root, .dark {   /* dark is the default, so first paint is dark */
  color-scheme: dark;
  --ui-app: #0e0e10; --ui-subtle: #141416; --ui-surface: #18181b;
  --ui-hover: #1f1f23; --ui-active: #26262b;
  --ui-divider: #232328; --ui-line: #2e2e34; --ui-line-strong: #3f3f46;
  --ui-primary: #ededf0; --ui-secondary: #b4b4bd; --ui-muted: #8d8d97; --ui-faint: #6b6b75;
  --ui-accent: #5e63e6; --ui-accent-text: #9da2ff; --ui-on-accent: #fff;
  --ui-danger: #f87171;  --ui-danger-solid: #e5484d;  --ui-danger-button: #cd2b31;
  --ui-warning: #f5b74a; --ui-warning-solid: #f59e0b;
  --ui-success: #4ccf7f; --ui-success-solid: #30a46c;
  --ui-info: #7cb4ff;    --ui-info-solid: #3b82f6;
  --ui-tint: 12%;  --ui-select: 16%;
  --ui-scrim: rgb(0 0 0 / 0.5);
  --ui-shadow-popover: 0 0 0 1px var(--ui-line), 0 8px 24px rgb(0 0 0 / 0.45);
  --ui-shadow-dialog: 0 0 0 1px var(--ui-line), 0 24px 64px rgb(0 0 0 / 0.6);
}
.light {
  color-scheme: light;
  --ui-app: #ffffff; --ui-subtle: #f8f8f9; --ui-surface: #ffffff;
  --ui-hover: #f2f2f4; --ui-active: #eaeaee;
  --ui-divider: #ececef; --ui-line: #dedee3; --ui-line-strong: #c8c8cf;
  --ui-primary: #18181b; --ui-secondary: #46464e; --ui-muted: #6a6a73; --ui-faint: #8e8e98;
  --ui-accent: #4f52d9; --ui-accent-text: #4346c8; --ui-on-accent: #fff;
  --ui-danger: #c4252b;  --ui-danger-solid: #dc2626;  --ui-danger-button: #dc2626;
  --ui-warning: #985700; --ui-warning-solid: #d97706;
  --ui-success: #18794e; --ui-success-solid: #16a34a;
  --ui-info: #1d63c9;    --ui-info-solid: #2563eb;
  --ui-tint: 10%;  --ui-select: 10%;
  --ui-scrim: rgb(16 16 24 / 0.25);
  --ui-shadow-popover: 0 0 0 1px var(--ui-line), 0 8px 24px rgb(16 16 24 / 0.10);
  --ui-shadow-dialog: 0 0 0 1px var(--ui-line), 0 24px 64px rgb(16 16 24 / 0.16);
}
/* Derived tokens: defined once, follow the theme automatically.
   sRGB mixing matches the contrast measurements in 2.3-2.4. */
:root, .dark, .light {
  --ui-danger-tint:  color-mix(in srgb, var(--ui-danger-solid)  var(--ui-tint), transparent);
  --ui-warning-tint: color-mix(in srgb, var(--ui-warning-solid) var(--ui-tint), transparent);
  --ui-success-tint: color-mix(in srgb, var(--ui-success-solid) var(--ui-tint), transparent);
  --ui-info-tint:    color-mix(in srgb, var(--ui-info-solid)    var(--ui-tint), transparent);
  --ui-selected:     color-mix(in srgb, var(--ui-accent)        var(--ui-select), transparent);
}

/* ---------- Base ---------- */
@layer base {
  html { background: var(--ui-app); color: var(--ui-primary); font-family: var(--font-sans); }
  /* v4 defaults borders to currentColor; restore a token default so a bare `border` reads right */
  *, ::before, ::after { border-color: var(--ui-line); }
  :focus-visible { outline: 2px solid var(--ui-accent); outline-offset: 2px; }
  ::selection { background: color-mix(in srgb, var(--ui-accent) 30%, transparent); }
  ::-webkit-scrollbar { width: 10px; height: 10px; }
  ::-webkit-scrollbar-track { background: transparent; }
  ::-webkit-scrollbar-thumb { background: var(--ui-line); border-radius: 6px; border: 3px solid transparent; background-clip: content-box; }
  ::-webkit-scrollbar-thumb:hover { background-color: var(--ui-line-strong); }
  button:not(:disabled), [role="button"]:not(:disabled) { cursor: pointer; } /* v4 removed this default */
  @media (prefers-reduced-motion: reduce) {
    *, ::before, ::after { transition-duration: 0ms !important; animation-duration: 0ms !important; }
  }
}
```

Also delete from `app.css`: the old `.dark, .light` accent block, `.dark` / `.light` triplet blocks, the whole `.neon` section, and the hardcoded scrollbar and selection rules (today's `app.css:39-183`).

Things to know when writing classes against this file:
- **Opacity modifiers work on every token**, legacy and new: `bg-accent-primary/20`, `bg-accent/10`, `border-danger-solid/30`. Tailwind 4 emits `color-mix(in oklab, var(--ui-…) N%, transparent)`.
- `--color-*: initial` removes the default palette on purpose, so `text-purple-400` and friends **stop compiling**. Any leftover raw palette class in a view becomes visibly unstyled, which is how PR7 finds them. If that's too aggressive for the PR1a merge (views still use 62 of them, audit 21), PR1a keeps the default palette and PR7b adds `--color-*: initial` in its last commit. **Decision: keep the palette in PR1a, drop it in PR7b.**
- The aliasing immediately improves light-theme muted text contrast in every untouched view. Old `text-accent-primary` text renders indigo; that's intended.
- Tailwind 4 changes that affect existing markup and belong to the **deps PR**, not this spec: `shadow-sm`→`shadow-xs`, `rounded-sm`→`rounded-xs` and `rounded`→`rounded-sm` renames, `ring` default width 3px → 1px, `outline-none` → `outline-hidden`, `space-*` / `divide-*` selector change, `flex-shrink-0` → `shrink-0`. Run `npx @tailwindcss/upgrade` there. Since this spec resets `--radius-*`, the redesign PRs rewrite radii to the table in 2.6 as they touch each file.
- Svelte `<style>` blocks don't use `@apply` today (checked). If one ever needs it, it must start with `@reference "../styles/app.css";`.
- `Terminal.svelte:47-57`: read `--ui-app`, `--ui-primary` and `--ui-accent` with `getComputedStyle(document.documentElement)` at init and on theme change, so the terminal stops carrying its own palette. ANSI colors stay as they are (they're content, not chrome).

---

## 3. Density system

### 3.1 Mechanism

- `settings.ts` gains `density: 'compact' | 'comfortable'` (default `'comfortable'`), persisted with the existing `apex-kube-manager-settings` key.
- A new `src/lib/stores/density.ts` subscribes and sets `document.documentElement.dataset.density`. It's imported in `App.svelte` next to `./lib/stores/theme` (`App.svelte:31`) **and** in the detail window entry, so detail windows match.
- Detail windows are separate webviews. They read the same `localStorage` key on load and listen to the `storage` event to follow changes live.
- Components never branch on density in JS. They consume CSS variables through Tailwind 4's variable shorthand (`h-(--row-h)`, `px-(--cell-px)`, `text-(length:--cell-font)`) or plain CSS.
- The density variables are runtime variables, **not** `@theme` tokens: they change at runtime with the attribute, and they don't need their own utilities.

### 3.2 Variables and values

```css
:root, [data-density="comfortable"] {
  --row-h: 36px;        --cell-px: 12px;     --cell-font: 13px;
  --header-row-h: 32px; --page-px: 24px;     --page-py: 16px;
  --toolbar-gap: 12px;  --nav-item-h: 30px;  --nav-font: 13px;
  --control-h: 28px;    --badge-h: 20px;     --title-font: 20px;
  --section-gap: 24px;  --card-p: 16px;
}
[data-density="compact"] {
  --row-h: 26px;        --cell-px: 8px;      --cell-font: 12px;
  --header-row-h: 26px; --page-px: 16px;     --page-py: 10px;
  --toolbar-gap: 8px;   --nav-item-h: 26px;  --nav-font: 12px;
  --control-h: 24px;    --badge-h: 18px;     --title-font: 16px;
  --section-gap: 16px;  --card-p: 12px;
}
```

What changes and what doesn't:

| | Compact | Comfortable |
|---|---|---|
| Rows visible in 600px table | ~22 | ~16 |
| Row content | Single line, truncate with `title` tooltip | Single line too. No second lines in either mode, so keyboard scanning stays predictable |
| Status in table | `StatusPill` variant `dot` (icon + text, no tint background) | `StatusPill` variant `soft` (icon + text on tint) |
| Column set | Same columns. Compact additionally shows `IP` and `Node` on Pods by default | Same |
| Kbd hints in buttons | Shown | Shown |
| Hit targets | Min 24px (desktop pointer, WCAG 2.2 AA 2.5.8 minimum) | Min 28px |

Header, sidebar width, status bar and dialogs do not change with density. Density is about content, not chrome.

### 3.3 Where the toggle lives

1. **Settings popover** in the header (gear icon, replaces separate ThemeToggle + NotificationToggle buttons). Contents:
   ```
   ┌ Settings ─────────────────────────┐
   │ Theme      [ Light | Dark | Auto ] │
   │ Density    [ Compact | Comfortable]│
   │ Warning notifications        [on]  │
│ Production contexts           ›    │
   │ ───────────────────────────────── │
   │ Keyboard shortcuts            ?    │
   └───────────────────────────────────┘
   ```
2. **Shortcut `⌘⇧D` / `Ctrl+Shift+D`** toggles, shows a 1.5s toast "Density: Compact".
3. **Status bar** shows the current mode as text (`Compact`), clickable to toggle.
4. Command palette entry "Toggle density" once the palette exists (TODO in CLAUDE.md).

---

## 4. Shell

### 4.1 Layout

```
┌──────────────┬──────────────────────────────────────────────────────────────────────┐
│ ◆ Apex       │ [● prod-eu-1 ▾] [ns: all ▾]   [⌕ Search or jump to…   ⌘K]   ⟳  ▤  ⇄2  ⚙  ●○○ │ 44px header (drag region)
├──────────────┼──────────────────────────────────────────────────────────────────────┤
│ ◉ Pulse   7  │                                                                      │
│              │                                                                      │
│ WORKLOADS    │                         page content                                 │
│  Pods     4  │                                                                      │
│  Deploym. 2  │                                                                      │
│  StatefulS.  │                                                                      │
│  DaemonSets  │                                                                      │
│  ReplicaSets │                                                                      │
│  Jobs     1  │                                                                      │
│  CronJobs    │                                                                      │
│ NETWORK      │                                                                      │
│  …           │                                                                      │
│ CLUSTER      │                                                                      │
│  Nodes    1  │                                                                      │
│  Events  37▲ │                                                                      │
│  …           │                                                                      │
├──────────────┴──────────────────────────────────────────────────────────────────────┤
│ ● Connected · prod-eu-1 · v1.30.4 · 12 nodes · live · synced 2s ago   Comfortable  ⇄ 2 forwards  ? │ 24px status bar
└─────────────────────────────────────────────────────────────────────────────────────┘
```

CapacityRail (`⌘E`) stays as a right-side overlay panel under the header, restyled per tokens (no `▚`, `[WARN]` glyphs; uses `StatusPill` + `UsageBar`). It's useful while on a list view; Pulse has the full version.

### 4.2 Sidebar

- Width 220px expanded, 48px collapsed. Background `bg-subtle`, right border `border-line`.
- Top 44px: logo mark (16px square, `bg-accent`, rounded-sm, replaces `#20D4A9` at `Sidebar.svelte:87`) + "Apex" `text-sm` 600. Drag region.
- **Remove the `onclick` on `<aside>`** (`Sidebar.svelte:80`). Collapse only via the footer button or `⌘B`.
- Sections: labels `text-2xs` uppercase `text-muted`, 16px top padding, 4px bottom. Collapsed: labels become a 1px `border-divider` divider.
- Sections and contents. ~32 entries, so sections are **collapsible** and the most-used views get **pinning** and a **⌘K jump** (below):
  - Pulse
  - Events (moved up: it's a triage view)
  - PINNED (only shown when the user has pins)
  - WORKLOADS: Pods, Deployments, StatefulSets, DaemonSets, ReplicaSets, Jobs, CronJobs
  - NETWORK: Services, Endpoints, Ingresses, Ingress Classes, Network Policies
  - CONFIG: ConfigMaps, Secrets
  - STORAGE: Persistent Volumes, PV Claims, Storage Classes
  - SCALING & POLICY: HPAs, Pod Disruption Budgets, Resource Quotas, Limit Ranges, Priority Classes
  - ACCESS CONTROL: Service Accounts, Roles, Role Bindings, Cluster Roles, Cluster Role Bindings
  - CLUSTER: Nodes, Namespaces
  - CUSTOM RESOURCES: Definitions (the CRD list), then up to 8 CRD kinds (pinned kinds first, then recently opened), grouped visually by API group with the group as a `text-2xs text-faint` suffix (`Certificate · cert-manager.io`)
  - Full names everywhere ("Network Policies", "Pod Disruption Budgets"); truncate with ellipsis if needed.
- **Collapsible sections**: the section label is a button (chevron on hover, `aria-expanded`). Collapse state is persisted in settings. Default: Workloads, Network, Config, Storage and Cluster expanded; Scaling & Policy, Access Control and Custom Resources collapsed. A **collapsed section still shows the worst badge of its items** next to its label (e.g. `SCALING & POLICY  ▲ 2`), so problems are never hidden by collapsing. `⌥`-click a section label to collapse/expand all.
- **Pinning**: hovering an item shows a pin icon on the right (replaced by the badge when one is present; the pin then lives in the right-click menu). Right-click menu: Pin / Unpin, Open in new detail window (where relevant). Pins (max 8, any view including CRD kinds) appear in PINNED in pin order; drag to reorder. Pinned items also stay in their own section.
- **⌘K jump**: GlobalSearch gets a "Go to" group listing every view and every CRD kind, matched on name, plural and kubectl short names (`po, deploy, sts, ds, rs, job, cj, svc, ep, ing, ingclass, netpol, cm, secret, pv, pvc, sc, hpa, pdb, quota, limits, pc, sa, role, rb, clusterrole, crb, no, ns, crd`, plus each CRD's `shortNames`). Typing a leading `:` (k9s style, e.g. `:pdb`) restricts results to "Go to" and Enter jumps straight there.
- Item: height `--nav-item-h`, `px-2`, gap 8px, icon 16px `text-muted`, label `--nav-font`.

| State | Style |
|---|---|
| Default | `text-secondary`, transparent bg |
| Hover | `bg-hover`, `text-primary` |
| Active (current view) | `bg-active`, `text-primary`, font 500, icon `text-accent`. No colored background tint. |
| Focus-visible | focus ring, offset -2px |
| Collapsed | icon only, tooltip with label + badge count after 400ms |

- **Health badges** (right-aligned, only when > 0, hidden when that source's coverage isn't `ok`). They all come from the Pulse engine snapshot, so the sidebar and Pulse can never disagree:

| Item | Badge value | Source | Style |
|---|---|---|---|
| Pulse | active issues (not muted) | `pulseSnapshot.issues` (6.13) | worst severity |
| Pods | affected pods | `pulseSnapshot.counts.pods_affected` | worst severity of pod-rooted and workload issues |
| Deployments / StatefulSets / DaemonSets | issues rooted at that kind | `pulseSnapshot` issues by root kind | worst severity |
| Jobs / CronJobs | Job and CronJob issues | same | worst severity |
| Nodes | node issues | same | worst severity |
| PV Claims | PVC issues | same | worst severity |
| Events | Warning events, last 1h | `pulseSnapshot.counts.warning_events_1h` | warning, shown as `37` with a ▲ glyph |
| Pod Disruption Budgets, Resource Quotas | their issues (PR8) | same | worst severity |
| Collapsed section label | worst badge of its items | derived | worst state |

  Badge: height 18px, min-width 18px, `px-1.5`, `rounded-full`, `text-2xs` 600 tabular, `bg-danger-tint text-danger` or `bg-warning-tint text-warning`. Counts above 99 show `99+`. Collapsed sidebar: badge becomes a 6px dot at the icon's top-right corner, and the item's `aria-label` carries the number.
  Accessibility: the item's accessible name is "Pods, 4 failing". Not color only: number is always present, failed vs warning also differs by the ▲ glyph on warning badges.

- Footer: collapse button (`«`/`»`) only. Connection status moves to the status bar.

### 4.3 Header (44px)

Background `bg-subtle`, bottom border `border-line`, `px-3`, gap 8px. Whole empty area is `data-tauri-drag-region`.

Left to right:

1. **Context switcher** (button, `--control-h` 28px, max-width 220px):
   `[● ctx-name ▾]`. The dot is connection state (success / warning+pulse / danger) with `aria-label`.
   Menu (`shadow-popover`, 320px): search input at top (autofocus, filters as you type), list of contexts with cluster name as a second line in `text-muted`, current marked with a check icon, `↑↓ Enter` to choose. Keyboard: `⌘⇧K` opens it.
   While switching: button shows a spinner and "Switching…", and the 2px indeterminate accent bar at the header bottom stays (`Header.svelte:322-326`).
   Production contexts get the shield, `PROD` tag and window edge line from 4.6, and each menu row has the mark/unmark toggle.
2. **Namespace switcher**: `[ns: All namespaces ▾]`. Same searchable menu. First item "All namespaces" (replaces `*`). Recently used namespaces (last 5, per context, localStorage) pinned at top under a "Recent" label. Keyboard: `⌘⇧N`.
3. Flexible drag spacer.
4. **Search entry**: a field-shaped button, 280px, `bg-app`, `border-line`, placeholder "Search or jump to…", `⌘K` kbd on the right. Opens `GlobalSearch`. This makes the existing ⌘K discoverable.
5. Icon buttons (28px square, ghost): Refresh `⟳` (spins while `isLoading`), Pulse rail `▤` (`⌘E`, active state = `bg-active` + `text-primary`), Port forwards `⇄` (count badge in neutral `bg-active text-primary`, not accent; `⌘P`), Update available (only when present, `text-success` icon + dot), Settings `⚙`.
6. Window controls, separated by a 1px divider.

Removed from header: K8s version pill, Nodes pill, NS count pill (`Header.svelte:171-196`). Version and node count move to the status bar.

### 4.4 Status bar (new, 24px)

`bg-subtle`, top border `border-line`, `text-xs text-muted`, `px-3`, items separated by `·`.

Left: connection (`● Connected` / `◐ Connecting` / `✕ Disconnected`, colored icon + word), context, k8s version, node count (`12 nodes`, or `11/12 ready` in warning color when not all ready), data freshness: `live` when watch streams are active for the current view, else `synced 4s ago` (from the last successful load timestamp; turns warning color after 30s with "stale").
Right: density (clickable), port-forward count (clickable, opens panel), `?` shortcuts (clickable).

Data freshness needs a small store addition: `lastSyncedAt` writable in `kubernetes.ts`, set by each load/watch callback. No backend change.

### 4.5 One source for health data

The frontend no longer polls for health. The Rust Pulse engine (6.9) runs for the current context and pushes `pulse://snapshot`; `src/lib/stores/pulse.ts` subscribes once in `App.svelte`. Sidebar badges, the status bar, the CapacityRail and Pulse all read that store. The three existing `loadPulseMetrics` callers (`Header.svelte:53`, `Dashboard.svelte:36-39`, `CapacityRail.svelte:14-24`) are removed (PR2 for Dashboard, PR6 for Header and the rail). Every payload carries `context`, and the store drops payloads for a context that isn't current (same guard as `loadNodeMetrics`).

### 4.6 Production guard

Goal: you always know when you're on production, and destructive actions on production take one more deliberate step. Calm, but impossible to miss.

**Which contexts count as production**
- **Auto-flag**: a context whose **name** matches `/prod|prd/i` (substring, case-insensitive). Only the context name is checked, not the cluster or user name. False positives (`products-dev`) are expected and fixed with an unmark.
- **Manual override** per context: `prodOverrides: Record<contextName, boolean>` in the `apex-kube-manager-settings` localStorage object. `true` = production, `false` = not production, absent = auto.
- Effective value: override if present, else auto. `src/lib/stores/production.ts` exports `isProdContext(name)`, a derived `currentContextIsProd`, `markProd(name)`, `unmarkProd(name)`, `resetProd(name)`.
- Detail and terminal windows evaluate their **locked** context (not the main window's current one) and follow changes through the `storage` event.

**Visual treatment** (marker, not a status: shield icon + the word "PROD", warning colors, never the triangle icon from 11.1)
- **Window edge**: a 2px `bg-warning-solid` line across the full width at the very top of the main window, every detail window and every terminal window on a prod context.
- **Context switcher button**: shield icon replaces the connection dot's left slot (the dot moves after the name), the name, then a `PROD` tag (`text-2xs` 600 uppercase, `bg-warning-tint text-warning`, 1px `border-warning-solid/30`, `rounded-sm`, height 16px). Button border `border-warning-solid/40`. Accessible name: "Context prod-eu-1, production".
- **Context menu rows**: prod contexts show the shield and the `PROD` tag; auto-detected ones add a `text-2xs text-faint` "auto" hint.
- **Status bar**: context segment reads `⛨ prod-eu-1 · Production` in `text-warning`.
- **Detail window titlebar**: `PROD` tag next to the context name (7.1).
- **Switching into a prod context**: one toast, "Switched to production context prod-eu-1", with the shield icon, 4s. No modal.

```
┌──────────────────────────────────────────────────────────────────────── 2px warning line ┐
│ [⛨ prod-eu-1 ● PROD ▾] [ns: payments ▾]      [⌕ Search or jump to…  ⌘K]   ⟳ ▤ ⇄ ⚙  ●○○ │
```

**Where the toggle lives**
1. **Context switcher menu**: hover a row → trailing shield toggle button ("Mark as production" / "Unmark as production"); also in the row's right-click menu, together with "Reset to auto" when overridden.
2. **Settings popover → Production contexts**: one row per context with a "Production" checkbox and "auto" / "manual" source label; "Reset" link on manual rows.
3. **⌘K command**: "Mark current context as production" / "Unmark current context".

Marking is instant. **Unmarking** a context that's currently production asks for a tier-1 confirm: "Treat prod-eu-1 as non-production? Safety prompts on this context will be lighter." Reset to auto applies immediately.

**How it reinforces confirmations** (details in 7.4): on a prod context every destructive dialog shows a production banner, names the context in the confirm button, and **escalates one tier** (tier 1 → tier 2 behavior, tier 2 → type-to-confirm, tier 3 → type `context/name`). Non-destructive actions (logs, exec, port-forward, copy) never get prompts; exec windows on prod just carry the window edge and the PROD tag.

---

## 5. Shared components

All new components live in `src/lib/components/ui/`. Svelte 5 runes, props typed, snippets for slots.

### 5.1 Health model (`src/lib/utils/health.ts`)

Health is computed **in Rust** by the shared triage rules (`src-tauri/src/triage/`, 6.10), and each `*Info` struct carries a `verdict`. The frontend never re-derives health from raw fields, so list views, Pulse, the sidebar and mobile all agree.

`health.ts` only maps a verdict to display:

```ts
export type Health = 'failed' | 'warning' | 'progressing' | 'healthy' | 'neutral' | 'unknown';
export interface Verdict { status: 'ok' | 'warn' | 'fail' | 'unknown'; reason: string; ruleId: string;
  since?: string; severity?: 'critical' | 'error' | 'warning'; tone?: 'progressing' | 'neutral' }
export function toDisplay(v: Verdict): { health: Health; label: string; rank: number; severity?: string }
// fail→failed, warn→warning, unknown→unknown, ok+progressing→progressing, ok+neutral→neutral, ok→healthy
// rank per 11.1
```

Rules, labels and thresholds: **sections 6.10 and 11.5**. `label` is the verdict `reason`; for pods that's the most severe container reason (`CrashLoopBackOff`), not the phase. This is the fix for audit 1.1-1.3. `podsByStatus` in `kubernetes.ts:1216` is rewritten on top of verdicts with buckets `all, failed, warning, progressing, healthy, completed`.

### 5.2 `StatusPill.svelte`

Props: `health: Health`, `label: string`, `variant?: 'soft' | 'dot'` (default follows density: comfortable `soft`, compact `dot`), `title?: string` (tooltip, e.g. full container message).

Icons, colors and label rules: **section 11.1-11.3**. Icons are 12px, stroke 2, from `status-icons.ts`.

- `soft`: height `--badge-h`, `px-1.5`, `rounded-sm`, `bg-{state}-tint`, label in `{state}` fg, `text-2xs` 500 in compact / `text-xs` 500 comfortable.
- `dot`: no background, icon + label in fg color.
- Labels are never truncated below 12 chars; longer reasons truncate with tooltip.
- Replaces `StatusBadge.svelte`, `Badge.svelte` status use, NodeList `getStatusColor` (`NodeList.svelte:195-202`), EventList `getTypeColor` (`EventList.svelte:56-64`), PodDetail `getEventTypeColor` (`PodDetail.svelte:327`), ResourceDetail status helpers (`ResourceDetail.svelte:~200-215`).

### 5.3 `PageHeader.svelte`

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│ Pods  248                     [All 248][✕ Failing 4][▲ Warning 3][◐ Pending 2]   │
│                               [⌕ Filter pods…            ⌘F]  [⋯]               │
└──────────────────────────────────────────────────────────────────────────────────┘
```
One row when width ≥ 1100px, wraps to two rows below.

Props: `title`, `count`, `subtitle?` (e.g. "cluster-scoped"), `segments?: {id,label,count,health?}[]`, `activeSegment`, `onSegment`, `filter: string` (bindable), `filterPlaceholder`, `actions?: Snippet`.

- Padding `var(--page-py) var(--page-px)`, bottom border `border-divider`, background `bg-app`.
- Title `--title-font` 600 `text-primary`; count `text-sm text-muted tabular-nums`.
- **Segmented status filter**: buttons `--control-h`, `text-sm`, inactive `text-secondary` hover `bg-hover`, active `bg-active text-primary` with 1px `border-line`. Segment with `health` shows its pill icon. A segment with count 0 is shown but disabled (`text-faint`), except "All". Keys `1`–`5` select segments when focus isn't in an input.
- **Filter input** always visible (replaces `ViewFilter`'s collapse, audit 17), 240px, `bg-app`, `border-line`, hover `border-line-strong`, focus ring. Leading search icon, trailing `⌘F` kbd when empty, `✕` clear button when not empty. `Esc` clears then blurs. Filter value persists per view for the session (in-memory map keyed by view id), so switching Pods → Nodes → Pods keeps it.
- Actions slot: only non-destructive things (Create secret, Create network policy, column options).

### 5.4 `DataTable.svelte`

The main deliverable of PR1a. Replaces the table markup in all list views.

**Props**

```ts
interface Column<T> {
  id: string; label: string;
  sortKey?: keyof T | ((r: T) => string | number);   // omit = not sortable
  align?: 'left' | 'right';                           // numbers right-aligned
  width?: string;                                     // e.g. '28%', '96px'; name column flexible
  mono?: boolean; numeric?: boolean;                  // numeric => tabular-nums + right
  hideBelow?: number;                                 // px viewport width
  cell?: Snippet<[T]>;                                // custom render; default = String(row[id])
}
interface Props<T> {
  rows: T[]; columns: Column<T>[];
  rowKey: (r: T) => string;              // e.g. `${ns}/${name}`; used for selection anchoring
  health?: (r: T) => HealthResult;       // drives row rail, tint, "problems first" sort
  defaultSort?: { id: string; dir: 'asc'|'desc' };  // workloads default: 'health' asc
  state: 'loading' | 'ready' | 'error' | 'forbidden';
  error?: string;
  filtered?: boolean;                    // true when filter/segment hides rows → filtered-empty state
  onOpen: (r: T) => void;                // click / Enter
  onClearFilter?: () => void;
  emptyState?: { title: string; hint?: string; icon?: string };
  rowActions?: Snippet<[T]>;             // non-destructive only (logs, exec, copy)
  shortcuts?: Record<string, (r: T) => void>;  // e.g. { l: openLogs, e: exec, y: copyName }
}
```

**Behaviour it owns** (removing it from every view):
- Sets `totalRows`, reads `selectedRowIndex` / `keyboardNavActive` from `stores/keyboard.ts`. All 20 views get `j/k/gg/G/Enter` for free (audit 10).
- **Anchors selection by `rowKey`** after every re-sort or data update: the NodeList logic (`NodeList.svelte:97-120`) moves into DataTable verbatim and applies to all tables (audit 11). If the anchored row disappears, selection moves to the row now at the same index and the row briefly isn't highlighted with the rail (no jump to top).
- Scrolls the selected row into view (`block: 'nearest'`, **not smooth** in compact, smooth in comfortable).
- Registers its own keydown listener only while mounted; `Enter`, `y`, `r` built in; view-specific keys via `shortcuts`. Removes `d` (delete) from list views (audit 8). `d` on a list opens the detail window instead, with a toast "Delete from the detail window", so muscle memory isn't punished silently. Update `keyboardShortcuts` in `keyboard.ts:89` accordingly.
- Sorting: click header toggles asc → desc → default. `aria-sort` set on the `<th>`. A synthetic column id `'health'` sorts by `rank` then name; it's the default for Pods, Deployments, StatefulSets, DaemonSets, ReplicaSets, Jobs, Nodes, PVCs, Events (warnings first, then newest).
- Sticky header.
- Virtualization: **not in PR1a**. Note for later: lists over ~2,000 rows (pods in big clusters) will need windowing; design the row component so it can be moved into a windowed container without API change.

**Anatomy**

```
 ┌ th (sticky, bg-subtle, h = --header-row-h, text-2xs uppercase 500 text-muted) ─────────┐
 │  NAME ▲         NAMESPACE     STATUS              READY   RESTARTS   AGE      NODE      │
 ├────────────────────────────────────────────────────────────────────────────────────────┤
 ▌ api-7f9c-x2     payments      ✕ CrashLoopBackOff   0/1        14 ▲    3m      ip-10-1  │  ← failed row
 │ web-5d8-kq      frontend      ✓ Running            2/2         0      2d      ip-10-2  │
 ▐█checkout-9z     payments      ▲ Not ready          1/2         2      1h      ip-10-3 █│  ← keyboard-selected
 └────────────────────────────────────────────────────────────────────────────────────────┘
```

- Row: height `--row-h`, cells `padding: 0 var(--cell-px)`, `font-size: var(--cell-font)`, bottom border `border-divider`. Name column: `text-primary` 500, truncates, full name in `title`. Other text cells `text-secondary`. Numeric cells right-aligned tabular.
- Restarts cell: `text-warning` + ▲ icon when ≥ 5, `text-danger` when ≥ 20; otherwise `text-secondary`. (Current: any restart > 0 is amber at `PodList.svelte:276`, which is noisy.)

**Row states**

| State | Visual | Notes |
|---|---|---|
| Default | `bg-app` | |
| Hover | `bg-hover` | cursor pointer; row actions fade in (opacity 0 → 1, 100ms) |
| Keyboard-selected | `bg-selected` (accent 16% dark / 10% light), plus **2px left rail** `bg-accent` (inset box-shadow `inset 2px 0 0 var(--ui-accent)`) | Replaces ring+tint (`PodList.svelte:261`). Primary text keeps 14.3:1 / 15.4:1, muted 5.1:1 / 4.65:1 on this background (measured). |
| Selected + hover | same as selected, no extra change | |
| Focus-visible (Tab into table) | focus ring inset on the row | The table container is one tab stop (`tabindex=0`, `role="grid"`); arrow keys move rows (roving), so `Tab` doesn't walk 300 rows. |
| Failed row | 2px left rail `bg-danger-solid`; status pill does the rest. **No full-row red tint** (a CrashLoop storm would make the whole table red and unreadable). | Rail is decorative; meaning is carried by the pill. |
| Warning row | 2px left rail `bg-warning-solid` | |
| Failed + selected | accent rail wins on the left, danger keeps a 2px rail on the **right** edge | Both signals visible at once. |
| Deleted (watch removed it while selected) | row stays 1 poll with `text-faint` + strikethrough name, then removed | Avoids a selected row vanishing under the cursor. |

**Table states**

| State | Render |
|---|---|
| `loading`, no rows yet | Header + 8 skeleton rows: `--row-h`, bars `bg-hover` `rounded-sm`, widths 40% / 15% / 12% / 8%…, shimmer off (static) when reduced motion, else opacity 0.5 ↔ 1 at 1.2s. |
| `loading`, rows present (refresh) | Keep rows; 2px indeterminate accent bar under PageHeader. Never blank the table on refresh. |
| `ready`, 0 rows, not filtered | `EmptyState` "No pods in payments" + hint "Switch namespace or check the context." + button "All namespaces" when a namespace is selected. |
| `ready`, 0 rows, filtered | `EmptyState` "No pods match "xyz"" + button "Clear filter" (calls `onClearFilter`). |
| `error` | `EmptyState` variant error: danger icon, title "Couldn't load pods", error text in mono `text-xs`, buttons "Retry" (primary) and "Copy error". |
| `forbidden` (403) | `EmptyState` variant neutral lock icon: "No access to pods in payments", hint "Your kube user can't list this resource." Not red: it's not broken, it's RBAC. |

The store layer needs `{ status, error }` per resource so DataTable can tell these apart. Add a tiny helper `resourceState<T>(name)` in `kubernetes.ts` returning a writable `{ status: 'idle'|'loading'|'ready'|'error'|'forbidden', error?: string }`, set inside each `load*` / `start*Watch` function's try/catch. Backend errors already come through `thiserror`; map 403 by message prefix until the error type carries a code.

**Row actions** (comfortable and compact): right-most column, 1–2 ghost icon buttons (24px) shown on hover/selection only, never destructive. Pods: Logs, Exec. Others: none by default. Each has a tooltip with its key (`Logs  L`).

### 5.5 `EmptyState.svelte`

Props: `variant: 'empty' | 'filtered' | 'error' | 'forbidden' | 'unavailable'`, `title`, `hint?`, `icon?`, `actions?: Snippet`, `detail?: string` (mono block).
Layout: centered in the table body, max-width 360px, 48px from top (not vertically centered: keeps the eye near the header). Icon 24px `text-muted` (error: `text-danger`), title `text-sm` 500 `text-primary`, hint `text-sm text-muted`, actions gap 8px, 12px above.

### 5.6 `Button.svelte` and `IconButton.svelte`

Variants: `primary` (bg-accent, text-on-accent), `secondary` (bg-surface, border-line, text-primary), `ghost` (transparent, text-secondary), `danger` (bg-danger-button, white), `danger-ghost` (transparent, text-danger, used in menus).
Sizes: `sm` 24px / `text-xs` / px-2; `md` 28px / `text-sm` / px-2.5 (default); `lg` 32px / `text-sm` / px-3 (dialogs).
Optional `icon` (16px, leading), `kbd` (trailing `Kbd` hint), `loading` (spinner replaces icon, label stays, width fixed so it doesn't jump).

| State | primary | secondary | ghost | danger |
|---|---|---|---|---|
| Hover | accent brightened: `filter: brightness(1.08)` | `bg-hover`, `border-line-strong` | `bg-hover`, `text-primary` | brightness 1.08 |
| Active | brightness 0.95 | `bg-active` | `bg-active` | brightness 0.95 |
| Focus-visible | focus ring | focus ring | focus ring | focus ring (ring stays accent) |
| Disabled | opacity 0.45, `cursor-not-allowed`, no hover | same | same | same |
| Loading | `aria-busy="true"`, disabled interaction | same | same | same |

`IconButton`: square, same sizes, ghost by default, requires `label` prop (used for `aria-label` and tooltip).

### 5.7 Other primitives

- **`Kbd.svelte`**: `text-2xs` mono, `px-1`, height 16px, `border-line`, `rounded-sm`, `text-muted`, `bg-subtle`.
- **`Tooltip`** (action, `use:tooltip={'Logs · L'}`): 400ms delay, `bg-surface`, `shadow-popover`, `text-xs`, max-width 280px.
- **`Menu.svelte`** / **`Popover.svelte`**: `bg-surface`, `shadow-popover`, `rounded-lg`, `p-1`; items 28px, `rounded-md`, hover `bg-hover`; arrow keys, Enter, Esc, type-ahead; destructive items at the bottom after a separator, `danger-ghost` style. Closes on outside click and Esc. Replaces the ad-hoc dropdowns in Header and ThemeToggle.
- **`Combobox.svelte`**: `CustomSelect` with search. `CustomSelect` keeps its API (CLAUDE.md documents it) and gets `searchable?: boolean`; context/namespace switchers use it.
- **`ConfirmDialog.svelte`**: see 7.4.
- **`Toast`** store + `Toaster.svelte`: bottom-right above the status bar, `bg-surface`, `shadow-popover`, 4s, max 3 stacked, icon by health. Used for action results ("Restart triggered for checkout"), copy confirmations, density toggle.
- **`UsageBar.svelte`** (exists): keep API. Restyle: track `bg-hover` no border, 4px tall, `rounded-sm`, fill `bg-muted` under 75%, `bg-warning-solid` 75–89% with ▲ after the number, `bg-danger-solid` ≥ 90% with ✕ after the number. Neutral fill below threshold instead of accent: usage isn't "good", it's just usage. Null stays `–` in `text-faint` (keep current behavior, `UsageBar.svelte:27-28`).
- **`Tabs.svelte`**: replaces `TabFilter.svelte` and the three hand-written tab bars (`ResourceDetail.svelte:403-414`, `PodDetail.svelte:347+`, `DeploymentDetail.svelte:308-330`). Height 36px, `text-sm`, inactive `text-muted`, hover `text-primary`, active `text-primary` 500 with 2px `bg-accent` underline. Optional count and health badge per tab (`Events 5 ▲`). `role="tablist"`, arrow keys move, `1`–`9` jump.
- **`KeyValueList.svelte`**: label column 140px `text-muted text-sm`, value `text-primary text-sm` (mono when `mono`), row 28px, copy-on-hover icon. Replaces `InfoCard` grids where a list reads better.

Deleted after migration: `StatusBadge.svelte`, `Badge.svelte` (status usage), `ViewFilter.svelte`, `TabFilter.svelte`, `MetricCard.svelte`, `ThemeToggle.svelte` + `NotificationToggle.svelte` (merged into Settings popover).

---

## 6. Pulse (flagship)

Pulse is the product's highlight and the first thing every user sees. This section is the most detailed part of the spec on purpose. It covers the UI (6.1-6.8) and the backend that powers it (6.9-6.14).

### 6.1 Jobs to be done and the 5-second test

Pulse answers four questions, in this order of screen priority:

| # | Question | Block | Section |
|---|---|---|---|
| 1 | **What's broken right now?** | Needs Attention feed (+ health strip) | 6.3, 6.4 |
| 2 | **What changed?** | Change timeline, correlated with issue spikes | 6.5 |
| 3 | **Is it about to break?** | Capacity and trends | 6.6 |
| 4 | **Which of my clusters needs me?** | Fleet strip and All clusters view | 6.7 |

**5-second test** (acceptance for PR2): on a 1280×800 window, a user who opens the app on a cluster with problems can name the worst problem, the object it's on and how long it's been happening, without scrolling, clicking or reading more than the first feed item. Concretely:
- The first feed item is the highest-ranked issue, fully visible above the fold (top of the feed ≤ 220px from the window top).
- Each item's first line carries severity icon, root object, reason and duration.
- The feed renders from the engine's first snapshot. Target: **first issues on screen ≤ 3s after the app connects** on a 2,000-pod cluster (the engine's initial LISTs, 6.9). Before that, a skeleton, never an "All clear".

Principles:
- **Problems are listed, not counted.** Counts are secondary (health strip); the feed is primary.
- **One root cause, one item.** 12 crashlooping pods from one Deployment is one item, not 12.
- **Show evidence, suggest, never conclude.** "Started 2m after rollout to api:1.4.3", never "caused by".
- **When nothing is wrong, say so plainly** ("All clear"), not a wall of green.
- **Every source degrades on its own.** A forbidden or slow source makes the feed say "partial", never blank or wrong.

### 6.2 Layout

≥ 1280px wide (comfortable density):

```
┌ Pulse  [This cluster | All clusters 14]                         synced 2s ago · live  ⟳ ┐  page header
│ [⛨ prod-eu-1 PROD ✕3 ▲5] [⛨ prod-us-1 PROD ✓] [staging ▲2] [⊘ dev-kind unreachable] [+10 ›]│  fleet strip   (6.7)
│ ┌───────────┬────────────┬────────────┬────────────┬────────────┬─────────────┐           │
│ │ ✕ 3       │ ✕ 14       │ ✕ 1        │ ▲ 1        │ ✕ 1        │ ▲ 37        │           │  health strip  (6.4)
│ │ Workloads │ Pods       │ Nodes      │ Storage    │ Jobs       │ Warnings 1h │           │
│ └───────────┴────────────┴────────────┴────────────┴────────────┴─────────────┘           │
│ ┌ Needs attention  9 · 2 new          [All|Critical|Errors|Warnings] ┐ ┌ Capacity · 1h ─────┐│
│ │ ✕ CRIT  deploy/checkout   CrashLoopBackOff   payments   for 14m  ↗ │ │ CPU    62% ▁▂▃▄▅▆  ││  feed (6.3)
│ │         0/12 available · 12 pods · started 2m after rollout #42    │ │ Memory 81%▲▃▄▅▆▇▇  ││  capacity (6.6)
│ │ ✕ CRIT  node/ip-10-0-3-17 NotReady                      for 6m     │ │ Disk   71% ▅▅▆▆▆▆  ││
│ │         8 pods affected                                            │ │ Pods   412/1,320   ││
│ │ ✕ ERR   cronjob/nightly-report  BackoffLimitExceeded  batch  1h    │ │ ─────────────────  ││
│ │ ▲ WARN  pvc/data-pg-0  Pending · blocks 1 pod     db      for 12m  │ │ Under pressure     ││
│ │ ▲ WARN  BackOff ×42  pod/api-7f9c-x2 (no workload issue)   1m      │ │ ✕ ip-10-0-1-4 CPU94││
│ │ …                                                                  │ │ ▲ ip-10-0-2-9 Mem88││
│ │ ▸ Recently resolved 2 · Muted 1                                    │ └────────────────────┘│
│ └────────────────────────────────────────────────────────────────────┘                     │
│ ┌ Changes · last 1h ─────────────────────────────── [15m|1h] [lanes ▾] ┐                   │
│ │ issues ▁▁▁▁▁▁▁▁▁▁▁▂▂▅█████▇▇   (spike ┊ at 10:31)                       │                   │  timeline (6.5)
│ │ rollouts        ◆            ┊◆                                         │                   │
│ │ scaling      ▲      ▼        ┊                                          │                   │
│ │ restarts                 ↻   ┊ ↻↻↻↻                                     │                   │
│ │ nodes                        ┊   ●                                      │                   │
│ │ warnings ▁▁▂▁▁▁▁▁▁▁▁▁▃▅▇██▇▆                                            │                   │
│ │ 10:00      10:10      10:20      10:30      10:40      10:50   now      │                   │
│ └────────────────────────────────────────────────────────────────────────┘                   │
│ Inventory  Services 84 · Ingresses 12 · ConfigMaps 210 · Secrets 160 · PVCs 33 · …          │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

- Grid: 12 columns, gap 16px comfortable / 12px compact. Feed 8 columns, capacity 4, timeline 12.
- < 1100px: one column, in order: fleet strip, health strip, feed, timeline, capacity, inventory.
- Above the fold at 1280×800 (comfortable): header 44 + page header 48 + fleet strip 40 + health strip 64 → feed starts at ~200px and shows ~7 items (comfortable) / ~12 (compact).
- Panels: `bg-app`, `border-line`, `rounded-lg`, panel header 36px (title `text-sm` 600, count `text-muted`, controls on the right).
- Page header shows the namespace scope when a namespace is selected ("payments · 3 of 9 issues"). The engine always watches the whole cluster; the namespace filter is applied in the UI, so switching namespaces is instant.
- Focus order: page header → fleet strip → health strip → feed → capacity → timeline. When Pulse opens, the feed owns `j/k` (it's the active DataTable).

### 6.3 Needs Attention feed

#### Item anatomy

Collapsed (comfortable 52px, two lines; compact 28px, one line):

```
 ▌✕ CRIT  deploy/checkout  CrashLoopBackOff        payments  prod-eu-1   for 14m   ↗ New
 ▌        0/12 available · 12 pods · 38 restarts · started 2m after rollout #42 (api:1.4.3)
           [Logs L] [Events V] [Open ↵] [⋯]                                 ← on hover / selection
```

| Part | Content | Style |
|---|---|---|
| Rail | 2px left rail in the severity's solid color | decorative |
| Severity | icon (11.1) + `CRIT` / `ERR` / `WARN` tag | `text-2xs` 600; critical and error use failed colors, warning uses warning colors. The tag text is what tells critical from error (not color) |
| Root object | `kind/name` (kind short: `deploy, sts, ds, job, cronjob, pod, node, pvc, pdb, quota`) | name `text-primary` 500, kind `text-muted` mono |
| Reason | dominant reason (`CrashLoopBackOff`; `Multiple` with tooltip when mixed) | StatusPill `dot` variant |
| Namespace, context | namespace; context only in All-clusters mode | `text-muted` |
| Duration | "for 14m" since onset (6.10) | `text-muted` tabular |
| Trend | `↗` getting worse (impact grew in the last 5 min), `↘` recovering, nothing when stable; `New` tag when onset < 2 min | `text-2xs`; `New` uses `bg-active text-primary` |
| Line 2 (impact) | affected counts, restarts, the correlation hint (6.5) | `text-sm text-secondary`; hint in `text-muted` with a ◆ marker |

Compact density drops line 2 into a tooltip and shows impact as `0/12` after the reason.

#### Expanded item (`Space` / `→` / click the chevron)

```
 ✕ CRIT  deploy/checkout  CrashLoopBackOff                                      for 14m
   Members (12)                                   reason              restarts  node
     checkout-7f9c-x2                             CrashLoopBackOff    14        ip-10-0-1-4
     checkout-7f9c-kq                             CrashLoopBackOff    12        ip-10-0-2-9
     … 10 more                                                                   Show all ›
   Evidence
     ▲ BackOff ×42        Back-off restarting failed container api            1m
     ▲ Unhealthy ×18      Readiness probe failed: HTTP 503                    3m
     Last exit            api: Error (code 1) · 3s after start
   Possible trigger
     ◆ Rollout #41 → #42  image api:1.4.2 → api:1.4.3 · by argocd · 16m ago   View in timeline ›
   [Logs L] [Events V] [Open ↵] [Restart… ⇧R] [Scale… ⇧S] [Mute 1h M]
```

- Members are rendered with the DataTable row component (no header), max 10 then "Show all" (opens the workload's detail Pods tab).
- Evidence: up to 3 attached warning events (grouped by reason, `×count`, latest message) and the latest container termination for pod-rooted issues.
- Possible trigger: the top correlated change (6.11) when its score is ≥ 0.7; otherwise the block is hidden.

#### Grouping, ranking and sections

- Grouping and ranking come from the engine (6.10). The UI never regroups, it only filters.
- Feed order: rank from the engine. Sections inside the feed: active issues, then a collapsed **Recently resolved** group (issues resolved in the last 10 min, `text-faint`, ✓ icon, with "resolved 3m ago"), then a collapsed **Muted** group.
- Segments: `All | Critical | Errors | Warnings` with counts; `1`–`4` select them.
- Feed header shows **coverage**: when any source is degraded, a `▲ Partial` tag with tooltip "No access to jobs, events in this cluster; issues from them aren't shown." (from `coverage` in the snapshot).
- Cap: 200 items rendered; beyond that, "Showing 200 of 1,240 · narrow with a namespace or segment".

#### Live updates without losing your place

- Items are keyed by issue `fingerprint` (6.10); DataTable's selection anchoring keeps the cursor on the same issue across updates.
- When the user has a selection or has scrolled, new items don't push content: a pill "2 new issues ↑" appears at the top of the feed; clicking it (or `gg`) scrolls up. With no selection and at scroll top, new items insert in place.
- Resolved issues fade to the Recently resolved group on the next snapshot, never vanish under the cursor: a selected issue that resolves stays in place, rendered resolved, until the selection moves.

#### Quick actions

Updated principle (replaces Architecture Note 7 in CLAUDE.md in PR2): **non-destructive actions may be inline anywhere; restart and scale may be offered from Pulse only through ConfirmDialog; delete lives only in detail windows.** List views still get no restart/scale.

| Action | Key | Applies to | Behavior |
|---|---|---|---|
| Open | `Enter` | all | Opens the root object's detail window (pods-rooted: the pod; workload: the workload) |
| Logs | `L` | pod-rooted, workload-rooted | Opens the logs tab of the worst member pod (most restarts, then newest) |
| Events | `V` | all | Navigates to Events filtered to the issue's objects |
| Copy name | `Y` | all | Copies `kind/name`; toast |
| Copy command | `C` | all | Copies `kubectl -n ns describe kind/name --context ctx` |
| Expand / collapse | `Space`, `→` / `←` | all | |
| Restart… | `⇧R` | Deployment, StatefulSet, DaemonSet roots | ConfirmDialog tier 1 (prod: escalated, 7.4). Body names root object, namespace, context, and the issue ("12/12 pods in CrashLoopBackOff"). If a possible trigger is a rollout, the body adds: "This started 2m after rollout #42. A restart reruns the same image." |
| Scale… | `⇧S` | Deployment, StatefulSet roots | Scale dialog (number input, current → new, `↑/↓` to adjust) inside ConfirmDialog tier 1; prod escalated |
| Mute 1h | `M` | all | Local only, per fingerprint, persisted in settings; moves the item to Muted; it un-mutes automatically after 1h or if severity escalates |
| Delete | — | — | Not available from Pulse |

After a restart or scale: toast "Restart triggered for checkout on prod-eu-1", the item shows `◐ Restarting` for up to 2 min (until the next rollout completes), and the timeline gets a marker attributed to "Apex".

Row actions show on hover and on keyboard selection: Logs, Events, Open as ghost IconButtons with key tooltips; the rest in `⋯`.

#### States

| State | Render |
|---|---|
| Connecting / first snapshot pending | 6 skeleton items |
| All clear | `✓ Nothing needs attention in prod-eu-1` (`text-primary`), below it "Watching 2,140 pods, 186 workloads, 24 nodes" (`text-muted`), and "Last issue resolved 14m ago" when known. Neutral styling, no green panel |
| All clear but partial | The same, plus `▲ Partial: no access to jobs`. Never say "All clear" without the caveat when coverage is partial |
| Engine error (cluster unreachable) | EmptyState error with "Retry"; the fleet strip chip shows the same state |
| Namespace filter hides all | "No issues in payments · 9 in other namespaces" + "All namespaces" button |

#### Notifications

- System notification (existing notification setting and plugin, `kubernetes.ts:733-751`) for **new critical and error issues**, not muted, after the first snapshot (no burst at startup).
- One notification per fingerprint per 30 min; more than 3 within a minute collapse into one ("5 new issues in prod-eu-1").
- Clicking it focuses the main window, opens Pulse and selects the issue.
- The existing per-Warning-event notification is replaced by this (it was too noisy for triage); the setting label becomes "Notify on new critical issues".

### 6.4 Health strip

Six tiles in one panel, equal width, separated by `border-divider`:

| Tile | Number | Source |
|---|---|---|
| Workloads | issues rooted at Deployment / StatefulSet / DaemonSet | snapshot |
| Pods | affected pods across all issues | snapshot |
| Nodes | node issues (NotReady, pressure, network) | snapshot |
| Storage | PVC issues | snapshot |
| Jobs | Job / CronJob issues | snapshot |
| Warnings 1h | Warning events in the last hour | snapshot `counts.warning_events_1h` |

- Number `text-xl` 600 tabular in the worst severity's fg color with its icon; label `text-xs text-muted`.
- Zero shows `0` in `text-secondary` with the neutral circle (11.3 rule 3).
- Click / Enter: applies the tile as a feed filter (category chip in the feed header, removable). It doesn't navigate away; Pulse stays the triage surface. `⌘`-click opens the matching list view with its problem segment.
- Muted issues are excluded from the numbers.

### 6.5 Change timeline

Purpose: see what changed in the last hour and how it lines up with issue spikes.

#### Data shown

| Lane | Markers | Source (6.12) |
|---|---|---|
| Issues (band) | step area: errors+critical (failed solid) stacked with warnings (warning solid), 15s resolution | issue-count series |
| Rollouts | ◆ per rollout (Deployment revision change, StatefulSet / DaemonSet update revision change); hollow ◇ while in progress | change log |
| Restarts | ↻ for `kubectl rollout restart`-style restarts (restartedAt annotation); a small bar per minute for container restarts summed across pods | change log + restarts series |
| Scaling | ▲ scale up / ▼ scale down (replica changes); `HPA` tag when attributed to an autoscaler | change log |
| Nodes | ● node added, ○ removed, ✕ NotReady, ✓ Ready again, ⊘ cordoned, ⊙ uncordoned | change log |
| Warnings | per-minute histogram of Warning events | warning-event series |

Every marker has a shape **and** a lane, so meaning never depends on color. Marker color is neutral (`text-secondary`) except NotReady (failed solid) and changes made by Apex (accent ring).

#### Interaction

- Range: `15m | 1h` (default 1h). The left edge reads "History since 10:42 (app opened)" when coverage is shorter than the range, with backfilled markers (6.12) drawn hollow.
- Hover a marker: tooltip with what changed, when, by whom (`managedFields` manager, e.g. `kubectl-rollout`, `argocd-controller`, `helm`), and a link to the object.
- Click a marker: selects it, shows a details strip under the chart: the change (image diff `api:1.4.2 → api:1.4.3`, replicas `3 → 6`), and "Issues that started within 15 min after" (links that select the issue in the feed).
- **Spikes**: when grouped issue count rises by ≥ 3 within 2 min, or any new critical appears, a dashed vertical guide is drawn at that time. Changes in the 15 min before a spike whose correlation score with the spiking issues is ≥ 0.7 (6.11) get a highlighted ring and a "possible trigger" tooltip line.
- Selecting an issue in the feed highlights its onset on the timeline and its possible trigger; selecting a change highlights related issues in the feed. This two-way link is the point of the timeline.
- Keyboard: `T` focuses the timeline; `←/→` move between markers (all lanes, chronological), `↑/↓` change lane, `Enter` opens the marker's object, `Esc` returns focus to the feed.
- Lanes menu: toggle lanes; persisted.
- Namespace filter applies (node lane always shows).

#### Accessible alternative and rendering

- Below the chart, a collapsed "Changes (23)" list (DataTable: time, kind of change, object, detail, by). It's the accessible equivalent and is always in the DOM for screen readers (`aria-describedby` from the chart).
- Rendering: inline SVG, no chart library. Height 180px comfortable / 140px compact. Time axis labels every 10 min (`text-2xs text-muted`, tabular). Gaps in history (app suspended, engine disconnected) draw as a hatched `bg-hover` region labeled "no data", never interpolated.
- Live: new changes stream in (`pulse://change`, 6.13); the "now" edge advances every 15s.

### 6.6 Capacity and trends

```
┌ Capacity · last 1h ────────────────────────────── [15m|1h] ┐
│ CPU     62%     ▁▂▂▃▃▄▅▅▆▆   used 24.8 / 40 cores  ↑8 pts │
│                              requested 31.2 (78%)          │
│ Memory  81% ▲   ▃▃▄▄▅▅▆▆▇▇   used 130 / 160 Gi     ↑3 pts │
│                              requested 140 (88%)           │
│ Disk    71%     ▅▅▅▅▅▆▆▆▆▆   worst node · 3 nodes ≥ 75%    │
│                              ▲ ip-10-0-2-9 full in ~3h (estimate) │
│ Pods    412 / 1,320 slots (31%)                            │
├ Under pressure ───────────────────────────────────────────┤
│ ✕ ip-10-0-1-4   CPU 94% ▁▃▅▇█   Mem 71%   DiskPressure      │
│ ▲ ip-10-0-2-9   Mem 88% ▂▄▅▆▇   Disk 91%                    │
│   … 2 more                                    All nodes ›   │
└────────────────────────────────────────────────────────────┘
```

| Row | Now value | Sparkline | Secondary | Source |
|---|---|---|---|---|
| CPU | usage / allocatable (cluster) | 1h of usage % | requested / allocatable | metrics-server via sampler; requests from pod projections |
| Memory | same | same | same | same |
| Disk | **worst node** root-fs % (cluster average hides the node that's filling up) | 1h of worst-node % | count of nodes ≥ 75%; time-to-full estimate | kubelet summary via the existing disk sweep cache |
| Pods | running pods / pod capacity (sum of allocatable `pods`) | — | — | projections |

- Thresholds from 11.4: ≥ 75% warning, ≥ 90% failed, shown as icon after the number plus colored sparkline segments.
- Trend chip: change in percentage points over the last 15 min (`↑8 pts`), `text-muted`; hidden when |Δ| < 2.
- **Time-to-full** (disk only): linear fit over the last 30 min of that node's disk usage; shown only when the slope is positive, R² ≥ 0.8 and the projection is < 6h. Always labeled "(estimate)".
- **Under pressure**: nodes with a pressure condition, NotReady, or any usage ≥ 75%, sorted by worst state then highest usage, max 5. Each row: node name, worst metric with sparkline, other notable metrics, condition chips (`MemoryPressure`, `DiskPressure`, `PIDPressure` as neutral-bordered tags with the warning icon).
- Unavailable sources follow the node-metrics rule: no metrics-server → usage rows show `–` and "metrics-server not installed" (neutral), requests and pod slots still show; disk unreachable → `–` with the existing hint ("needs nodes/proxy access"). Never a confident 0%.
- Click a row: opens a capacity detail popover (440px) with a 1h line chart (usage solid, requests dashed, allocatable as the 100% line), axis labels, and the per-node breakdown table.

#### Sparkline spec (shared component `Sparkline.svelte`)

- Size: 120×24 comfortable, 96×18 compact. 1.5px line in `text-muted`; segments above 75% in `warning-solid`, above 90% in `danger-solid`. Dashed 1px `border-line` guide at 90%. Fixed y-domain 0-100% (so shapes are comparable between rows). Last point: 3px dot.
- Gaps: line breaks, no interpolation.
- `role="img"` with `aria-label` "CPU 62 percent, up 8 points in the last 15 minutes".
- Hover: crosshair + tooltip (value, time). No animation.

### 6.7 Multi-cluster overview

#### Fleet strip (top of Pulse)

```
[⛨ prod-eu-1 PROD ✕3 ▲5] [⛨ prod-us-1 PROD ✓] [staging ▲2] [◐ qa connecting] [⊘ dev-kind unreachable] [⚿ eks-old auth] [+10 ›]
```

- One chip per context in the fleet (6.14), height 28px, `rounded-md`, `border-line`; the current context's chip is `bg-active` with the accent rail at the bottom (2px).
- Content: PROD shield + tag (4.6), name (truncate at 18 chars), then either the worst-state icon with counts (`✕3 ▲5` = critical+error count, warning count) or the reachability state: `◐ connecting`, `⊘ unreachable`, `⚿ auth` (needs re-authentication), `– excluded`. A stale entry (last poll failed once, 6.14) shows its last counts in `text-faint` with a clock icon.
- Order: current context first, then worst state, then name. Overflow collapses into `+N ›`, which opens All clusters.
- Click / Enter on a chip: **switches the app to that context** (existing `switch_context`, which also writes the kubeconfig `current-context`; the chip tooltip says so: "Switch to staging (also sets kubectl's current context)"). Switching is navigation, so no confirm; the prod toast from 4.6 applies.
- Keys: `⌥1`–`⌥9` switch to the nth chip.

#### All clusters view (`This cluster | All clusters` segmented control; `⌘⇧F`)

```
CONTEXT          CLUSTER        STATE         ISSUES        NODES    CPU   MEM   VERSION  POLLED
⛨ prod-eu-1 PROD eks-prod-eu    ✕ 3 critical  ✕3 ▲5         23/24    62%   81%▲  1.30.4   live
⛨ prod-us-1 PROD eks-prod-us    ✓ healthy     0             18/18    41%   55%   1.30.4   12s ago
staging          gke-staging    ▲ degraded    ▲2            6/6      22%   48%   1.31.1   40s ago
dev-kind         kind-dev       ⊘ unreachable —             —        —     —     1.32.0   5m ago · retry in 2m
eks-old          eks-legacy     ⚿ auth needed —             —        —     —     —        aws: token expired
```

- Built on DataTable; default sort worst first. Row expand (`→`) shows the context's top 3 issues (fleet-level, 6.14). Enter: switch to that context and show its Pulse.
- Row `⋯`: Refresh now, Exclude from fleet, Mark/Unmark production, Copy context name.
- Issue counts in fleet rows are **workload-level** for non-current contexts (6.14) and the column header tooltip says so. The current context shows full engine counts.
- Footer: "Polling 14 of 16 contexts every 60s · 2 excluded · Settings ›".

### 6.8 Pulse keyboard map

| Key | Action |
|---|---|
| `j` / `k`, `gg` / `G` | Move in the feed (or the fleet table in All clusters) |
| `Enter` | Open root object (feed) / switch context (fleet) |
| `Space`, `→` / `←` | Expand / collapse item |
| `1`–`4` | Feed segment |
| `L`, `V`, `Y`, `C` | Logs, events, copy name, copy kubectl command |
| `⇧R`, `⇧S` | Restart…, Scale… (ConfirmDialog) |
| `M` | Mute 1h / unmute |
| `T` | Focus timeline (`←/→` markers, `Esc` back) |
| `⌘⇧F` | Toggle This cluster / All clusters |
| `⌥1`–`⌥9` | Switch to fleet chip n |
| `r` | Refresh now (asks the engine for an immediate re-list) |

All keys are listed in KeyboardHelp under a "Pulse" group. Lowercase `l`, `y`, `e`, `r` keep their list-view meaning elsewhere; on Pulse the uppercase forms are shown in tooltips but both cases work for `L`, `V`, `Y`, `C`, `M`, `T`.

---

### 6.9 Backend: the Pulse engine (current context)

A new Rust module tree `src-tauri/src/pulse/` owns everything Pulse shows. It runs **whenever the app is connected**, not only while the Pulse view is open, because the sidebar badges, notifications and history depend on it.

```
src-tauri/src/
├── triage/            # pure rules (shared spec, 6.10); no I/O
│   ├── rules.rs       # per-object verdicts and findings (TRI-* ids)
│   ├── issues.rs      # grouping, root-cause resolution, ranking, fingerprints
│   └── fixtures.rs    # test runner over spec/triage/fixtures/**
└── pulse/
    ├── engine.rs      # per-context lifecycle, watchers, projections, snapshot emit
    ├── projections.rs # slim structs built from watch events
    ├── tracker.rs     # issue onset/resolution tracking, trend
    ├── changes.rs     # change detection + backfill (6.12)
    ├── correlate.rs   # issue ↔ change scoring (6.11)
    ├── history.rs     # ring buffers + HistoryProvider seam (6.12)
    ├── sampler.rs     # 15s metric sampling (6.12)
    └── fleet.rs       # multi-context poller (6.14)
```

**Sources** (cluster-wide, independent watchers using `kube::runtime::watcher`, already available through the `runtime` feature):

| Source | Mode | Projection fields kept |
|---|---|---|
| Pods | watch | uid, ns, name, controller owner ref, phase, conditions (Ready, PodScheduled: status, reason, lastTransitionTime), containers and init containers (name, ready, restartCount, waiting.reason, lastState.terminated: reason, exitCode, finishedAt), nodeName, creationTimestamp, PVC claim names, resource requests (cpu, memory) |
| ReplicaSets | watch | uid, ns, name, owner (Deployment), revision annotation, template images, replicas, creationTimestamp |
| Deployments, StatefulSets, DaemonSets | watch | spec replicas, status counts, conditions (Available, Progressing with reason), revision / updateRevision, restartedAt annotation, template images, latest `managedFields` manager for spec updates |
| Jobs, CronJobs | watch | conditions (Complete, Failed with reason), active/succeeded/failed, owner CronJob, start/completion time; CronJob last schedule, suspend |
| Nodes | watch | conditions (Ready, MemoryPressure, DiskPressure, PIDPressure, NetworkUnavailable), unschedulable, taints, allocatable (cpu, memory, pods), creationTimestamp |
| PVCs | watch | phase, storageClass, creationTimestamp |
| Events | watch (reuse the global event watch's stream; one watcher, two consumers) | type, reason, involvedObject (kind, ns, name, uid), count, last_seen, message (truncated to 300 chars) |
| PDBs, ResourceQuotas | poll 60s (added in PR8) | as in 12.1 |
| Node usage, disk | sampler (6.12) | — |

- **Projections, not full objects**: each watch event is converted into a slim struct and the full object is dropped. Budget: ~300 bytes per pod, so a 10,000-pod cluster costs ~3 MB, not the ~100 MB a full reflector store would.
- **Startup**: all watchers start concurrently; the first snapshot is emitted when pods, workloads and nodes have completed their initial list, or after 3s with whatever is ready (coverage marks the rest `loading`). This is what makes the 3-second target in 6.1 achievable.
- **Evaluation cadence**: re-evaluate issues on any projection change, debounced to at most once per 2s, plus a 15s tick for time-based rules ("Pending > 5 min"). Emit `pulse://snapshot` only when the snapshot changed (hash of issue fingerprints + severities + impact).
- **Per-source degradation** (same philosophy as node metrics in CLAUDE.md): every source has its own state `loading | ok | forbidden | error`. 403 on a cluster-wide watch falls back to watching the selected namespace only, and coverage says "namespace payments only". A failing source never blocks the others or the snapshot. Watch errors reconnect with backoff 1s → 30s (kube watcher default backoff).
- **Context switch**: `switch_context` stops the old engine (drops watchers) and starts a new one. Every emitted payload carries `context`; the frontend drops payloads for a context that isn't current (same guard as `loadNodeMetrics`). The last 3 contexts' **history** (not watchers) is kept (LRU), so switching back shows the earlier hour.
- **Relationship to list-view watches** (`watch.rs`): unchanged in this phase; the Pods view may run its own pod watch alongside the engine's. A later PR can serve list views from engine projections.

### 6.10 Issue detection rules (shared with mobile)

The rules below are the **content of `spec/triage/rules.md`**, the shared triage spec that `docs/mobile/architecture.md` (section 3) defines for desktop, iOS and Android. Rule ids, verdicts and fixtures are shared; the desktop Rust `triage` module is the reference implementation and runs `spec/triage/fixtures/**` in `cargo test` (mobile plan's PR "2b. Desktop fixture runner"). Whichever track lands first creates the files; the other adopts them.

**Verdict model** (mobile section 3, extended): `status: ok | warn | fail | unknown`, `reason`, `ruleId`, `since?`, plus two optional fields this spec adds:
- `severity: critical | error | warning`, present when status is `fail` (critical or error) or `warn` (warning).
- `tone: progressing | neutral`, only with status `ok`, so the six display states of section 11 can be derived on every platform (11.1).
The schema change goes into `spec/triage/schema/verdict.schema.json` with the first desktop PR that touches it.

**Per-object rules** (findings):

| Id | Object | Condition | Status / severity | Reason label | since |
|---|---|---|---|---|---|
| TRI-POD-001 | Pod | phase `Failed`, not owned by a Job, reason ≠ `Evicted` | fail / error | `Failed` or pod reason | status.startTime or last transition |
| TRI-POD-002 | Pod | phase `Unknown` | fail / error | `Unknown` | Ready lastTransitionTime |
| TRI-POD-003 | Pod | any container waiting `CrashLoopBackOff` | fail / error | `CrashLoopBackOff` | earliest lastState.terminated.finishedAt in the current backoff, else Ready transition |
| TRI-POD-004 | Pod | waiting `ImagePullBackOff`, `ErrImagePull`, `InvalidImageName` | fail / error | the waiting reason | Ready transition |
| TRI-POD-005 | Pod | waiting `CreateContainerConfigError`, `CreateContainerError`, `RunContainerError` | fail / error | the waiting reason | Ready transition |
| TRI-POD-006 | Pod | last termination `OOMKilled` within 1h | fail / error if also TRI-POD-003, else warn / warning | `OOMKilled` | finishedAt |
| TRI-POD-007 | Pod | phase `Running`, not all containers ready for > 5 min | warn / warning | `n/m ready` | Ready transition |
| TRI-POD-008 | Pod | phase `Pending` > 5 min; `PodScheduled=False, Unschedulable` | warn / warning; **fail / error** when unschedulable | `Pending` / `Unschedulable` | creationTimestamp |
| TRI-POD-009 | Pod | ≥ 3 restarts in the last 1h (sampler deltas; on cold start: terminations with finishedAt in the last 1h) | warn / warning | `n restarts in 1h` | first of those restarts |
| TRI-POD-010 | Pod | phase `Failed`, reason `Evicted` | warn / warning | `Evicted` | status.startTime |
| TRI-POD-011 | Pod | `Pending` ≤ 5 min, or ContainerCreating / PodInitializing | ok / tone progressing | the state | — |
| TRI-WL-001 | Deployment, StatefulSet | desired > 0 and available (ready for StatefulSet) == 0 for > 1 min | fail / **critical** | `Unavailable` | Available condition transition |
| TRI-WL-002 | Deployment, StatefulSet | 0 < available < desired for > 3 min | warn / warning; fail / error when available ≤ 50% | `n/m available` | condition transition |
| TRI-WL-003 | Deployment | `Progressing=False`, reason `ProgressDeadlineExceeded` | fail / error | `Rollout stuck` | condition transition |
| TRI-WL-004 | DaemonSet | numberUnavailable > 0 for > 3 min; numberMisscheduled > 0 | warn / warning; fail / critical when numberAvailable == 0 and desired > 0 | `n/m available` / `Misscheduled` | observed |
| TRI-WL-005 | Any workload | updating (updated < desired) ≤ progress deadline | ok / tone progressing | `Rolling out` | — |
| TRI-JOB-001 | Job | `Failed=True` (BackoffLimitExceeded, DeadlineExceeded, …) and (completed within 1h, or it's the latest Job of its CronJob) | fail / error | condition reason | condition transition |
| TRI-JOB-002 | CronJob | its last ≥ 2 Jobs failed | fail / error (groups TRI-JOB-001 members) | `Last n runs failed` | first failure |
| TRI-NODE-001 | Node | `Ready` ≠ True | fail / **critical** | `NotReady` / `Unknown` | Ready transition |
| TRI-NODE-002 | Node | `MemoryPressure`, `DiskPressure` or `PIDPressure` True | fail / error | the condition | transition |
| TRI-NODE-003 | Node | `NetworkUnavailable` True | fail / critical | `NetworkUnavailable` | transition |
| TRI-NODE-004 | Node | unschedulable (cordoned) | warn / warning | `Cordoned` | — (no timestamp in the API) |
| TRI-NODE-005 | Node | CPU or memory ≥ 90% of allocatable for ≥ 5 min (sampler) | warn / warning | `CPU 94%` | first sample ≥ 90% |
| TRI-NODE-006 | Node | root fs ≥ 90% (warning), ≥ 95% (error) | warn / warning · fail / error | `Disk 92%` | first sample |
| TRI-PVC-001 | PVC | `Pending` > 2 min | warn / warning; fail / error when a pod is waiting on it | `Pending` | creationTimestamp |
| TRI-PVC-002 | PVC | `Lost` | fail / error | `Lost` | observed |
| TRI-EVT-001 | Event | Warning, last_seen within 1h, deduped by involvedObject uid + reason | evidence; standalone warn / warning only per the grouping rules | event reason `×count` | first_seen |
| TRI-PDB-001/002, TRI-QUOTA-001 | PDB, ResourceQuota | as 11.5 (desktop-only, PR8) | as 11.5 | | |

High-signal event reasons (standalone even with count < 3): `FailedScheduling, FailedMount, FailedAttachVolume, FailedCreatePodSandBox, FailedCreate, Evicted, OOMKilling, NodeNotReady, Rebooted, FreeDiskSpaceFailed, ImageGCFailed`.

**Grouping into issues** (`issues.rs`, content of `spec/triage/issues.md`, with fixtures under `spec/triage/fixtures/issues/`):

1. **Root resolution** for every pod finding: pod → controller owner. ReplicaSet → its Deployment (or the ReplicaSet if bare). Job → its CronJob (or the Job). StatefulSet, DaemonSet direct. No controller → the pod itself.
2. **Node absorption**: a pod finding with reason `Unknown`, NotReady or Terminating on a node that has TRI-NODE-001 is attached to the node issue as an affected pod and dropped as its own finding.
3. **Storage absorption**: a Pending pod whose claims include a Pending/Lost PVC is attached to that PVC's issue ("blocks 1 pod").
4. **Workload merge**: all findings with the same root become one issue. Workload-level findings (TRI-WL-*) and member pod findings combine. Reason label = the dominant member reason (most members; ties by severity), or `Multiple` when no reason covers ≥ 60% of members, with the breakdown in the tooltip (`CrashLoopBackOff ×8, OOMKilled ×4`).
5. **Severity** = max member severity, escalated to critical when the root workload has TRI-WL-001 (zero available).
6. **Evidence**: Warning events whose involvedObject is the root or any member are attached (deduped by reason, top 3 by count). Remaining Warning events become standalone issues only if count ≥ 3 in 1h or the reason is high-signal; they are grouped by (reason, root of the involved object).
7. **Fingerprint** = `{ruleFamily}/{rootKind}/{rootNamespace}/{rootName}/{reasonClass}` where reasonClass is the dominant reason bucket (crash, image, config, oom, ready, pending, unavailable, node, pressure, storage, job, events). Context is added by the transport, not the fingerprint. Stable across polls; used for onset, mute, selection, notifications.
8. **Rank** (descending priority): severity (critical > error > warning) → impact score → onset (newer first) → name. Impact score = affected pods + 10 × affected nodes + 5 if the root has TRI-WL-001. Ties stay stable.

**Issue tracker** (`tracker.rs`): keeps `{fingerprint → onset, last_seen, peak_impact, impact_5m_ago}` in memory (and in history persistence, 6.12). `onset` = earliest member `since`, or first seen by the engine when no timestamp exists. Trend: impact now vs 5 min ago (`↗` if +20% or +2, `↘` if -20%). An issue absent for two consecutive evaluations (≥ 4s apart) is resolved; it re-opens with the same onset if it returns within 10 min (no flapping onsets).

### 6.11 Correlation (issues ↔ changes)

`correlate.rs` scores each change `c` in `[onset - 15 min, onset + 1 min]` against an issue `i`:

| Relation of `c.target` to `i` | Base score |
|---|---|
| Same object as the issue root | 1.0 |
| In the root's owner chain (e.g. ReplicaSet of the Deployment) | 0.9 |
| Node that hosts ≥ 50% of the members | 0.7 |
| Same namespace | 0.3 |
| Other | 0 |

Time decay: × (1 - 0.5 × minutes_before_onset / 15). The top change with score ≥ 0.7 becomes the issue's "possible trigger". Wording rule: always "started N min after <change>", never "caused by". Scores are computed in the engine and shipped in the snapshot, so mobile and desktop can show the same hint later.

### 6.12 History: ring buffer, sampling, change log, persistence

**Sampler** (`sampler.rs`), every 15s for the current context:
- Node CPU and memory from metrics-server (the same code path as `get_node_metrics`, so its caching, concurrency and "null, never 0" rules apply).
- Disk: reads the existing per-cluster disk cache from the kubelet sweep (60s cache, back-off rules in CLAUDE.md). The sampler **never triggers its own sweep**; it records whatever the cache holds, so the sweep's invariants stay intact.
- Requests: sums of pod projection requests (cluster and per node).
- Issue counts by severity, Warning events per minute, container restarts per minute (restartCount deltas from projections).

**Ring buffers** (`history.rs`): fixed-capacity, in memory, one set per context.

| Series | Resolution | Capacity (1h) | Notes |
|---|---|---|---|
| cluster cpu/mem usage %, requests %, worst-node disk % | 15s | 240 points | `f32` + timestamp |
| per-node cpu/mem/disk % | 15s | 240 points × nodes | capped at 200 nodes; beyond that, keep the 50 highest-usage nodes plus aggregates |
| issue counts (critical, error, warning) | 15s | 240 | |
| warning events/min, restarts/min | 1 min | 60 | |
| change log | event | 2,000 entries or 1h, whichever first | `ChangeEvent` (below) |
| issue tracker records | event | 1,000 | onset/resolved |

Memory budget ≤ 8 MB per context; at most 3 contexts kept (LRU). Gaps are explicit: a missing tick writes a `gap` marker, which the UI draws as "no data".

**Change log** (`changes.rs`): derived from projection diffs and events.

```rust
struct ChangeEvent {
    at: DateTime<Utc>,
    kind: ChangeKind,       // Rollout, RolloutComplete, Restart, Scale, NodeAdded, NodeRemoved,
                            // NodeNotReady, NodeReady, Cordon, Uncordon, JobFailed
    target: ObjectRef,      // kind, ns, name, uid
    detail: ChangeDetail,   // revisions, image diff, replicas from→to, reason
    actor: Option<String>,  // managedFields manager; "HPA" when matched to SuccessfulRescale; "Apex" for in-app actions
    source: Source,         // Live | Backfill
}
```

Detection rules: Deployment revision annotation increments → `Rollout` (image diff from the old and new ReplicaSet templates); `kubectl.kubernetes.io/restartedAt` annotation changes → `Restart`; `spec.replicas` changes → `Scale` (actor HPA if a `SuccessfulRescale` event for the same object arrives within 30s); StatefulSet/DaemonSet `updateRevision` changes → `Rollout`; node watch add/delete, Ready transitions, `unschedulable` toggles → node kinds; TRI-JOB-001 onset → `JobFailed`.

**Backfill on engine start**, so the first hour after launch isn't empty: ReplicaSets with creationTimestamp in the last hour → `Rollout` entries (by revision); events still held by the API server (default TTL 1h) with reasons `ScalingReplicaSet`, `SuccessfulRescale`, `NodeNotReady`, `NodeReady`, `NodeSchedulable`, `NodeNotSchedulable`, `RegisteredNode`, `RemovingNode` → matching entries. Backfilled entries have `source: Backfill` and render hollow. Metric series can't be backfilled locally; their coverage starts at app open.

**Optional persistence** (Settings → Pulse → "Keep history between launches", default **off**):
- Every 5 min and on quit, write `{app_data_dir}/pulse-history/{sanitized-context-name}.json` with series, change log and tracker records (serde_json, already a dependency; no new crate). On engine start, load it, drop anything older than 1h, and mark the gap between quit and launch.
- Stored data: metric numbers, object kinds/names/namespaces, event reasons and truncated messages. No secrets, no object specs. The setting's description says so.
- "Clear Pulse history" button next to the toggle.

**Seam for Prometheus later**:

```rust
#[async_trait]
trait HistoryProvider: Send + Sync {
    fn coverage(&self, context: &str) -> Option<DateTime<Utc>>;            // earliest available point
    async fn series(&self, context: &str, s: SeriesId, from: DateTime<Utc>, to: DateTime<Utc>, step: Duration)
        -> Result<Vec<Point>>;
    async fn changes(&self, context: &str, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<ChangeEvent>>;
}
```

v1 ships `LocalHistory` (ring buffers). A future `PrometheusHistory` (configured per context: URL, auth) implements the same trait for longer ranges; Pulse commands pick a provider per context and return `source: "local" | "prometheus"` plus `coverage_start`, which the UI already renders ("History since 10:42"). `async_trait` isn't required if the trait uses boxed futures; the engineer picks whichever matches the codebase (no new heavy dependency either way).

### 6.13 Tauri commands and events

Commands (`commands.rs`, registered in `lib.rs`):

| Command | Parameters | Returns |
|---|---|---|
| `get_pulse_snapshot` | `context_name` | `PulseSnapshot { context, generated_at, issues: Vec<Issue>, counts: PulseCounts, coverage: Vec<SourceState>, inventory: InventoryCounts }` (latest; also pushed by event) |
| `get_pulse_issue` | `context_name, fingerprint` | `IssueDetail { issue, members: Vec<MemberRow>, evidence: Vec<EventSummary>, last_termination, triggers: Vec<ScoredChange> }` |
| `pulse_refresh` | `context_name` | `()`: forces re-list of all sources (manual refresh, `r`) |
| `get_pulse_timeline` | `context_name, from, to` | `Timeline { source, coverage_start, changes: Vec<ChangeEvent>, series: { issues, warnings_per_min, restarts_per_min }, gaps, spikes: Vec<Spike> }` |
| `get_pulse_capacity` | `context_name, from, to` | `Capacity { source, coverage_start, cluster: { cpu, memory, disk_worst, pods }, nodes: Vec<NodeTrend> }`; each metric `{ now: Option<f32>, allocatable, requested, series }` |
| `get_fleet_overview` | — | `Vec<FleetEntry { context, cluster, is_current, state, counts, top_issues, nodes_ready, nodes_total, cpu_pct, mem_pct, version, last_ok_at, next_poll_at, error }>` |
| `set_fleet_config` | `FleetConfig { enabled, interval_secs, excluded: Vec<String> }` | `()` |
| `fleet_refresh` | `context_name` | `()`: poll one context now, resets its backoff |
| `set_pulse_persistence` | `enabled: bool` | `()` |
| `clear_pulse_history` | `context_name?` | `()` |

Existing `restart_deployment`, `scale_deployment`, `restart_statefulset`, `scale_statefulset` are reused for quick actions (plus `restart_daemonset`, new, same pattern). Each records a `ChangeEvent` with actor `Apex`.

Events (`app.emit`), every payload carries `context`:

| Event | When | Payload |
|---|---|---|
| `pulse://snapshot` | snapshot changed (≤ 1 per 2s) | `PulseSnapshot` |
| `pulse://change` | a change is detected | `ChangeEvent` |
| `pulse://sample` | each 15s sample | latest points for the visible series (so charts don't re-fetch) |
| `pulse://fleet` | after each fleet cycle | `Vec<FleetEntry>` |

Frontend store (`src/lib/stores/pulse.ts`, new): `pulseSnapshot`, `pulseTimeline`, `pulseCapacity`, `fleet`; subscribes once in `App.svelte`; applies the context guard to every event; derives sidebar badge counts (4.2) and status-bar freshness from `pulseSnapshot`. `get_pulse_metrics` stays for inventory compatibility until PR11 removes it.

### 6.14 Multi-context poller (fleet)

`fleet.rs` gives a light health summary for **every other** kubeconfig context. The current context's fleet entry comes from the full engine.

- **Contexts**: read from kubeconfig at start and whenever the file changes (re-read every 60s, compare mtime). Default: all contexts included, up to 30; beyond 30, new ones need an explicit include. Exclusions live in Settings → Fleet and the fleet row menu.
- **Per-context work** (one cycle): `GET /version` (cached 10 min), then concurrently: nodes, Deployments, StatefulSets, DaemonSets, Jobs (LIST with `resourceVersion=0`, served from the API server's watch cache, `limit=500`, at most 3 pages), and metrics-server node metrics. **No pod LIST** in fleet mode, which keeps the cost bounded on big clusters; fleet issues are therefore workload-level (TRI-WL-*, TRI-JOB-*, TRI-NODE-*) and labeled as such.
- **Concurrency and timeouts**: a semaphore of **4 contexts in flight**; inside a context, its calls run concurrently. Per call 4s, whole context cycle **8s hard timeout** (`tokio::time::timeout`). Interval 60s (Settings: 30s / 60s / 5 min), with ±10% jitter so contexts don't sync up. The fleet runs on its own tokio tasks; **the current-context engine never waits on it**.
- **Clients**: one cached `kube::Client` per context, rebuilt on kubeconfig change or auth error. Exec credential plugins run **non-interactively** (`KUBERNETES_EXEC_INFO` with `interactive: false`, no stdin), so polling 20 contexts can never pop SSO browser windows; a plugin that needs interaction yields state `auth`.
- **Degradation per context** (mirrors the node-metrics rules: whatever answers in time is used, one failure never condemns):

| Outcome | State | Behavior |
|---|---|---|
| All calls OK | `ok` | counts updated |
| Some calls 403 / timed out | `degraded` | partial counts; missing sources listed in the tooltip |
| Whole cycle failed **once** (timeout, connection refused, DNS) | previous state + `stale` | last counts kept, shown faint with a clock |
| Failed **twice in a row** | `unreachable` | backoff 1m → 2m → 5m → 10m (cap); "retry in 2m" shown |
| 401, or exec plugin error | `auth` | backoff 10 min immediately (the credential won't fix itself); hint shows the plugin command (`aws`, `gke-gcloud-auth-plugin`, `kubelogin`) |
| 403 on everything | `forbidden` | backoff 10 min |
| Excluded | `excluded` | not polled |

  Manual "Refresh now" on a row resets its backoff. A successful cycle resets everything.
- **Budget**: with 16 contexts, 4 in flight and an 8s cap, a full sweep takes ≤ 32s worst case inside a 60s interval. Fleet results are kept only as the latest summary per context (no fleet history in v1).
- **Switching** to a fleet context hands it to the full engine; its fleet task pauses while it's current.

---

## 7. Detail windows

### 7.1 Layout

```
┌ 44px titlebar (drag region, bg-subtle) ──────────────────────────────────────────────────────┐
│ [pod] api-7f9c-x2  ✕ CrashLoopBackOff      prod-eu-1 / payments  🔒      [Logs L][Exec E][⋯]  ●○○ │
├ Tabs 36px ───────────────────────────────────────────────────────────────────────────────────┤
│ Overview   Containers 2 ✕   Logs   Events 5 ▲   YAML                                          │
├──────────────────────────────────────────────────────────────────────────────────────────────┤
│ ┌ Problem banner (only when health is failed/warning) ─────────────────────────────────────┐ │
│ │ ✕ Container "api" is in CrashLoopBackOff · 14 restarts · last exit: Error (code 1)  [View logs] │
│ └──────────────────────────────────────────────────────────────────────────────────────────┘ │
│ ┌ Main column (flex 1) ───────────────────────────────┐ ┌ Meta rail 300px ─────────────────┐ │
│ │ Conditions (table)                                   │ │ Namespace   payments              │ │
│ │ Containers summary (name, image, state pill, restarts)│ │ Node        ip-10-0-1-4 ›         │ │
│ │ Recent events (5, link to Events tab)                │ │ Pod IP      10.1.4.22   ⧉         │ │
│ │                                                      │ │ Owner       ReplicaSet api-7f9c › │ │
│ │                                                      │ │ Age         3m                    │ │
│ │                                                      │ │ QoS         Burstable             │ │
│ │                                                      │ │ ───────────────────────────────── │ │
│ │                                                      │ │ Labels (chips)                    │ │
│ │                                                      │ │ Annotations (collapsed, count)    │ │
│ └──────────────────────────────────────────────────────┘ └───────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
```

- Titlebar replaces the current header (`DetailWindow.svelte:~95-134`): kind tag (`text-2xs` mono, `bg-active`, rounded-sm), name `text-md` 600 truncating, **StatusPill with resource health** (new; computed from detail data with the same `health.ts` rules), then context / namespace `text-xs text-muted`, `PROD` tag and window edge line when the locked context is production (4.6), lock icon with tooltip "Pinned to prod-eu-1. Doesn't follow context switches." (replaces the "Locked View" pill at `DetailWindow.svelte:127-129`).
- Below 900px window width, the meta rail moves under the main column.
- Overview sections: `text-md` 600 heading, `--section-gap` between sections. No card-in-card. Number groups (replicas etc., `ResourceDetail.svelte:497-513`) become one inline stat row: `Desired 3 · Ready 1 ✕ · Available 1 · Up-to-date 3` in `text-sm`, problem numbers in state color with icon.
- Problem banner: `bg-{state}-tint`, 1px `border-{state}-solid/30`, `rounded-lg`, `text-sm`, icon + one-sentence cause + one action. Derived from health `reason` + container `last_state` / conditions. Hidden when healthy.
- Deleted banner (`ResourceDetail.svelte:416-432`) keeps its behavior, restyled as neutral banner (`bg-active`, `text-primary`, "Deleted 2m ago · read-only"), since the resource being gone isn't an error in the window.

### 7.2 Tabs per type

Keep the config-driven approach (`ResourceDetail.svelte:20-38`). Changes:
- Pod: `Overview, Containers, Logs, Events, YAML` (unchanged, now via shared `Tabs`).
- Events tab shows count and ▲ when any Warning.
- Containers tab badge ✕ when any container is failed.
- `[`/`]` previous/next tab, `1`–`9` jump, remembered per resource type in localStorage.

### 7.3 Action placement

Rule: **safe actions visible, destructive actions one step away, always confirmed.**

| Resource | Visible buttons (secondary, `md`) | `⋯` More menu | Danger (in menu, bottom, after separator) |
|---|---|---|---|
| Pod | Logs `L`, Exec `E` | Port forward…, Copy name, Copy `kubectl` command | Delete pod… |
| Deployment | Scale… `S`, Restart… `R` | Copy name, View ReplicaSets | (Delete when implemented) |
| StatefulSet | Scale… `S`, Restart… `R` | Copy name | (Delete when implemented) |
| Node | Cordon / Uncordon… | Add taint…, Copy name | (Drain… when implemented) |
| Secret | Reveal values (toggle) | Edit…, Copy name | Delete secret… |
| Others | — | Copy name, Copy YAML | (Delete when implemented) |

The `…` in a label means "opens a dialog". Buttons disabled (with tooltip "Resource was deleted") when `isDeleted`.

### 7.4 Confirmations: `ConfirmDialog.svelte`

Replaces every native `confirm()` (audit 9) and the hand-rolled modals in PodList/DeploymentDetail.

```
┌ Restart deployment ─────────────────────────────── ✕ ┐
│ checkout · payments · prod-eu-1                       │
│                                                       │
│ Pods are replaced one by one (rolling restart).       │
│ 3 pods will restart. Traffic should keep flowing if   │
│ readiness probes are set.                             │
│                                                       │
│                         [Cancel  Esc] [Restart  ↵]    │
└───────────────────────────────────────────────────────┘
```

Props: `title`, `resource: {kind, name, namespace?, context}` (always shown, so identity is confirmed), `body`, `confirmLabel`, `tone: 'default' | 'danger'`, `requireTyping?: string`, `onConfirm: () => Promise<void>`.

| Tier | Used for | Behavior |
|---|---|---|
| 1, default | Restart, scale, cordon/uncordon, add taint (non-NoExecute) | Primary confirm button. Enter confirms, Esc cancels. |
| 2, danger | Delete pod, delete secret, remove NoExecute taint, uncordon with evictions | Danger button. **Focus starts on Cancel**, Enter does not confirm until focus moves to the danger button (Tab) or ⌘Enter is used. |
| 3, danger + type | Delete namespace, delete PV, drain node (future) | Danger button disabled until user types the resource name. |

**On a production context** (4.6), the dialog is reinforced:

```
┌ Restart deployment ─────────────────────────────────── ✕ ┐
│ ┌ ⛨ PROD  Production context: prod-eu-1 ──────────────┐ │
│ └─────────────────────────────────────────────────────┘ │
│ checkout · payments · prod-eu-1                          │
│ Pods are replaced one by one (rolling restart).          │
│                                                          │
│ [Cancel  Esc]  [Restart on prod-eu-1  ⌘↵]                 │
└──────────────────────────────────────────────────────────┘
```

| Normal tier | On prod it behaves as | Concretely |
|---|---|---|
| 1 (restart, scale, cordon) | tier 2 | Focus starts on Cancel; Enter alone doesn't confirm; ⌘↵ or Tab + Enter does. Confirm button stays the default style (it's not a delete), labeled "Restart on prod-eu-1". |
| 2 (delete pod, delete secret, NoExecute taint) | tier 3 | Danger button disabled until the resource name is typed. |
| 3 (delete namespace / PV, drain) | tier 3+ | Must type `prod-eu-1/<name>`. |

- The banner is the first child of the dialog: `bg-warning-tint`, 1px `border-warning-solid/30`, `rounded-lg`, shield icon, `PROD` tag, "Production context: {name}".
- The resource line shows the context in `text-warning`.
- `ConfirmDialog` reads `isProdContext(resource.context)` itself, so no call site can forget it. Props gain `prodEscalation?: boolean` (default `true`; only the unmark-production confirm sets it `false`).

While `onConfirm` runs: buttons disabled, confirm shows spinner, Esc ignored. On error: dialog stays, error shown inline in `text-danger` above buttons. On success: dialog closes, toast "Restart triggered for checkout".

Dialog: 440px, `bg-surface`, `shadow-dialog`, `rounded-xl`, padding 20px, title `text-md` 600. Focus trapped, returns focus to the invoking button on close. `role="alertdialog"` for tier 2-3, `aria-describedby` on body.

---

## 8. Accessibility checklist (applies to every PR)

- Status = icon shape + text + color. No dot-only or color-only status anywhere (including UsageBar thresholds and sidebar badges).
- Text contrast ≥ 4.5:1 for anything you need to read; `text-faint` reserved for placeholders, disabled and empty markers.
- Non-text UI (focus ring, status icons, bars) ≥ 3:1 against adjacent background.
- Every interactive element reachable by keyboard; visible `:focus-visible` ring; no `focus:outline-none` without a replacement (current `ThemeToggle.svelte:83`).
- Tables: `role="grid"`, `aria-rowcount`, `aria-selected` on the keyboard-selected row, `aria-sort` on sorted column. Live region (`aria-live="polite"`, visually hidden) announces "4 failing pods" when the failing count changes, debounced 5s, only on Pulse and Pods.
- Icon-only buttons have `aria-label`; tooltips are not the only label.
- Min target 24×24 (compact), 28×28 (comfortable).
- `prefers-reduced-motion` respected (2.7).
- Shortcuts never fire while typing in inputs (keep `isInputFocused` in `App.svelte:37-42`; DataTable uses the same helper, extended to `role="combobox"`).

---

## 9. Phased implementation plan

Pulse is the priority, so it comes right after the foundation.

```
PR0  deps: Tailwind 4 + Vite (separate, already approved)
PR1a Foundation UI ─┐
PR1b Triage rules  ─┴─► PR2 Pulse·Issues ─┬─► PR3 Pulse·Timeline ─┐
                                          ├─► PR4 Pulse·Capacity ─┼─► PR10 Detail windows ─► PR11 Cleanup
                                          ├─► PR5 Pulse·Fleet ────┤
                                          └─► PR6 Shell ─► PR7a/7b Lists ─► PR8 New types, PR9 CRDs
```

- PR1a and PR1b can be built in parallel. PR3, PR4, PR5 and PR6 can all run in parallel after PR2 (they touch different files; PR3 and PR4 share `history.rs`, so PR3 owns its creation and PR4 rebases).
- The shell stays separate from Pulse (PR6) because it's a big, mostly visual diff; Pulse ships inside the current shell first. Sidebar badges and the status bar read the Pulse store when PR6 lands.

Every PR follows the repo rule: wait for the Claude Code review, fix or reply to every finding, then merge.

Common acceptance for all PRs:
- Built on the Tailwind 4 deps PR (rebase onto it); no `tailwind.config.js` or PostCSS config reintroduced.
- `npm run build`, `npm run check` (svelte-check) and `npm test` pass; `cargo test` passes when Rust is touched (including the `spec/triage` fixture runner from PR1b on).
- Dark and light, compact and comfortable, verified by screenshots in the PR description.
- No new runtime dependencies (dev-only `vitest` is fine).
- Keyboard nav invariants intact: `j/k/gg/G/Enter/Esc`, `selectedRowIndex` highlight, NodeList polling invariants and the disk-sweep rules (CLAUDE.md).

### PR1a: Foundation UI (tokens, density, Neon removal, core components, prod guard logic)

Files:
- `src/styles/app.css`: the full 2.8 file (`@theme`, `@theme inline`, runtime tokens, legacy aliases, base layer), density variables (3.2). Default palette kept until PR7b (see 2.8).
- **Neon theme removal**: `src/styles/app.css` (`.neon` block, today `app.css:73-129`), `src/lib/stores/theme.ts` (drop `neon` from `Theme` / `ResolvedTheme` and the `classList.remove` list; migrate a stored `neon` to `dark` on load), `src/lib/components/ui/ThemeToggle.svelte` (remove the Neon option and `NeonIcon`; the component itself is replaced in PR6). These are the only three files that reference it (checked).
- `src/lib/stores/settings.ts`: `density`, `prodOverrides`, `mutedIssues`; `src/lib/stores/density.ts` (new); `src/lib/stores/production.ts` (new, 4.6 logic only: `isProdContext`, mark/unmark/reset).
- `src/App.svelte`: import density store; `⌘⇧D`. `DetailWindow.svelte`: import density store.
- New in `src/lib/components/ui/`: `StatusPill`, `DataTable`, `PageHeader`, `EmptyState`, `Button`, `IconButton`, `Kbd`, `Sparkline`, `Toaster` + `src/lib/stores/toast.ts`, `ConfirmDialog` (tiers + prod escalation from 7.4), `status-icons.ts`; `src/lib/actions/tooltip.ts`.
- `src/lib/utils/health.ts`: verdict → display state mapping (11.1), no rules.
- `src/lib/components/ui/UsageBar.svelte`: restyle, same props.
- `Terminal.svelte`: palette from tokens.
- `package.json`: `vitest` devDependency, `npm test`.

Acceptance:
- Unmigrated views render correctly through aliases in both themes.
- Stored theme `neon` lands on Dark with no errors; the theme menu lists Light, Dark, System only; `grep -ri neon src` returns nothing.
- ConfirmDialog: tier behaviors and prod escalation covered by component tests (context `eks-prod-eu` auto-escalates; an unmarked one doesn't).
- DataTable: selection stays on the same row key across re-sorts (unit test with a reordering store).

### PR1b: Triage rules module + verdicts + Pods reference view

Files:
- `spec/triage/rules.md`, `spec/triage/schema/verdict.schema.json` (with `severity` and `tone`), `spec/triage/fixtures/**` for the per-object rules in 6.10 that apply to pods, workloads, nodes and PVCs. If the mobile track already created `spec/triage/`, extend it instead.
- `src-tauri/src/triage/rules.rs`, `fixtures.rs` (runner). `src-tauri/src/kubernetes.rs`: `pod_to_info`, workload and node `*_to_info` functions attach `verdict: Verdict` to `PodInfo`, `DeploymentInfo`, `StatefulSetInfo`, `DaemonSetInfo`, `ReplicaSetInfo`, `JobInfo`, `NodeInfo`, `PersistentVolumeClaimInfo`.
- `src/lib/stores/kubernetes.ts`: `Verdict` type on those interfaces; `podsByStatus` rewritten on verdicts (buckets `all, failed, warning, progressing, healthy, completed`); `resourceState` helper; `lastSyncedAt`.
- `src/lib/components/views/PodList.svelte`: migrated to PageHeader + DataTable as the reference view. List-level delete button and `d` delete removed (pod delete stays in PodDetail). `keyboard.ts` shortcut text updated.

Acceptance:
- A pod in CrashLoopBackOff shows `✕ CrashLoopBackOff` with icon in Pods and appears under `Failing`.
- `cargo test` runs every fixture in `spec/triage/fixtures/` and fails if an expected file changes.
- Pods table: compact ≥ 20 rows in a 600px table area, comfortable ≥ 15; skeleton, filtered-empty and error states reachable.

### PR2: Pulse · Issues engine + Needs Attention feed

The flagship PR. Replaces `Dashboard.svelte`.

Files:
- `src-tauri/src/triage/issues.rs` (grouping, ranking, fingerprints, 6.10) + `spec/triage/issues.md` + `spec/triage/fixtures/issues/**` (at least: 12 crashlooping pods of one Deployment → 1 issue; NotReady node absorbing its pods; Pending PVC absorbing its pod; CronJob with 2 failed Jobs; standalone high-signal event; mixed-reason workload → `Multiple`).
- `src-tauri/src/pulse/engine.rs`, `projections.rs`, `tracker.rs`; `mod.rs`; engine lifecycle hooked into app start and `switch_context`.
- `src-tauri/src/commands.rs`, `lib.rs`: `get_pulse_snapshot`, `get_pulse_issue`, `pulse_refresh`, `restart_daemonset`; `pulse://snapshot` event.
- `src-tauri/src/kubernetes.rs` `get_pulse_metrics` (`:2863-2869`): pod OK classification uses the triage verdict, and `PulseMetrics` gains `nodes_ready`, `nodes_not_ready` (compatibility until PR11).
- `src/lib/stores/pulse.ts` (new); `App.svelte` subscribes; notifications move from per-Warning-event (`kubernetes.ts:733-790`) to new-critical-issue (6.3).
- `src/lib/components/views/Dashboard.svelte` → rewritten as `src/lib/components/pulse/PulseView.svelte` (route stays `dashboard`), with `HealthStrip.svelte`, `NeedsAttention.svelte`, `IssueItem.svelte`, `IssueExpanded.svelte`, `Inventory.svelte`. Capacity panel shows the current single values (no sparklines yet) until PR4; timeline panel placeholder hidden until PR3.
- `CLAUDE.md`: Pulse section, new commands, updated Architecture Note 7 (quick actions principle, 6.3).

Acceptance:
- **5-second test** (6.1) passes on a kind cluster seeded with: a Deployment of 12 pods crashlooping, an ImagePullBackOff pod, an OOMKilled pod, a CronJob with 2 failed runs, a cordoned node, a NotReady node (kubelet stopped), a Pending PVC with a pod waiting on it, and repeated `FailedScheduling` events. The feed shows exactly one item per root cause, ordered: NotReady node (critical), checkout (critical), then errors, then warnings.
- First issues render ≤ 3s after connect on a 2,000-pod cluster (record the timing in the PR).
- `j/k/Enter/Space/L/V/Y/C/M` work; `⇧R` opens ConfirmDialog naming deployment, namespace and context; on `eks-prod-*` it's escalated (focus on Cancel, ⌘↵ required). No delete anywhere on Pulse.
- Selection stays on the same issue while snapshots update; new issues show the "N new" pill instead of shifting the list.
- Revoking `list pods` cluster-wide (RBAC) shows `▲ Partial` and keeps workload and node issues.
- Context switch: no snapshot from the old context ever paints the new one (test by switching during the initial list).
- Engine memory on a 10,000-pod synthetic list stays under 20 MB (projection test).

### PR3: Pulse · History + change timeline

Files:
- `src-tauri/src/pulse/history.rs` (ring buffers, `HistoryProvider` trait, `LocalHistory`, optional persistence), `changes.rs` (detection + backfill), `correlate.rs`; sampler stub recording issue counts, warning/min and restarts/min (metric series come in PR4).
- Commands: `get_pulse_timeline`, `set_pulse_persistence`, `clear_pulse_history`; events `pulse://change`, `pulse://sample`.
- Snapshot issues gain `triggers` (top scored change); `IssueExpanded` shows "Possible trigger".
- `src/lib/components/pulse/Timeline.svelte`, `TimelineLane.svelte`, `ChangeList.svelte`; settings → "Keep Pulse history between launches".

Acceptance:
- `kubectl set image` on a Deployment shows a ◆ within 15s with the image diff and actor; `kubectl rollout restart` shows ↻; `kubectl scale` shows ▲/▼; an HPA rescale is labeled `HPA`; cordon shows ⊘.
- Restarting the app shows backfilled rollouts and scale events from the last hour as hollow markers.
- Breaking a Deployment with a bad image produces an issue whose expanded view says "started N min after rollout #x" and the timeline draws a spike guide with that rollout highlighted.
- Selecting an issue highlights its onset and trigger on the timeline; `T` + `←/→` walks markers; the Changes list matches the chart.
- With persistence on, quitting and relaunching within 1h keeps the series with a "no data" gap; with it off, nothing is written to disk (check the app data dir).
- History memory ≤ 8 MB per context with 200 nodes (unit test on buffer sizing).

### PR4: Pulse · Capacity and trends

Files:
- `src-tauri/src/pulse/sampler.rs`: node CPU/memory via the `get_node_metrics` path, disk from the existing sweep cache only, requests from projections; per-node series with the 200-node cap.
- Command `get_pulse_capacity`; engine rules TRI-NODE-005 and TRI-NODE-006 enabled (they need samples).
- `src/lib/components/pulse/CapacityPanel.svelte`, `PressureList.svelte`, `CapacityPopover.svelte`; `Sparkline` usage.

Acceptance:
- Without metrics-server: usage rows show `–` with the neutral hint; requests and pod slots still render; no 0% anywhere.
- With `nodes/proxy` denied: disk shows `–`; the sampler never triggers a sweep of its own (assert the sweep count in a test).
- A node driven to > 90% memory for 5 min appears in "Under pressure" and as a TRI-NODE-005 issue.
- Disk time-to-full appears only under the 6.6 conditions and always says "(estimate)".

### PR5: Pulse · Multi-cluster fleet

Files:
- `src-tauri/src/pulse/fleet.rs` (6.14), fleet-level use of `triage::issues` (workload-level), commands `get_fleet_overview`, `set_fleet_config`, `fleet_refresh`, event `pulse://fleet`.
- `src/lib/components/pulse/FleetStrip.svelte`, `FleetTable.svelte`; settings → Fleet (enable, interval, include/exclude).

Acceptance:
- With 10 contexts where 2 are unreachable and 1 has an expired exec credential: the current cluster's feed is unaffected (first issues still ≤ 3s), unreachable ones show `stale` after one failure and `unreachable` after two, the expired one shows `auth` without opening any browser window.
- A sweep of 16 contexts never has more than 4 in flight (instrumented test) and no context cycle exceeds 8s.
- Clicking a chip switches context; its Pulse loads with the full engine; the old context's history is kept (switch back shows it).
- Excluding a context stops its polling immediately.

### PR6: Shell (sidebar, header, status bar, prod marker, settings)

Files:
- `Sidebar.svelte` (4.2: sections, collapsible + persisted, worst-badge on collapsed sections, PINNED, badges from `pulse.ts`), `Header.svelte` (4.3), `StatusBar.svelte` (new, freshness from `pulse.ts`), `SettingsMenu.svelte` replacing `ThemeToggle` + `NotificationToggle` (delete both), `Combobox` / `CustomSelect searchable`, `Menu`, `Popover`.
- Production marker visuals (4.6): window edge line (main, detail, terminal windows), context button and menu rows, status bar, Settings → Production contexts, ⌘K mark/unmark, switch-into-prod toast.
- `GlobalSearch.svelte`: "Go to" with short names and `:` mode; `KeyboardHelp.svelte`: new shortcuts including the Pulse group.
- `CapacityRail.svelte`: restyled, reads `pulse.ts` (its own poll removed). `StartupScreen.svelte`: tokens only.

Acceptance:
- Sidebar badges equal the Pulse counts and update within one snapshot; collapsed sections still show a badge when an item has problems.
- A context named `eks-prod-eu` shows the edge line, shield and `PROD` tag with no setup; marking `staging` and unmarking `products-dev` persist across restarts; detail windows of a prod context show the edge line.
- `:po` + Enter in ⌘K jumps to Pods. Clicking empty sidebar space no longer collapses it.
- No component other than `pulse.ts` calls `get_pulse_metrics`.

### PR7: Migrate list views (7a + 7b)

Split so each stays reviewable. Each view: replace toolbar with `PageHeader`, table with `DataTable`, status cells with `StatusPill`, delete per-view keyboard handlers and scroll/anchor effects, wire `resourceState`.

- **PR7a, workloads + nodes + events (9 views)**: `DeploymentList`, `StatefulSetList`, `DaemonSetList`, `ReplicaSetList`, `JobList`, `CronJobList`, `HPAList`, `NodeList`, `EventList`. Remove list-level scale/restart buttons (`DeploymentList.svelte:267-277`, `StatefulSetList.svelte:173-183`) and their `confirm()` calls. NodeList: anchor logic now inside DataTable, **keep** `loadNodeMetrics` polling, in-flight and context guards untouched; usage columns stay sortable. EventList: default sort warnings first then newest, Normal shown neutral, segments `All / Warning / Normal`, reason filter stays as `CustomSelect`.
- **PR7b, network, config, storage, cluster (10 views)**: `ServiceList`, `IngressList`, `NetworkPolicyList`, `ConfigMapList`, `SecretList`, `PVList`, `PVCList`, `NamespaceList`, `ServiceAccountList`, plus `PodList` follow-ups if any. SecretList: remove list delete (`SecretList.svelte:190`) and list edit (`:173`); "Create secret" stays as PageHeader action. NetworkPolicyList: "Create policy" PageHeader action. Last commit removes legacy CSS aliases and `StatusBadge`, `Badge`, `ViewFilter`, `TabFilter`, `MetricCard` once grep shows no users.

Acceptance (both):
- All 20 views support `j/k/gg/G/Enter/y/r` and `⌘F`.
- No destructive buttons or keys in any list view (grep: no `delete`, `restart`, `scale` invokes in `src/lib/components/views/`).
- No raw Tailwind palette classes (`grep -E "(purple|blue|cyan|pink|amber|indigo|violet|emerald|orange|rose|teal)-[0-9]{3}" src/lib/components/views` returns nothing).
- Every view shows loading, empty, filtered-empty and error states correctly.
- Each migrated view file is at least 40% shorter (sanity check that duplication actually went away).

### PR8: New built-in resource types (after PR7b)

Builds the 11 views of 12.1 on the shared components only. Runs after PR7b; PR8 and PR9 can run in parallel. Split into two PRs if the diff passes ~1,500 lines: **PR8-1** RBAC (4 types) + PriorityClasses, **PR8-2** StorageClasses, EndpointSlices/Endpoints, IngressClasses, PDBs, ResourceQuotas, LimitRanges.

Files:
- `src-tauri/src/kubernetes.rs`: `{Type}Info` and `{Type}Detail` structs, list/detail/yaml/events functions per type (12.5), `get_role_bindings_for`, `get_pdb_pods`.
- `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`: commands + `generate_handler![]` registration; `open_resource_detail` accepts the new `resourceType` ids.
- `src/lib/stores/kubernetes.ts`: interfaces, stores, `load*` functions with `resourceState`.
- `src-tauri/src/triage/rules.rs` + `spec/triage/fixtures/**`: PDB, ResourceQuota, EndpointSlice verdicts (11.5); `verdict` on the new `*Info` structs.
- `src/lib/components/views/`: `RoleList`, `RoleBindingList`, `ClusterRoleList`, `ClusterRoleBindingList`, `StorageClassList`, `EndpointsList` (slices + legacy switch), `IngressClassList`, `PDBList`, `ResourceQuotaList`, `LimitRangeList`, `PriorityClassList`. Each is a thin column config over DataTable.
- `src/lib/components/detail/ResourceDetail.svelte` + `DetailWindow.svelte`: 11 new `resourceConfig` entries and their tab bodies (Rules, Bindings, Subjects, Parameters, PVs, Endpoints, Pods, Usage, Limits).
- `src/lib/stores/navigation.ts`, `MainPanel.svelte`, `Sidebar.svelte`, `GlobalSearch.svelte`: view ids, routes, sidebar entries, short names.
- Pulse: TRI-PDB-001/002 and TRI-QUOTA-001 added to `triage/rules.rs` + fixtures, polled 60s by the engine (6.9), shown in the feed with kind tags `pdb`, `quota`; sidebar badges for PDBs and Resource Quotas.
- `CLAUDE.md`: resource tables, command tables.

Acceptance:
- Every new view lists, filters, sorts, supports `j/k/gg/G/Enter/y/r`, and shows loading / empty / filtered / error / forbidden states (test forbidden with a user lacking `list` on `roles`).
- Every new type opens a detail window with Overview, Events and YAML, plus its extra tabs.
- A PDB with `maxUnavailable: 0` shows `▲ Blocks evictions`; a quota at 100% of `pods` shows `✕ At limit`; a Service with no ready endpoints shows `▲ No ready endpoints` in Endpoints.
- No view file in this PR contains its own `<table>` markup or keyboard handler.
- `cargo test` covers the PDB / quota / endpoint health inputs the backend computes.

### PR9: Generic CRD browser (after PR7b)

Files:
- `src-tauri/src/kubernetes.rs`: discovery cache (5 min per context), `get_crds`, CRD detail/yaml/events, Table-based `get_custom_resources`, `get_custom_resource_detail` / `_yaml` / `_events` (12.5); unit tests for Table-response parsing (fixtures: cert-manager `Certificate`, Argo CD `Application`, Karpenter `NodePool`, a CRD with no printer columns).
- `src-tauri/src/commands.rs`, `lib.rs`: commands; `open_resource_detail` accepts `resourceType: 'custom'` with `group, version, plural, kind`, and `'crd'`.
- `src/lib/stores/kubernetes.ts`: `crds` store, `loadCrds`, `loadCustomResources(target, ns)` with 10s polling + in-flight and context guards.
- `src/lib/stores/navigation.ts`: `crds` and `custom` view ids, `customResourceTarget`, pinned/recent CRD kinds (settings).
- `src/lib/components/views/CrdList.svelte`, `CustomResourceList.svelte`; `src/lib/components/detail/CustomResourceDetail.svelte`, CRD detail via `ResourceDetail` config (`crd`, cluster-scoped, extra tabs Schema and Printer columns).
- `src-tauri/src/triage/rules.rs` + `spec/triage/fixtures/**`: CRD and generic custom resource verdicts (11.5); verdicts ship in `CustomResourceTable` rows.
- `Sidebar.svelte`, `GlobalSearch.svelte`: CUSTOM RESOURCES section (Definitions + pinned/recent kinds), CRD kinds and their `shortNames` in "Go to".

Acceptance:
- On a cluster with cert-manager, `Certificates` lists the same columns kubectl prints (`kubectl get certificates -A`), with `Ready` rendered as a StatusPill and not-ready certificates sorted first.
- Wide (`priority > 0`) columns show in compact, hide in comfortable, and the Columns menu overrides per kind.
- A CRD without printer columns shows Name, Namespace, Status (if conditions exist), Age.
- Pinning a kind puts it in the sidebar; `:cert` (its short name) in ⌘K jumps to it.
- A user without access to a CR kind gets the forbidden EmptyState, not an error; deleting the CRD while its list is open shows "This resource type no longer exists".
- The IPC payload for a 500-object list contains no `spec` fields (backend strips `includeObject`).

### PR10: Detail windows

Files:
- `src/lib/components/detail/DetailWindow.svelte`: new titlebar with kind tag, health pill, context/namespace, lock, action slot.
- `src/lib/components/detail/ResourceDetail.svelte`, `PodDetail.svelte`, `DeploymentDetail.svelte`, `StatefulSetDetail.svelte`: shared `Tabs`, two-column overview with meta rail, problem banner, inline stat rows, actions per 7.3, every `confirm()` and hand-rolled modal replaced by `ConfirmDialog`, `[`/`]` and `1`–`9` tab keys.
- `src/lib/components/ui/`: `KeyValueList.svelte` (new), `ProblemBanner.svelte` (new); restyle `EventsTable`, `ConditionsTable`, `MetadataSection`, `YamlEditorPanel` (its `confirm()` at `:41` becomes a ConfirmDialog tier 1), `TombstoneOverlay`.
- `src/lib/components/LogViewer.svelte`, `Terminal.svelte` chrome: tokens and Buttons only (no behavior change).
- `src/lib/components/ui/SecretCreator.svelte`, `NetworkPolicyCreator.svelte`: tokens, Buttons, ConfirmDialog for destructive steps (visual only, no logic change).
- Pod delete lives in PodDetail `⋯` menu with tier-2 confirm (verifies PR1's removal didn't drop the capability).

Optional split if the diff gets big: PR10a = shell + Pod/Deployment/StatefulSet, PR10b = ResourceDetail (15 types) + creators.

Acceptance:
- No native `confirm()` left (`grep -rn "confirm(" src` returns only `ConfirmDialog` usages).
- Every destructive action shows resource kind, name, namespace and context in its dialog; tier-2 dialogs open with focus on Cancel.
- Failed/warning resources show the problem banner with a correct one-line cause (CrashLoop pod: container name + restarts + last exit reason).
- Deleted resources show the read-only banner and disabled actions.
- Density applies inside detail windows and follows toggles made in the main window.
- On a production context, a restart dialog opens with focus on Cancel, shows the PROD banner and "Restart on {context}", and plain Enter does not confirm; deleting a pod requires typing its name.

### PR11: Cleanup (small)

- Delete leftover legacy components and aliases if PR7b didn't; remove `get_pulse_metrics` and `PulseMetrics` once nothing uses them.
- Update `CLAUDE.md`: UI Theme section (new token names, Dark/Light only), production guard, status model pointer (section 11), Shared UI Components table (DataTable, PageHeader, StatusPill, EmptyState, ConfirmDialog, Button), Keyboard Shortcuts table (`d` change, new keys), View Component Pattern (now DataTable-based).
- Screenshot set of every screen in both themes and densities in `docs/design/screens/`.

---

## 10. Resolved decisions

Answered by the owner on 2026-10-28:

1. **Accent**: teal → **indigo**, as proposed (2.3).
2. **Neon theme**: **dropped**. Dark + Light only (System picks between them). Removal is part of PR1.
3. **Production guard**: auto-flag context names matching `/prod|prd/i`, **plus** manual mark/unmark per context, persisted locally. Spec in 4.6, dialog escalation in 7.4. Logic and dialogs in PR1a, visual marker and toggles in PR6.
4. **Pulse is the flagship**: issues feed + quick actions, change timeline, capacity + trends, and multi-cluster overview are all in scope; 1h in-app history (Rust ring buffer, optional local persistence), no Prometheus in v1 but a seam for it; restart/scale allowed from Pulse via ConfirmDialog, delete never. Section 6, PR2-PR5.

Also folded in: support for all popular resource types and a generic CRD browser (section 12, PR8 and PR9), and the status model as a shared contract for the mobile apps (section 11).

No open questions remain.

---

## 11. Status model (shared contract: desktop + mobile)

**This is the one place that defines how status looks and reads.** Desktop (`StatusPill`, tables, sidebar badges, Pulse) and the mobile apps (`docs/mobile/architecture.md`) both follow it. **Detection rules** (which object gets which verdict, and how findings group into issues) live in `spec/triage/` and are specified in 6.10; this section defines the display contract on top of those verdicts. Tokens referenced here are defined in 2.2-2.4.

This section supersedes the icon proposal in mobile architecture section 8 ("filled dot / half dot / triangle"): both platforms use the shapes in 11.1.

### 11.1 States

| State | Means | Rank (sort key) | Icon shape | SF Symbol / Material equivalent | Label rule |
|---|---|---|---|---|---|
| `failed` | Broken now, needs a human | 0 | ✕ inside an **octagon** | `xmark.octagon` / `dangerous` | The specific reason (`CrashLoopBackOff`, `NotReady`, `At limit`) |
| `warning` | Degraded or at risk | 1 | ! inside a **triangle** | `exclamationmark.triangle` / `warning` | The specific reason (`1/3 ready`, `Pending`, `No ready endpoints`) |
| `progressing` | Changing on purpose (rollout, scheduling, terminating) | 2 | **Half-filled circle**, static | `circle.lefthalf.filled` / `contrast` | What is happening (`Rolling out`, `ContainerCreating`, `Terminating`) |
| `unknown` | No data, or data we can't interpret | 3 | ? inside a circle | `questionmark.circle` / `help` | `Unknown` |
| `healthy` | Working as intended | 4 | ✓ inside a circle | `checkmark.circle` / `check_circle` | The normal state word (`Running`, `Ready`, `Bound`) |
| `neutral` | Informational, no health meaning | 5 | **Empty circle** | `circle` / `radio_button_unchecked` | The state word (`Completed`, `Normal`, `Suspended`, `Scaled to 0`) |

Rank drives every "problems first" sort and every "worst of" aggregation.

**Mapping from the shared triage verdict** (6.10): `fail` → failed, `warn` → warning, `unknown` → unknown, `ok` + `tone: progressing` → progressing, `ok` + `tone: neutral` → neutral, `ok` → healthy. Within failed, `severity` (`critical` / `error`) only affects ranking and the `CRIT` / `ERR` tag in the Pulse feed; both render with the failed icon and colors.

### 11.2 Colors

| State | Text (fg) dark / light | Icon, dot, bar (solid) dark / light | Background tint |
|---|---|---|---|
| failed | `#F87171` / `#C4252B` | `#E5484D` / `#DC2626` | solid at 12% dark, 10% light |
| warning | `#F5B74A` / `#985700` | `#F59E0B` / `#D97706` | same |
| progressing | `#7CB4FF` / `#1D63C9` | `#3B82F6` / `#2563EB` | same |
| healthy | `#4CCF7F` / `#18794E` | `#30A46C` / `#16A34A` | same |
| unknown, neutral | text-secondary / text-secondary | text-muted / text-muted | none |

Desktop utilities: `text-{state}`, `bg-{state}-solid`, `bg-{state}-tint`, where `{state}` maps failed→`danger`, warning→`warning`, progressing→`info`, healthy→`success`.

Measured contrast: every fg is ≥ 5.4:1 on the app background and ≥ 4.5:1 on the active background in both themes, and ≥ 4.85:1 on its own tint. Every solid is ≥ 3.19:1 vs the app background (light warning is the tightest, which is why an icon and a label are mandatory).

### 11.3 Presentation rules

1. **Never color alone.** Every status shows icon shape + text label. Color is the third, redundant signal. This also applies to counts (badges show the number; warning badges add ▲), usage bars (thresholds add ▲ / ✕ after the number) and row rails (decorative, the pill carries meaning).
2. **Reason over phase.** Show the most specific cause available. A pod in phase `Running` with a container in `CrashLoopBackOff` is labeled `CrashLoopBackOff`, state `failed`.
3. **Zero problems isn't green.** "0 failing" renders neutral (`text-secondary`), not success.
4. **Normal events are neutral**, not healthy. Only Warning events carry a state (warning).
5. **Labels**: source casing for Kubernetes reasons (`ImagePullBackOff`), sentence case for our own words (`1/3 ready`, `At limit`). Truncate after 24 chars with the full text in a tooltip / long-press; never truncate below 12 chars.
6. **Aggregation** (sidebar section headers, Pulse tiles, a deployment's pod list): worst rank wins; counts are per state, never blended into a score.
7. Accessible name pattern: "{label}, {state}" (e.g. "CrashLoopBackOff, failed"), so screen readers don't depend on the icon.

### 11.4 Thresholds

| Signal | warning | failed |
|---|---|---|
| Pod restarts (total) | ≥ 5 | ≥ 20 (restart count alone never makes a pod failed; it only colors the restarts cell) |
| Node / cluster CPU or memory usage (of allocatable) | ≥ 75% | ≥ 90% |
| ResourceQuota usage (worst resource, used / hard) | ≥ 90% | ≥ 100% |
| Warning-event window for badges and Pulse | last 1h | — |
| Data freshness (status bar) | no sync for 30s ("stale") | disconnected |

### 11.5 Per-resource rules

First match wins, left to right (failed → warning → progressing → healthy → neutral). Resources without a row have no health and show no status column.

| Resource | failed | warning | progressing | healthy | neutral |
|---|---|---|---|---|---|
| Pod | phase `Failed`; any container state in `CrashLoopBackOff, ImagePullBackOff, ErrImagePull, CreateContainerConfigError, CreateContainerError, InvalidImageName, OOMKilled, Error, RunContainerError` | phase `Running` but not all containers ready; restarts ≥ 5; phase `Unknown` | phase `Pending`; state `ContainerCreating`, `PodInitializing` | `Running` and all ready | `Succeeded` ("Completed") |
| Deployment / StatefulSet / ReplicaSet | desired > 0 and ready == 0 | 0 < ready < desired | updated < desired ("Rolling out") | ready == desired > 0 | desired == 0 ("Scaled to 0") |
| DaemonSet | desired > 0 and ready == 0 | ready < desired or misscheduled > 0 | updated < desired | ready == desired | desired == 0 |
| Job | failed > 0 and active == 0 | failed > 0 and active > 0 | active > 0 | succeeded ≥ completions | suspended |
| CronJob | last job failed (if data available) | — (later: missed schedules) | active > 0 | otherwise | suspended ("Suspended") |
| Node | status contains `NotReady` or `Unknown` | `SchedulingDisabled`; any `*Pressure` condition | — | `Ready` | — |
| PVC | `Lost` | `Pending` | — | `Bound` | — |
| PV | `Failed` | `Released` | `Pending` | `Bound` | `Available` |
| Namespace | — | — | `Terminating` | `Active` | — |
| Event | — | `Warning` | — | — | `Normal` |
| PodDisruptionBudget | currentHealthy < desiredHealthy ("Below minimum") | disruptionsAllowed == 0 ("Blocks evictions") | — | otherwise | expectedPods == 0 ("No pods") |
| ResourceQuota | worst usage ≥ 100% ("At limit") | worst usage ≥ 90% ("Near limit") | — | otherwise | no `hard` entries |
| EndpointSlice / Endpoints | — | 0 ready endpoints ("No ready endpoints"); some not ready ("2/3 ready") | any `terminating` | all ready | no endpoints and service has no selector ("Manual") |
| CustomResourceDefinition | condition `Established` or `NamesAccepted` is `False` | — | `Established` missing or `Unknown` | `Established=True` | — |
| Custom resource (generic, see 12.3) | first of conditions `Ready`, `Available`, `Synced`, `Healthy` is `False` (label = its `reason` or "Not ready") | condition `Degraded=True` | that condition is `Unknown`; `status.phase` contains Pending/Progressing/Provisioning | that condition is `True`; phase Running/Ready/Active/Bound/Succeeded | no conditions and no phase ("—" neutral, column hidden if no row has status) |
| Roles, Bindings, StorageClass, IngressClass, PriorityClass, LimitRange, ConfigMap, Secret, ServiceAccount, Service, Ingress, NetworkPolicy, HPA | no health (HPA later: at max replicas = warning) | | | | |

Phase-keyword matching for custom resources is case-insensitive; `Failed`, `Error`, `CrashLoop` in a phase map to failed.

### 11.6 Implementation contract

- **Rules source of truth**: `spec/triage/rules.md` (per-object rules, ids `TRI-*`) and `spec/triage/issues.md` (grouping, ranking), with golden fixtures in `spec/triage/fixtures/**`. Desktop implements them once, in Rust (`src-tauri/src/triage/`), and attaches a `verdict` to every `*Info` struct; `cargo test` runs all fixtures. iOS and Android implement the same rules against the same fixtures (mobile architecture section 3). A rule change without a fixture change fails review.
- Every row of 11.5 and every threshold of 11.4 has at least one fixture. Display-only outcomes (e.g. PV `Available` → neutral) are verdicts too (`ok` + `tone: neutral`), so they're covered by the same fixtures.
- Desktop display mapping: `src/lib/utils/health.ts` (`toDisplay`, 5.1), no rules in TypeScript.
- Icons: `src/lib/components/ui/status-icons.ts` exports the six 12×12 SVG paths (stroke 2, round caps). Mobile uses the platform symbols listed in 11.1, matching the shapes.
- Things that are **not** status and must not borrow its icons: the accent (selection, focus, links), the production marker (4.6: shield icon + "PROD", uses warning colors but never the triangle), and the "default" tag on StorageClass / IngressClass / PriorityClass (neutral tag).

---

## 12. New resource types and generic CRD browser

The desktop covers all popular Kubernetes resources. Every new view is built **only** from the shared components (PageHeader, DataTable, StatusPill, EmptyState, Tabs, KeyValueList, ConfirmDialog) and follows the existing backend pattern: `get_{resources}` for lists and `get_{resource}_detail` / `_yaml` / `_events` for detail (CLAUDE.md, "Resource Detail Commands").

### 12.1 Built-in types

| View (sidebar) | `resourceType` id | Scope | List command | List columns | Detail tabs (beyond Overview, Events, YAML) |
|---|---|---|---|---|---|
| Roles | `role` | ns | `get_roles` | Name, Namespace, Rules (count), Age | Rules, Bindings |
| Role Bindings | `rolebinding` | ns | `get_rolebindings` | Name, Namespace, Role (`Role/x` or `ClusterRole/x`), Subjects (count + first 2), Age | Subjects |
| Cluster Roles | `clusterrole` | cluster | `get_clusterroles` | Name, Rules, Aggregated (yes/—), Age | Rules, Bindings |
| Cluster Role Bindings | `clusterrolebinding` | cluster | `get_clusterrolebindings` | Name, Role, Subjects, Age | Subjects |
| Storage Classes | `storageclass` | cluster | `get_storageclasses` | Name (+ `default` tag), Provisioner, Reclaim, Binding mode, Expansion, Age | Parameters, PVs |
| Endpoints | `endpointslice` (default) / `endpoints` | ns | `get_endpointslices`, `get_endpoints` | Name, Namespace, Service, Address type, Ready (pill, `2/3 ready`), Ports, Age | Endpoints |
| Ingress Classes | `ingressclass` | cluster | `get_ingressclasses` | Name (+ `default` tag), Controller, Parameters, Age | — |
| Pod Disruption Budgets | `pdb` | ns | `get_pdbs` | Name, Namespace, Min available / Max unavailable, Allowed disruptions, Healthy (current/desired), Status, Age | Pods |
| Resource Quotas | `resourcequota` | ns | `get_resourcequotas` | Name, Namespace, Highest usage (UsageBar + resource name), Resources (count), Status, Age | Usage |
| Limit Ranges | `limitrange` | ns | `get_limitranges` | Name, Namespace, Types (Container, Pod, PVC), Age | Limits |
| Priority Classes | `priorityclass` | cluster | `get_priorityclasses` | Name (+ `global default` tag), Value (numeric), Preemption, Age | — |

Notes:
- **Endpoints** is one sidebar item showing EndpointSlices by default (Endpoints is deprecated since 1.33). PageHeader has a segmented `EndpointSlices | Endpoints (legacy)` switch.
- **Rules tab** (Role / ClusterRole): table of API groups, resources, verbs, resource names. Verbs are mono chips; `*` in verbs, resources or groups gets a neutral "wildcard" tag (not a warning state: RBAC breadth isn't health). Aggregation selectors shown for aggregated ClusterRoles.
- **Bindings tab**: bindings that reference this role (backend lists RoleBindings / ClusterRoleBindings and filters by `roleRef`). Subjects tab: kind, name, namespace; ServiceAccount subjects link to their detail.
- **Usage tab** (ResourceQuota): one row per resource, `used / hard` with UsageBar thresholds from 11.4.
- **PDB Pods tab**: pods matching the selector, with pod health, so "Blocks evictions" is explainable.
- Status columns and sidebar badges follow 11.5: PDBs (count of failed + warning) and Resource Quotas (count ≥ 90%) get sidebar badges; the other new types don't.
- Pulse "Needs attention" includes PDBs below minimum and quotas at limit (item format from 6.3, kind tags `pdb`, `quota`, rules TRI-PDB-*, TRI-QUOTA-001).
- None of these get destructive actions in this phase. Detail actions: Copy name, Copy YAML.

### 12.2 CRD list ("Definitions")

```
┌ Custom Resource Definitions  42 ─────────────── [All 42][✕ Not established 0] [⌕ Filter…] ┐
│ KIND             GROUP                    VERSIONS        SCOPE       STATUS         AGE  │
│ Certificate      cert-manager.io          v1 ★            Namespaced  ✓ Established  210d │
│ Application      argoproj.io              v1alpha1 ★      Namespaced  ✓ Established  98d  │
│ NodePool         karpenter.sh             v1 ★, v1beta1   Cluster     ✓ Established  30d  │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

- View id `crds`. Columns: Kind (plus plural in `text-muted` on hover tooltip), Group, Versions (served versions; ★ marks the storage version; deprecated versions get a neutral `deprecated` tag), Scope, Status (11.5 CRD rule), Age.
- Default sort: Group, then Kind. Filter matches kind, plural, group and short names.
- **Enter / click** opens that kind's **resource list in the main panel** (12.3). The row action "Definition" (key `i`) opens the CRD itself in a detail window: tabs Overview (names, scope, versions, conversion strategy), Schema (read-only tree of the storage version's `openAPIV3Schema`, depth-limited with expand), Printer columns, Events, YAML.
- Pin a kind from the row's right-click menu or with `p` on the selected row; pinned kinds show under CUSTOM RESOURCES in the sidebar.

### 12.3 Generic custom resource list

- View id `custom`, with a `customResourceTarget` store in `navigation.ts`: `{ group, version, plural, kind, namespaced }`. Version = the CRD's storage version unless the user picks another served version from a PageHeader `CustomSelect` (only when more than one is served).
- PageHeader: title = Kind, subtitle = `group/version`, "cluster-scoped" when applicable. Namespace filter applies only when namespaced.
- **Columns come from the API server**, not from our own JSONPath: the backend lists with `Accept: application/json;as=Table;v=v1;g=meta.k8s.io` and `includeObject=Object`, which returns exactly what `kubectl get` prints, using the CRD's `additionalPrinterColumns`. This avoids implementing JSONPath (including the common `[?(@.type=="Ready")]` filters) and matches kubectl output.
  - Column set: Name, Namespace (if namespaced), the printer columns, Status (generic health from 11.5, shown only if at least one row has conditions or a phase), Age (only if the printer columns don't already include a date column named Age).
  - Printer column `priority > 0` ("wide" columns): shown by default in **compact** density, hidden in comfortable, toggleable from the PageHeader `⋯` "Columns" menu (choice persisted per kind).
  - Cell rendering by printer column type: `string` → text (truncate + tooltip); `integer` / `number` → right-aligned tabular; `boolean` → `true` / `false` text; `date` → relative age like the Age column; values that look like condition statuses (`True`/`False`/`Unknown` in a column whose name contains Ready/Status/Synced/Healthy) render as a StatusPill.
- Default sort: health (problems first), then name. Keyboard, selection anchoring and states are all DataTable's (5.4). 403 → `forbidden` EmptyState naming the kind.
- Refresh: polled every 10s while the view is open (no watch in this phase), with the same in-flight and context guards as `loadNodeMetrics`. A later PR can switch to a dynamic watch.

### 12.4 Generic custom resource detail

Opened via `open_resource_detail` with `resourceType: 'custom'` plus `group`, `version`, `plural`, `kind`. Routed in `DetailWindow.svelte` to a new `CustomResourceDetail.svelte` (not one of the 15 `ResourceDetail` configs).

- Titlebar: kind tag shows the Kind, health pill from the generic rule, PROD tag when relevant (4.6).
- Tabs: **Overview** (KeyValueList of metadata, owner refs as links when the owner is a known type, then the printer columns as key-values), **Conditions** (only when `status.conditions` exists; uses `ConditionsTable`, failed conditions first), **Spec & Status** (read-only collapsible tree of `spec` and `status`, depth 2 expanded, long strings truncated with copy, "Open in YAML" link), **Events** (events whose involvedObject matches kind + name + namespace), **YAML**.
- Actions: Copy name, Copy YAML. No edit or delete in this phase (YAML editing is a known TODO; delete arrives with general resource deletion, using ConfirmDialog tier 2 and prod escalation).

### 12.5 Backend commands

Built-in types (12.1): one list command each plus `get_{resource}_detail`, `get_{resource}_yaml`, `get_{resource}_events`, taking `context_name, namespace?, name` (no namespace param for cluster-scoped ones), exactly like existing types. All types are in `k8s-openapi` already (`rbac/v1`, `storage/v1`, `discovery/v1`, `networking/v1` IngressClass, `policy/v1`, `core/v1` ResourceQuota / LimitRange / Endpoints, `scheduling/v1`). Extra: `get_role_bindings_for(context_name, kind, namespace?, name)` for the Bindings tab, `get_pdb_pods(context_name, namespace, name)`.

Generic (12.2-12.4), cluster discovery cached per context for 5 minutes:

| Command | Parameters | Returns |
|---|---|---|
| `get_crds` | `context_name` | `Vec<CrdInfo { name, group, kind, plural, singular, short_names, scope, versions: Vec<{name, served, storage, deprecated}>, established, age }>` |
| `get_crd_detail` / `_yaml` / `_events` | `context_name, name` | `CrdDetail` / `String` / `Vec<Event>` (cluster-scoped pattern) |
| `get_custom_resources` | `context_name, group, version, plural, namespace?` | `CustomResourceTable { columns: Vec<{name, type, format, priority, description}>, rows: Vec<{name, namespace, uid, age, cells: Vec<serde_json::Value>, conditions: Vec<{type, status, reason, message}>, phase: Option<String>}> }`. The backend keeps only metadata, `status.conditions` and `status.phase` from each `includeObject` payload before sending, so full objects never cross IPC. |
| `get_custom_resource_detail` | `context_name, group, version, plural, namespace?, name` | `{ metadata, printer: Vec<{name, value}>, conditions, spec: Value, status: Value }` |
| `get_custom_resource_yaml` / `_events` | same | `String` / `Vec<Event>` |

Implementation: kube-rs `DynamicObject` / `ApiResource` from discovery for detail and YAML; the Table list is a raw `client.request` with the Table `Accept` header. Errors map 403 → forbidden and 404 (CRD removed) → an EmptyState "This resource type no longer exists".
