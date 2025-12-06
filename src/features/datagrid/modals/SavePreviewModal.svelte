<script>
  import BaseModal from "../../../shared/components/base/BaseModal.svelte";
  import { generateSql, saveData } from "../../../core/integrations/tauri.js";

  export let isOpen = false;
  export let connectionId = null;
  export let database = null;
  export let table = null;
  export let schema = null;
  export let newRows = new Map();
  export let editedRows = new Map();
  export let deletedRows = new Set();
  export let displayRows = [];
  export let originalRowData = new Map(); // Add this prop
  export let columns = []; // Column definitions for converting array rows to objects
  export let onClose = null;
  export let onSaveSuccess = null;

  let previewQuery = "";
  let isLoading = false;
  let isSaving = false;
  let saveResult = null;
  let error = null;
  let copySuccess = false;

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(previewQuery);
      copySuccess = true;
      setTimeout(() => {
        copySuccess = false;
      }, 2000);
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
    }
  }

  function buildSaveRequest(
    newRows,
    editedRows,
    deletedRows,
    displayRows,
    originalRowData,
    columns
  ) {
    // Helper function to convert array row to object using column names
    function rowToObject(row) {
      if (Array.isArray(row)) {
        const obj = {};
        columns.forEach((col, index) => {
          const colName = typeof col === "object" ? col.name : col;
          obj[colName] = row[index];
        });
        return obj;
      }
      return row;
    }

    // Transform edited_rows from Map<rowIndex, Map<column, newValue>>
    // to Array<{original_data, updated_data}>
    const edited_rows_array = [];

    editedRows.forEach((columnsMap, rowIndex) => {
      if (columnsMap && columnsMap.size > 0) {
        const original = originalRowData.get(rowIndex) || {};
        // Only include columns that were actually changed
        const updated = {};
        columnsMap.forEach((newValue, column) => {
          updated[column] = newValue;
        });

        console.log(`[buildSaveRequest] Row ${rowIndex}:`, {
          original,
          changedColumns: Array.from(columnsMap.keys()),
          updated,
        });

        edited_rows_array.push({
          original_data: original,
          updated_data: updated,
        });
      }
    });

    // Convert new rows from Map values to array, converting array rows to objects
    const new_rows_array = Array.from(newRows.values()).map((row) =>
      rowToObject(row)
    );

    console.log("[buildSaveRequest] New rows:", new_rows_array);

    // Extract column names in order for backend
    const columnNames = columns.map((col) =>
      typeof col === "object" ? col.name : col
    );

    // Convert deleted rows from Set to array, converting array rows to objects if needed
    const deleted_rows_array = Array.from(deletedRows).map((row) => {
      const converted = rowToObject(row);
      console.log("[buildSaveRequest] Deleted row conversion:", {
        original: row,
        converted,
      });
      return converted;
    });
    console.log("[buildSaveRequest] Deleted rows (final):", deleted_rows_array);

    return {
      column_names: columnNames, // Send column order to backend
      new_rows: new_rows_array,
      edited_rows: edited_rows_array, // Now properly formatted
      deleted_rows: deleted_rows_array,
    };
  }

  const handleGeneratePreview = async () => {
    if (!isOpen) return;
    isLoading = true;
    error = null;
    previewQuery = "";
    saveResult = null;

    try {
      const saveRequest = buildSaveRequest(
        newRows,
        editedRows,
        deletedRows,
        displayRows,
        originalRowData,
        columns
      );
      console.log("📝 Generating SQL preview...", {
        newRows: saveRequest.new_rows.length,
        editedRows: saveRequest.edited_rows.length,
        deletedRows: saveRequest.deleted_rows.length,
      });

      const query = await generateSql({
        connectionId,
        database,
        table,
        schema,
        saveRequest,
      });
      previewQuery = query;
    } catch (err) {
      error = `Failed to generate SQL: ${err.message || err}`;
      console.error("❌ Error:", error);
    } finally {
      isLoading = false;
    }
  };

  const handleExecute = async () => {
    if (!previewQuery) return;
    isSaving = true;
    error = null;
    saveResult = null;

    try {
      const saveRequest = buildSaveRequest(
        newRows,
        editedRows,
        deletedRows,
        displayRows,
        originalRowData,
        columns
      );
      console.log("💾 Executing save...", {
        newRows: saveRequest.new_rows.length,
        editedRows: saveRequest.edited_rows.length,
        deletedRows: saveRequest.deleted_rows.length,
      });

      const response = await saveData({
        connectionId,
        database,
        table,
        schema,
        saveRequest,
      });
      saveResult = response;

      if (response.status === "success") {
        console.log("✅ Save successful!", response);
        if (onSaveSuccess) {
          onSaveSuccess(response);
        }
      } else if (response.status === "partial") {
        console.warn("⚠️ Partial save:", response);
      } else {
        error = response.message;
      }
    } catch (err) {
      error = `Failed to save data: ${err.message || err}`;
      console.error("❌ Error:", error);
    } finally {
      isSaving = false;
    }
  };

  const handleClose = () => {
    isOpen = false;
    previewQuery = "";
    saveResult = null;
    error = null;
    if (onClose) {
      onClose();
    }
  };

  $: if (isOpen && !previewQuery && !saveResult) {
    handleGeneratePreview();
  }
</script>

<BaseModal
  show={isOpen}
  size="lg"
  centered={true}
  backdrop={true}
  keyboard={true}
  on:close={handleClose}
