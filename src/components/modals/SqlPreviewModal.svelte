<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let pendingUpdates = [];
  export let previewSql = "";

  function close() {
    dispatch("close");
  }

  function execute() {
    dispatch("execute");
  }
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop show" on:click={close}></div>
  <div class="modal d-block" tabindex="-1">
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="modal-dialog modal-lg modal-dialog-centered"
      on:click|stopPropagation
    >
      <div class="modal-content">
        <div class="modal-header bg-primary text-white">
          <h5 class="modal-title">
            <i class="fas fa-code"></i> Preview SQL Update
          </h5>
        </div>

        <div class="modal-body">
          <div class="alert alert-info">
            <i class="fas fa-info-circle"></i>
            <strong>{pendingUpdates.length}</strong> update(s) will be executed:
          </div>

          <pre
            class="bg-dark text-light p-3 rounded"
            style="max-height: 400px; overflow-y: auto;"><code
              >{previewSql}</code
            ></pre>

          <div class="alert alert-warning mt-3">
            <i class="fas fa-exclamation-triangle"></i>
            <strong>Warning:</strong> This action cannot be undone. Please review
            the SQL carefully before executing.
          </div>
        </div>

        <div class="modal-footer">
          <button type="button" class="btn btn-success" on:click={execute}>
            <i class="fas fa-play"></i> Execute
          </button>
          <button type="button" class="btn btn-secondary" on:click={close}>
            <i class="fas fa-times"></i> Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
