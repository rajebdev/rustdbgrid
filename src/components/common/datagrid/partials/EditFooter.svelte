<script>
  export let hasUnsavedEdits = false;
  export let editedRowsSize = 0;

  export let onSaveChanges = null;
  export let onCancelChanges = null;
</script>

{#if hasUnsavedEdits}
  <div
    class="sticky-bottom bg-warning border-top shadow-sm"
    style="position: sticky; bottom: 0; z-index: 11;"
  >
    <div class="d-flex align-items-center justify-content-between gap-2 p-2">
      <div class="d-flex align-items-center gap-2">
        <i class="fas fa-exclamation-triangle text-danger"></i>
        <span class="fw-semibold text-danger">
          You have unsaved changes ({editedRowsSize} row{editedRowsSize > 1
            ? "s"
            : ""} modified)
        </span>
      </div>
      <div class="d-flex gap-2">
        <button
          class="btn btn-sm btn-success"
          on:click={() => onSaveChanges?.()}
        >
          <i class="fas fa-save"></i> Save Changes
        </button>
        <button
          class="btn btn-sm btn-secondary"
          on:click={() => onCancelChanges?.()}
        >
          <i class="fas fa-times"></i> Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
