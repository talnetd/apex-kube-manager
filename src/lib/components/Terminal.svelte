<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import '@xterm/xterm/css/xterm.css';

  interface Props {
    namespace: string;
    podName: string;
    container?: string;
    onClose?: () => void;
  }

  let { namespace, podName, container, onClose }: Props = $props();

  let terminalContainer: HTMLDivElement;
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;

  onMount(() => {
    terminal = new Terminal({
      cursorBlink: true,
      fontSize: 13,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#0a0a0a',
        foreground: '#ffffff',
        cursor: '#00d4aa',
        cursorAccent: '#0a0a0a',
        selectionBackground: 'rgba(0, 212, 170, 0.3)',
        black: '#000000',
        red: '#ef4444',
        green: '#22c55e',
        yellow: '#f59e0b',
        blue: '#3b82f6',
        magenta: '#a855f7',
        cyan: '#00d4aa',
        white: '#ffffff',
        brightBlack: '#555555',
        brightRed: '#f87171',
        brightGreen: '#4ade80',
        brightYellow: '#fbbf24',
        brightBlue: '#60a5fa',
        brightMagenta: '#c084fc',
        brightCyan: '#5eead4',
        brightWhite: '#ffffff',
      },
    });

    fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon();

    terminal.loadAddon(fitAddon);
    terminal.loadAddon(webLinksAddon);

    terminal.open(terminalContainer);
    fitAddon.fit();

    // Write initial message
    terminal.writeln(`\x1b[36mConnecting to pod: ${podName}\x1b[0m`);
    terminal.writeln(`\x1b[90mNamespace: ${namespace}${container ? `, Container: ${container}` : ''}\x1b[0m`);
    terminal.writeln('');

    // Handle window resize
    const handleResize = () => {
      fitAddon?.fit();
    };
    window.addEventListener('resize', handleResize);

    // TODO: Implement actual pod exec via Tauri events
    // For now, show a placeholder message
    terminal.writeln('\x1b[33mTerminal exec functionality coming soon...\x1b[0m');
    terminal.writeln('\x1b[90mThis will connect to the pod shell via kubectl exec.\x1b[0m');

    return () => {
      window.removeEventListener('resize', handleResize);
      terminal?.dispose();
    };
  });

  onDestroy(() => {
    terminal?.dispose();
  });
</script>

<div class="flex flex-col h-full bg-bg-primary">
  <!-- Terminal Header -->
  <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
    <div class="flex items-center gap-3">
      <svg class="w-4 h-4 text-accent-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
      <div>
        <span class="text-sm text-text-primary font-medium">{podName}</span>
        <span class="text-xs text-text-muted ml-2">{namespace}</span>
        {#if container}
          <span class="text-xs text-text-muted ml-2">/ {container}</span>
        {/if}
      </div>
    </div>
    {#if onClose}
      <button
        onclick={onClose}
        class="p-1 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    {/if}
  </div>

  <!-- Terminal Content -->
  <div bind:this={terminalContainer} class="flex-1 p-2"></div>
</div>

<style>
  :global(.xterm) {
    height: 100%;
  }
  :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
  :global(.xterm-viewport::-webkit-scrollbar) {
    width: 8px;
  }
  :global(.xterm-viewport::-webkit-scrollbar-track) {
    background: #111111;
  }
  :global(.xterm-viewport::-webkit-scrollbar-thumb) {
    background: #333333;
    border-radius: 4px;
  }
</style>
