<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Header from './lib/components/Header.svelte';
  import StatusBar from './lib/components/StatusBar.svelte';
  import { currentContextIsProd } from './lib/stores/production';
  import { trackWindowCount, openNewWindow } from './lib/stores/windows';
  import ConfirmDialog from './lib/components/ui/ConfirmDialog.svelte';

  // Closing the last window with the menu bar item off quits Apex; Rust
  // holds the close and asks here when that would end terminal sessions.
  let quitTerminals = $state(0);
  async function listenConfirmQuit() {
    if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return;
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
    await getCurrentWebviewWindow().listen<number>('confirm-quit', (e) => (quitTerminals = e.payload));
  }
  async function quitNow() {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('quit_app');
  }
  import MainPanel from './lib/components/MainPanel.svelte';
  import StartupScreen from './lib/components/StartupScreen.svelte';
  import GlobalSearch from './lib/components/GlobalSearch.svelte';
  import KeyboardHelp from './lib/components/ui/KeyboardHelp.svelte';
  import CapacityRail from './lib/components/ui/CapacityRail.svelte';
  import { pulseRailOpen } from './lib/stores/rail';
  import { currentView } from './lib/stores/navigation';
  import { isInitialized } from './lib/stores/startup';
  import { globalSearchOpen } from './lib/stores/search';
  import { portForwardPanelOpen } from './lib/stores/portforward';
  import {
    currentContext,
    selectedNamespace,
    refreshTrigger,
    startEventWatch,
    stopEventWatch,
  } from './lib/stores/kubernetes';
  import {
    showKeyboardHelp,
    moveUp,
    moveDown,
    goToFirst,
    goToLast,
    clearSelection,
  } from './lib/stores/keyboard';
  // Intialize theme (subscribes to changes and applies to DOM)
  import './lib/stores/theme';
  // Density: sets data-density on <html> (spec 3.1)
  import { toggleDensity, densityLabel } from './lib/stores/density';
  import { toast } from './lib/stores/toast';
  import { isTypingTarget } from './lib/utils/keys';
  import Toaster from './lib/components/ui/Toaster.svelte';

  // Track 'g' key for 'gg' combo
  let lastKeyTime = 0;
  let lastKey = '';

  function isInputFocused(): boolean {
    return isTypingTarget(document.activeElement);
  }

  function handleKeydown(e: KeyboardEvent) {
    // Cmd+Shift+D (Mac) or Ctrl+Shift+D to toggle density (works anywhere)
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === 'd') {
      e.preventDefault();
      const next = toggleDensity();
      toast(`Density: ${densityLabel(next)}`, { duration: 1500 });
      return;
    }

    // ⌘N / Ctrl+N: another main window on the shared context. On macOS the
    // File menu's accelerator handles ⌘N, so only take it elsewhere.
    if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key.toLowerCase() === 'n' && !isMacPlatform) {
      e.preventDefault();
      openNewWindow().catch((err) => toast(`Couldn't open a window: ${err}`, { health: 'failed' }));
      return;
    }

    // Don't intercept when typing in inputs
    if (isInputFocused()) return;

    // Cmd+K (Mac) or Ctrl+K (Windows/Linux) to open global search
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      globalSearchOpen.set(true);
      return;
    }

    // Cmd+P (Mac) or Ctrl+P (Windows/Linux) to open port forwards
    if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
      e.preventDefault();
      portForwardPanelOpen.update(v => !v);
      return;
    }

    // Cmd+E (Mac) or Ctrl+E (Windows/Linux) to toggle the Pulse rail
    if ((e.metaKey || e.ctrlKey) && e.key === 'e') {
      e.preventDefault();
      pulseRailOpen.update(v => !v);
      return;
    }

    // '/' to open search (vim-style)
    if (e.key === '/') {
      e.preventDefault();
      globalSearchOpen.set(true);
      return;
    }

    // '?' to show keyboard shortcuts help
    if (e.key === '?' || (e.shiftKey && e.key === '/')) {
      e.preventDefault();
      showKeyboardHelp.set(true);
      return;
    }

    // Escape to clear selection or close modals
    if (e.key === 'Escape') {
      clearSelection();
      return;
    }

    // Navigation: j/k or arrow keys
    if (e.key === 'j' || e.key === 'ArrowDown') {
      e.preventDefault();
      moveDown();
      return;
    }

    if (e.key === 'k' || e.key === 'ArrowUp') {
      e.preventDefault();
      moveUp();
      return;
    }

    // 'G' to go to last row
    if (e.key === 'G' && e.shiftKey) {
      e.preventDefault();
      goToLast();
      return;
    }

    // 'gg' combo to go to first row
    const now = Date.now();
    if (e.key === 'g' && !e.shiftKey) {
      if (lastKey === 'g' && now - lastKeyTime < 500) {
        e.preventDefault();
        goToFirst();
        lastKey = '';
        return;
      }
      lastKey = 'g';
      lastKeyTime = now;
      return;
    }

    // Reset 'g' tracking for other keys
    if (e.key !== 'g') {
      lastKey = '';
    }
  }

  // Global event watch — runs for the whole app lifetime so that
  // Warning-event notifications fire regardless of which view is open.
  // Restarts whenever the context, namespace, or manual refresh changes.
  $effect(() => {
    if (!$isInitialized) return;
    const ctx = $currentContext;
    const ns = $selectedNamespace;
    const _trigger = $refreshTrigger;
    if (!ctx) return;
    startEventWatch(ns);
  });

  const isMacPlatform = typeof navigator !== 'undefined' && /Mac/.test(navigator.platform || navigator.userAgent);

  onMount(() => {
    trackWindowCount();
    listenConfirmQuit();
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
    stopEventWatch();
  });
</script>

{#if !$isInitialized}
  <StartupScreen />
{:else}
  <!-- Production marker: a 2px warning line across the top (spec 4.6) -->
  {#if $currentContextIsProd}<div class="prodline" aria-hidden="true"></div>{/if}

  <!-- Shell grid (spec 4.1, 13.2): sidebar | header / main, status bar below -->
  <div class="app-shell">
    <Sidebar />
    <Header />
    <MainPanel view={$currentView} />
    <StatusBar />
  </div>

  <!-- Pulse Rail: capacity + live events -->
  <CapacityRail />

  <!-- Global Search Modal -->
  <GlobalSearch />

  <!-- Keyboard Shortcuts Help -->
  <KeyboardHelp />
{/if}

<Toaster />

{#if quitTerminals > 0}
  <ConfirmDialog
    open={true}
    title="Quit Apex?"
    resource={{ kind: 'App', name: 'Apex', context: $currentContext }}
    confirmLabel="Quit"
    tone="danger"
    prodEscalation={false}
    body={`${quitTerminals} ${quitTerminals === 1 ? 'terminal is' : 'terminals are'} still open. Quitting ends ${quitTerminals === 1 ? 'its session' : 'their sessions'}. Turn on Show in Menu Bar to keep Apex running instead.`}
    onConfirm={quitNow}
    onClose={() => (quitTerminals = 0)}
  />
{/if}

<style>
  .app-shell {
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    grid-template-rows: 44px minmax(0, 1fr) 24px;
    height: 100%;
    background: var(--ui-app);
  }
  .prodline {
    position: fixed;
    left: 0;
    right: 0;
    top: 0;
    height: 2px;
    background: var(--ui-warning-solid);
    z-index: 60;
    pointer-events: none;
  }
  @media (min-width: 640px) and (max-width: 1199.98px) {
    .app-shell {
      grid-template-columns: 52px minmax(0, 1fr);
    }
  }
  @media (max-width: 639.98px) {
    .app-shell {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
