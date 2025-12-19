<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { yaml } from '@codemirror/lang-yaml';
  import { keymap } from '@codemirror/view';
  import { indentWithTab } from '@codemirror/commands';

  interface Props {
    content: string;
    readonly?: boolean;
    onchange?: (value: string) => void;
  }

  let { content, readonly = true, onchange }: Props = $props();

  let editorContainer: HTMLDivElement;
  let editorView: EditorView | null = null;

  // Dark theme for CodeMirror matching our app
  const darkTheme = EditorView.theme({
    '&': {
      backgroundColor: '#0a0a0a',
      color: '#e4e4e7',
      height: '100%',
    },
    '.cm-content': {
      fontFamily: 'Hack, Menlo, Monaco, "Courier New", monospace',
      fontSize: '12px',
      caretColor: '#00d4aa',
    },
    '.cm-cursor': {
      borderLeftColor: '#00d4aa',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: '#00d4aa',
    },
    '.cm-gutters': {
      backgroundColor: '#111111',
      color: '#555555',
      border: 'none',
      borderRight: '1px solid #222222',
    },
    '.cm-activeLineGutter': {
      backgroundColor: '#1a1a1a',
    },
    '.cm-activeLine': {
      backgroundColor: '#1a1a1a40',
    },
    '.cm-selectionBackground': {
      backgroundColor: '#00d4aa30 !important',
    },
    '&.cm-focused .cm-selectionBackground': {
      backgroundColor: '#00d4aa30 !important',
    },
    '.cm-matchingBracket': {
      backgroundColor: '#00d4aa40',
      outline: 'none',
    },
    // YAML syntax colors
    '.cm-string': { color: '#22c55e' },
    '.cm-number': { color: '#f59e0b' },
    '.cm-keyword': { color: '#00d4aa' },
    '.cm-comment': { color: '#555555' },
    '.cm-propertyName': { color: '#3b82f6' },
    '.cm-punctuation': { color: '#888888' },
    '.cm-atom': { color: '#a855f7' },
    '.cm-meta': { color: '#ef4444' },
    // Scrollbar styling
    '.cm-scroller': {
      overflow: 'auto',
    },
    '.cm-scroller::-webkit-scrollbar': {
      width: '8px',
      height: '8px',
    },
    '.cm-scroller::-webkit-scrollbar-track': {
      background: '#111111',
    },
    '.cm-scroller::-webkit-scrollbar-thumb': {
      background: '#333333',
      borderRadius: '4px',
    },
    '.cm-scroller::-webkit-scrollbar-thumb:hover': {
      background: '#444444',
    },
  }, { dark: true });

  onMount(() => {
    const extensions = [
      basicSetup,
      yaml(),
      darkTheme,
      keymap.of([indentWithTab]),
      EditorView.lineWrapping,
    ];

    if (readonly) {
      extensions.push(EditorState.readOnly.of(true));
    } else if (onchange) {
      extensions.push(EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          onchange(update.state.doc.toString());
        }
      }));
    }

    editorView = new EditorView({
      state: EditorState.create({
        doc: content || '',
        extensions,
      }),
      parent: editorContainer,
    });
  });

  onDestroy(() => {
    editorView?.destroy();
  });

  // Update content when prop changes
  $effect(() => {
    if (editorView && content !== undefined) {
      const currentContent = editorView.state.doc.toString();
      if (content !== currentContent) {
        editorView.dispatch({
          changes: {
            from: 0,
            to: currentContent.length,
            insert: content,
          },
        });
      }
    }
  });
</script>

<div bind:this={editorContainer} class="h-full w-full overflow-hidden"></div>

<style>
  :global(.cm-editor) {
    height: 100%;
  }
  :global(.cm-scroller) {
    font-family: Hack, Menlo, Monaco, 'Courier New', monospace !important;
  }
</style>
