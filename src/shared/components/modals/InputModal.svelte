<script>
  import BaseModal from "../base/BaseModal.svelte";
  import { autoFocus } from "../../composables/useModalFocus";

  export let show = false;
  export let title = "Input";
  export let label = "";
  export let value = "";
  export let placeholder = "";

  let inputValue = value;

  $: if (show) {
    inputValue = value;
  }

  function save() {
    if (inputValue && inputValue.trim()) {
      show = false;
    }
  }

  function handleKeydown(event) {
    if (event.key === "Enter") {
      event.preventDefault();
      save();
    }
  }
</script>

<BaseModal {show} on:close>
  <div slot="header" class="bg-primary text-white">
    <h5 class="modal-title">
      <i class="fas fa-edit"></i>
      {title}
    </h5>
  </div>

  <div slot="body">
    {#if label}
      <label for="input-field" class="form-label fw-semibold">{label}</label>
    {/if}
    <input
      id="input-field"
      type="text"
      class="form-control"
      bind:value={inputValue}
      {placeholder}
      on:keydown={handleKeydown}
      use:autoFocus
    />
    <div class="form-text mt-2">
      <i class="fas fa-info-circle"></i> Press Enter to save, Escape to cancel
    </div>
  </div>

  <div slot="footer">
    <button
      type="button"
      class="btn btn-primary"
      on:click={save}
      disabled={!inputValue || !inputValue.trim()}
    >
      <i class="fas fa-check"></i> OK
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
