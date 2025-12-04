<script>
  import BaseModal from "../../../shared/components/base/BaseModal.svelte";
  import { autoFocus } from "../../../shared/composables/useModalFocus";

  export let show = false;
  export let value = "";
  export let column = null;
  export let rowIndex = null;

  function save() {
    if (column !== null && rowIndex !== null) {
      show = false;
    }
  }

  function handleKeydown(event) {
    if (event.key === "Enter" && event.ctrlKey) {
      event.preventDefault();
      save();
    }
  }
</script>

<BaseModal {show} size="lg" backdrop={true} on:close>
  <div slot="header" class="bg-primary text-white">
    <h5 class="modal-title">
      <i class="fas fa-edit"></i> Edit Cell Value
    </h5>
  </div>

  <div slot="body">
    <div class="mb-2">
      <strong>Column:</strong>
      <span class="badge bg-info">{column}</span>
      <strong class="ms-3">Row:</strong>
      <span class="badge bg-info">{rowIndex + 1}</span>
    </div>
    <textarea
      class="form-control font-monospace"
      rows="15"
      bind:value
      placeholder="Enter value..."
      style="resize: vertical; min-height: 200px;"
      on:keydown={handleKeydown}
      use:autoFocus
    ></textarea>
    <div class="form-text mt-2">
      <i class="fas fa-info-circle"></i> Press Ctrl+Enter to save, Escape to cancel
    </div>
  </div>

  <div slot="footer">
    <button type="button" class="btn btn-primary" on:click={save}>
      <i class="fas fa-check"></i> Save
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
