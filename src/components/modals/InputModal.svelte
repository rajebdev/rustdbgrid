<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let title = "Input";
  export let label = "";
  export let value = "";
  export let placeholder = "";

  let inputValue = value;

  $: if (show) {
    inputValue = value;
  }

  function close() {
    dispatch("cancel");
  }

  function save() {
    if (inputValue && inputValue.trim()) {
      dispatch("submit", inputValue.trim());
    }
  }

  function handleKeydown(event) {
    if (event.key === "Escape") {
      close();
    } else if (event.key === "Enter") {
      event.preventDefault();
      save();
    }
  }

  // Auto-focus action for input
  function focusInput(node) {
    node.focus();
    node.select();
  }
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop show" on:click={close}></div>
  <div class="modal d-block" tabindex="-1">
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="modal-dialog modal-dialog-centered" on:click|stopPropagation>
      <div class="modal-content">
        <div class="modal-header bg-primary text-white">
          <h5 class="modal-title">
            <i class="fas fa-edit"></i>
            {title}
          </h5>
        </div>

        <div class="modal-body">
          {#if label}
            <label for="input-field" class="form-label fw-semibold"
              >{label}</label
            >
          {/if}
          <input
            id="input-field"
            type="text"
            class="form-control"
            bind:value={inputValue}
            {placeholder}
            on:keydown={handleKeydown}
            use:focusInput
          />
          <div class="form-text mt-2">
            <i class="fas fa-info-circle"></i> Press Enter to save, Escape to cancel
          </div>
        </div>

        <div class="modal-footer">
          <button
            type="button"
            class="btn btn-primary"
            on:click={save}
            disabled={!inputValue || !inputValue.trim()}
          >
            <i class="fas fa-check"></i> OK
          </button>
          <button type="button" class="btn btn-secondary" on:click={close}>
            <i class="fas fa-times"></i> Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
