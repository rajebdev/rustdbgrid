<script>
  import * as monaco from "monaco-editor";
  import BaseModal from "../../../shared/components/base/BaseModal.svelte";
  import { autoFormatQuery } from "../../../shared/utils/editor/queryFormatter";
  import { getMonacoTheme } from "../../settings/services/themeService";
  import { activeTheme } from "../../settings/stores/theme.js";
  import { DatabaseType } from "../../../core/config/databaseTypes";

  export let show = false;
  export let query = "";
  export let databaseType = null;

  export let onClose = null;
  export let onSave = null;

  let editorContainer;
  let editor;
  let editableQuery = "";

  // Create editor ketika modal ditampilkan
  $: if (show && editorContainer && !editor) {
    console.log("[QueryEditorModal] Creating editor...");
    const theme = getMonacoTheme($activeTheme);
    let language = "sql";
    if (databaseType === DatabaseType.MONGODB) {
      language = "json";
    } else if (databaseType === DatabaseType.REDIS) {
      language = "redis";
    }

    editor = monaco.editor.create(editorContainer, {
      value: "",
      language,
      theme,
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
    console.log("[QueryEditorModal] Editor created successfully");

    // Listen for changes
    editor.onDidChangeModelContent(() => {
      editableQuery = editor.getValue();
    });
  }

  // Update query saat modal dibuka
  $: if (show && editor && query) {
    console.log("[QueryEditorModal] Updating editor with formatted query");
    editableQuery = query;
    // Auto-format query saat modal dibuka
    if (databaseType) {
      editableQuery = autoFormatQuery(query, databaseType);
      console.log("[QueryEditorModal] Query formatted for", databaseType);
    }
    editor.setValue(editableQuery);
    console.log("[QueryEditorModal] Editor value updated");
  }

  // Cleanup editor saat modal ditutup
  $: if (!show && editor) {
    console.log("[QueryEditorModal] Disposing editor");
    editor.dispose();
    editor = null;
  }

  $: if (editor && $activeTheme) {
    const theme = getMonacoTheme($activeTheme);
    monaco.editor.setTheme(theme);
  }

  function handleClose() {
    if (onClose) {
      onClose();
    }
  }

  function handleSave() {
    if (onSave) {
      onSave(editableQuery);
    }
  }
</script>

<BaseModal {show} size="lg" on:close={handleClose}>
  <svelte:fragment slot="header">
    <h5 class="mb-0"><i class="fas fa-edit text-warning"></i> Edit Query</h5>
  </svelte:fragment>

  <svelte:fragment slot="body">
    <div class="editor-container" bind:this={editorContainer}></div>
  </svelte:fragment>

  <svelte:fragment slot="footer">
    <button type="button" class="btn btn-secondary" on:click={handleClose}>
      Cancel
    </button>
    <button type="button" class="btn btn-primary" on:click={handleSave}>
      Save Query
    </button>
  </svelte:fragment>
</BaseModal>

<style>
  .editor-container {
    height: 400px;
    border: 1px solid var(--border-color);
    border-radius: 0.25rem;
  }
</style>
