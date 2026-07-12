<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Header from './lib/components/Header.svelte';
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

  // Track 'g' key for 'gg' combo
  let lastKeyTime = 0;
  let lastKey = '';

  function isInputFocused(): boolean {
    const active = document.activeElement;
    if (!active) return false;
    const tag = active.tagName.toLowerCase();
    return tag === 'input' || tag === 'textarea' || (active as HTMLElement).isContentEditable;
  }

  function handleKeydown(e: KeyboardEvent) {
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

  onMount(() => {
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
  <div class="flex h-full bg-bg-primary">
    <!-- Sidebar -->
    <Sidebar />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- Header -->
      <Header />

      <!-- Main Panel -->
      <MainPanel view={$currentView} />
    </div>
  </div>

  <!-- Pulse Rail: capacity + live events -->
  <CapacityRail />

  <!-- Global Search Modal -->
  <GlobalSearch />

  <!-- Keyboard Shortcuts Help -->
  <KeyboardHelp />
{/if}
