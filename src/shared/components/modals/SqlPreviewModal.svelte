<script>
  import BaseModal from "../base/BaseModal.svelte";

  export let show = false;
  export let pendingUpdates = [];
  export let previewSql = "";

  function execute() {
    show = false;
  }
</script>

<BaseModal {show} size="lg" on:close>
  <div slot="header" class="bg-primary text-white">
    <h5 class="modal-title">
      <i class="fas fa-code"></i> Preview SQL Update
    </h5>
  </div>

  <div slot="body">
    <div class="alert alert-info">
      <i class="fas fa-info-circle"></i>
      <strong>{pendingUpdates.length}</strong> update(s) will be executed:
    </div>

    <pre
      class="sql-preview-code p-3 rounded"
      style="max-height: 400px; overflow-y: auto;"><code>{previewSql}</code
      ></pre>

    <div class="alert alert-warning mt-3">
      <i class="fas fa-exclamation-triangle"></i>
      <strong>Warning:</strong> This action cannot be undone. Please review the SQL
      carefully before executing.
    </div>
  </div>

  <div slot="footer">
    <button type="button" class="btn btn-success" on:click={execute}>
      <i class="fas fa-play"></i> Execute
    </button>
    <button
      type="button"
      class="btn btn-secondary"
      on:click={() => (show = false)}
    >
      <i class="fas fa-times"></i> Cancel
    </button>
  </div>
</BaseModal>

<style>
  .sql-preview-code {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.875rem;
  }
</style>