>
  <svelte:fragment slot="header">
    <h5 class="modal-title">
      {#if saveResult}
        Save Result
      {:else}
        Preview Changes
      {/if}
    </h5>
  </svelte:fragment>

  <svelte:fragment slot="body">
    {#if error}
      <div class="alert alert-danger alert-dismissible fade show" role="alert">
        <i class="fas fa-exclamation-circle"></i>
        <strong>Error:</strong>
        {error}
      </div>
    {/if}

    {#if isLoading && !previewQuery}
      <div class="text-center py-5">
        <div class="spinner-border text-primary" role="status">
          <span class="visually-hidden">Loading...</span>
        </div>
        <p class="mt-3 text-muted">Generating SQL...</p>
      </div>
    {:else if saveResult}
      <div>
        <div
          class="alert"
          class:alert-success={saveResult.status === "success"}
          class:alert-warning={saveResult.status === "partial"}
          class:alert-danger={saveResult.status === "error"}
        >
          <i
            class="fas"
            class:fa-check-circle={saveResult.status === "success"}
            class:fa-exclamation-triangle={saveResult.status === "partial"}
            class:fa-times-circle={saveResult.status === "error"}
          ></i>
          <strong>{saveResult.message}</strong>
        </div>

        <div class="row">
          <div class="col-md-6">
            <div class="card mb-3">
              <div class="card-body">
                <h6 class="card-title">
                  <i class="fas fa-info-circle"></i> Statistics
                </h6>
                <dl class="row mb-0">
                  <dt class="col-6">Affected Rows:</dt>
                  <dd class="col-6">{saveResult.affected_rows}</dd>
                  <dt class="col-6">Queries Executed:</dt>
                  <dd class="col-6">{saveResult.executed_queries.length}</dd>
                  {#if saveResult.errors && saveResult.errors.length > 0}
                    <dt class="col-6">Errors:</dt>
                    <dd class="col-6 text-danger">
                      {saveResult.errors.length}
                    </dd>
                  {/if}
                </dl>
              </div>
            </div>
          </div>

          <div class="col-md-6">
            <div class="card mb-3">
              <div class="card-body">
                <h6 class="card-title">
                  <i class="fas fa-database"></i> Status
                </h6>
                <p class="mb-0">
                  <span
                    class="badge"
                    class:bg-success={saveResult.status === "success"}
                    class:bg-warning={saveResult.status === "partial"}
                    class:bg-danger={saveResult.status === "error"}
                  >
                    {saveResult.status.toUpperCase()}
                  </span>
                </p>
              </div>
            </div>
          </div>
        </div>

        {#if saveResult.executed_queries && saveResult.executed_queries.length > 0}
          <div class="mt-3">
            <h6>Executed Queries:</h6>
            <div
              class="bg-dark p-3 rounded"
              style="max-height: 300px; overflow-y: auto;"
            >
              <code class="text-light">
                {saveResult.executed_queries.join("\n")}
              </code>
            </div>
          </div>
        {/if}

        {#if saveResult.errors && saveResult.errors.length > 0}
          <div class="mt-3">
            <h6 class="text-danger">Errors:</h6>
            <ul class="list-unstyled">
              {#each saveResult.errors as err}
                <li class="text-danger small">
                  <i class="fas fa-times-circle"></i>
                  {err}
                </li>
              {/each}
            </ul>
          </div>
        {/if}
      </div>
    {:else if previewQuery}
      <div>
        <div class="alert alert-info">
          <i class="fas fa-info-circle"></i>
          Review the generated SQL before executing. Click
          <strong>Execute</strong> to apply changes to the database.
        </div>

        <div>
          <p class="text-muted" style="font-size: 0.85rem;">
            <strong>Changes Summary:</strong>
            <span class="badge bg-primary me-2">
              +{newRows.size} New
            </span>
            <span class="badge bg-warning me-2">
              ~{editedRows.size} Edited
            </span>
            <span class="badge bg-danger">
              -{deletedRows.size} Deleted
            </span>
          </p>
        </div>

        <p class="text-muted" style="font-size: 0.85rem; margin-top: 1rem;">
          Generated SQL:
        </p>
        <div
          class="bg-dark p-3 rounded"
          style="max-height: 400px; overflow-y: auto;"
        >
          <code
            class="save-preview text-light font-monospace"
            style="white-space: pre-wrap; word-break: break-word;"
          >
            {previewQuery}
          </code>
        </div>
      </div>
    {/if}
  </svelte:fragment>

  <svelte:fragment slot="footer">
    <button
      type="button"
      class="btn btn-info me-2"
      on:click={copyToClipboard}
      disabled={!previewQuery}
    >
      <i class="fas fa-copy"></i>
      {copySuccess ? "Copied!" : "Copy Query"}
    </button>
    <button
      type="button"
      class="btn btn-secondary"
      on:click={handleClose}
      disabled={isSaving}
    >
      {saveResult ? "Close" : "Cancel"}
    </button>

    {#if !saveResult}
      <button
        type="button"
        class="btn btn-primary"
        on:click={handleExecute}
        disabled={!previewQuery || isLoading || isSaving}
      >
        {#if isSaving}
          <span class="spinner-border spinner-border-sm me-2"></span>
          Saving...
        {:else}
          <i class="fas fa-check"></i>
          Execute
        {/if}
      </button>
    {/if}
  </svelte:fragment>
</BaseModal>

<style>
  code {
    font-size: 0.85rem;
    line-height: 1.4;
  }

  .bg-dark {
    background-color: #1e1e1e !important;
  }

  .text-light {
    color: #e0e0e0 !important;
  }
</style>
