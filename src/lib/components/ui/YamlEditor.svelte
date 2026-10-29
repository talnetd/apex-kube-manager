<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState, Compartment, type Extension } from '@codemirror/state';
  import { yaml } from '@codemirror/lang-yaml';
  import { keymap } from '@codemirror/view';
  import { indentWithTab } from '@codemirror/commands';
  import { resolvedTheme } from "../../stores/theme";
  // Transitive deps of `codemirror` (already in the lockfile).
  import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
  import { tags as t } from '@lezer/highlight';

  // Token colours from the theme, so YAML reads on black and white alike
  // (the default highlight style is light-only: navy keys on black).
  const yamlHighlight = HighlightStyle.define([
    { tag: [t.propertyName, t.definition(t.propertyName)], color: 'var(--ui-accent-text)' },
    { tag: [t.string, t.special(t.string)], color: 'var(--ui-success)' },
    { tag: [t.number, t.bool, t.null, t.atom], color: 'var(--ui-warning)' },
    { tag: [t.keyword, t.typeName, t.labelName], color: 'var(--ui-accent-text)' },
    { tag: [t.comment, t.lineComment, t.blockComment], color: 'var(--ui-muted)', fontStyle: 'italic' },
    { tag: [t.punctuation, t.separator, t.bracket, t.operator], color: 'var(--ui-muted)' },
    { tag: t.meta, color: 'var(--ui-danger)' },
  ]);

  interface Props {
    content: string;
    readonly?: boolean;
    onchange?: (value: string) => void;
  }

  let { content, readonly = true, onchange }: Props = $props();

  let editorContainer: HTMLDivElement;
  let editorView: EditorView | null = null;

  // Get CodeMirror theme based on current theme
  function getEditorTheme(): Extension {
	const isDark = $resolvedTheme !== "light";
	return EditorView.theme({
    '&': {
      backgroundColor: 'var(--ui-app)',
	  color: 'var(--ui-primary)',
      height: '100%',
    },
    '.cm-content': {
      fontFamily: 'var(--font-mono)',
      fontSize: '12px',
      caretColor: 'var(--ui-accent)',
    },
    '.cm-cursor': {
      borderLeftColor: 'var(--ui-accent)',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: 'var(--ui-accent)',
    },
    '.cm-gutters': {
      backgroundColor: 'var(--ui-app)',
	  color: 'var(--ui-faint)',
      border: 'none',
	  borderRight: '1px solid var(--ui-line)',
    },
    '.cm-activeLineGutter': {
	  backgroundColor: 'var(--ui-hover)',
    },
    '.cm-activeLine': {
      backgroundColor: 'var(--ui-hover)',
    },
    '.cm-selectionBackground': {
      backgroundColor: 'color-mix(in srgb, var(--ui-accent) 30%, transparent) !important',
    },
    '&.cm-focused .cm-selectionBackground': {
      backgroundColor: 'color-mix(in srgb, var(--ui-accent) 30%, transparent) !important',
    },
    '.cm-matchingBracket': {
      backgroundColor: 'var(--ui-active)',
      outline: 'none',
    },
    // YAML syntax colors
    '.cm-string': { color: 'var(--ui-success)' },
    '.cm-number': { color: 'var(--ui-warning)' },
    '.cm-keyword': { color: 'var(--ui-accent-text)' },
    '.cm-comment': { color: 'var(--ui-faint)' },
    '.cm-propertyName': { color: 'var(--ui-accent-text)' },
    '.cm-punctuation': { color: 'var(--ui-muted)' },
    '.cm-atom': { color: 'var(--ui-primary)' },
    '.cm-meta': { color: 'var(--ui-danger)' },
    // Scrollbar styling
    '.cm-scroller': {
      overflow: 'auto',
    },
    '.cm-scroller::-webkit-scrollbar': {
      width: '8px',
      height: '8px',
    },
    '.cm-scroller::-webkit-scrollbar-track': {
      background: isDark ? '#111111' : '#f5f5f5',
    },
    '.cm-scroller::-webkit-scrollbar-thumb': {
      background: isDark ? '#333333' : '#d0d0d0',
      borderRadius: '4px',
    },
    '.cm-scroller::-webkit-scrollbar-thumb:hover': {
      background: isDark ? '#444444' : '#b0b0b0',
    },
  }, { dark: isDark });
  }

  // Read-only and theme live in compartments so they change in place. The
  // old version decided read-only once at mount (so Edit never allowed
  // typing) and called a non-existent EditorView.reconfigure on theme change.
  const readOnlyC = new Compartment();
  const themeC = new Compartment();

  onMount(() => {
    editorView = new EditorView({
      state: EditorState.create({
        doc: content || '',
        extensions: [
          basicSetup,
          yaml(),
          syntaxHighlighting(yamlHighlight),
          keymap.of([indentWithTab]),
          EditorView.lineWrapping,
          themeC.of(getEditorTheme()),
          readOnlyC.of(EditorState.readOnly.of(readonly)),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) onchange?.(update.state.doc.toString());
          }),
        ],
      }),
      parent: editorContainer,
    });
  });

  onDestroy(() => {
    editorView?.destroy();
  });

  // Edit / view toggles in place, keeping the text and cursor.
  $effect(() => {
    const ro = readonly;
    editorView?.dispatch({ effects: readOnlyC.reconfigure(EditorState.readOnly.of(ro)) });
    if (!ro) editorView?.focus();
  });

  // Theme follows light/dark.
  $effect(() => {
    void $resolvedTheme;
    editorView?.dispatch({ effects: themeC.reconfigure(getEditorTheme()) });
  });

  // A refreshed `content` replaces the text only while viewing, never over
  // the user's unsaved edits.
  $effect(() => {
    const next = content || '';
    if (!editorView || !readonly) return;
    const cur = editorView.state.doc.toString();
    if (cur !== next) editorView.dispatch({ changes: { from: 0, to: cur.length, insert: next } });
  });
</script>

<div bind:this={editorContainer} class="h-full w-full overflow-hidden"></div>

<style>
  :global(.cm-editor) {
    height: 100%;
  }
  :global(.cm-scroller) {
    font-family: var(--font-mono) !important;
  }
</style>

