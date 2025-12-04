<script>
  import BaseModal from "../base/BaseModal.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let tabTitle = "";

  function handleSave() {
    dispatch("save");
  }

  function handleDiscard() {
    dispatch("discard");
  }

  function handleCancel() {
    dispatch("cancel");
  }
</script>

<BaseModal {show} on:close={handleCancel}>
  <div slot="header">
    <h5 class="modal-title">
      <i class="fas fa-exclamation-triangle text-warning me-2"></i>
      Unsaved Changes
    </h5>
  </div>

  <div slot="body">
    <p class="mb-3">
      Do you want to save the changes you made to
      <strong>{tabTitle}</strong>?
    </p>
    <p class="text-muted small mb-0">
      Your changes will be lost if you don't save them.
    </p>
  </div>

  <div slot="footer">
    <button type="button" class="btn btn-secondary" on:click={handleCancel}>
      Cancel
    </button>
    <button type="button" class="btn btn-danger" on:click={handleDiscard}>
      Don't Save
    </button>
    <button type="button" class="btn btn-primary" on:click={handleSave}>
      Save
    </button>
  </div>
</BaseModal>
