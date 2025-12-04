<script>
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";
  import { getMonacoTheme } from "../../../settings/services/themeService";

  export let value = "";
  export let theme = "vs-dark";
  export let completionProvider = null;

  export let onChange = null;
  export let onContextMenu = null;
  export let onReady = null;

  let editorContainer;
  let editor;
  let completionDisposable;

  $: if (editor && theme) {
    const monacoTheme = getMonacoTheme(theme);
    monaco.editor.setTheme(monacoTheme);
  }

  onMount(() => {
    const monacoTheme = getMonacoTheme(theme);

    editor = monaco.editor.create(editorContainer, {
      value,
      language: "sql",
      theme: monacoTheme,
      automaticLayout: true,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      wordWrap: "on",
      fontSize: 13,
      fontFamily: "'Fira Code', 'Monaco', monospace",
      folding: true,
      formatOnPaste: true,
      formatOnType: true,
      contextmenu: false,
      suggest: {
        shareSuggestSelections: true,
        showIcons: true,
      },
    });

    // Register completion provider if provided
    if (completionProvider) {
      completionDisposable = monaco.languages.registerCompletionItemProvider(
        "sql",
        completionProvider
      );
    }

    // Listen for changes
    editor.onDidChangeModelContent(() => {
      const text = editor.getValue();
      if (onChange) onChange(text);
    });

    // Context menu listener
    if (onContextMenu) {
      editorContainer.addEventListener("contextmenu", onContextMenu);
    }

    // Notify parent that editor is ready
    if (onReady) onReady(editor);
  });

  onDestroy(() => {
    if (completionDisposable) {
      completionDisposable.dispose();
    }
    if (editor) {
      editor.dispose();
    }
    if (onContextMenu) {
      editorContainer?.removeEventListener("contextmenu", onContextMenu);
    }
  });

  // Expose editor instance
  export function getEditor() {
    return editor;
  }

  export function setValue(newValue) {
    if (editor) {
      editor.setValue(newValue);
    }
  }

  export function getValue() {
    return editor ? editor.getValue() : "";
  }
</script>

<div bind:this={editorContainer} class="flex-grow-1 editor-wrapper"></div>

<style>
  .editor-wrapper {
    overflow: auto;
    position: relative;
    background: var(--editor-bg);
  }
</style>
