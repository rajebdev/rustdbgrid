<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let value = "";
  export let column = null;
  export let rowIndex = null;

  function close() {
    dispatch("close");
  }

  function save() {
    dispatch("save", { value, column, rowIndex });
  }

  function handleKeydown(event) {
    if (event.key === "Escape") {
      close();
    } else if (event.key === "Enter" && event.ctrlKey) {
      event.preventDefault();
      save();
    }
  }

  // Auto-focus action for textarea
  function focusInput(node) {
    node.focus();
    node.select();
  }
</script>

{#if show && column !== null && rowIndex !== null}
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
            <i class="fas fa-edit"></i> Edit Cell Value
          </h5>
        </div>

        <div class="modal-body">
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
            use:focusInput
          ></textarea>
          <div class="form-text mt-2">
            <i class="fas fa-info-circle"></i> Press Ctrl+Enter to save, Escape to
            cancel
          </div>
        </div>

        <div class="modal-footer">
          <button type="button" class="btn btn-primary" on:click={save}>
            <i class="fas fa-check"></i> Save
          </button>
          <button type="button" class="btn btn-secondary" on:click={close}>
            <i class="fas fa-times"></i> Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
