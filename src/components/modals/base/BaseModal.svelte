<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let size = "md"; // 'sm', 'md', 'lg', 'xl'
  export let centered = true;
  export let scrollable = false;
  export let backdrop = true; // true = click to close, 'static' = no close on click
  export let keyboard = true; // Enable/disable ESC key to close
  export let showCloseButton = false; // Show X button in header

  $: dialogClasses = [
    "modal-dialog",
    size !== "md" ? `modal-${size}` : "",
    centered ? "modal-dialog-centered" : "",
    scrollable ? "modal-dialog-scrollable" : "",
  ]
    .filter(Boolean)
    .join(" ");

  function handleBackdropClick() {
    if (backdrop === true) {
      close();
    }
  }

  function handleKeydown(event) {
    if (keyboard && event.key === "Escape") {
      close();
    }
  }

  function close() {
    dispatch("close");
  }

  // Prevent modal dialog click from closing
  function stopPropagation(event) {
    event.stopPropagation();
  }
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop show" on:click={handleBackdropClick}></div>
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div
    class="modal d-block"
    tabindex="-1"
    role="dialog"
    on:keydown={handleKeydown}
  >
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class={dialogClasses} on:click={stopPropagation}>
      <div class="modal-content">
        {#if $$slots.header}
          <div class="modal-header">
            <slot name="header" />
            {#if showCloseButton}
              <button
                type="button"
                class="btn-close"
                aria-label="Close"
                on:click={close}
              ></button>
            {/if}
          </div>
        {/if}

        {#if $$slots.body}
          <div class="modal-body">
            <slot name="body" />
          </div>
        {/if}

        {#if $$slots.footer}
          <div class="modal-footer">
            <slot name="footer" />
          </div>
        {/if}

        <!-- Default slot for custom layout (no predefined structure) -->
        {#if !$$slots.header && !$$slots.body && !$$slots.footer}
          <slot />
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-content {
    background: var(--bg-modal);
    color: var(--text-primary);
  }

  .modal-header {
    border-bottom-color: var(--border-color);
  }

  .modal-footer {
    border-top-color: var(--border-color);
  }

  .modal-body {
    color: var(--text-primary);
  }

  /* Ensure backdrop is on top */
  .modal-backdrop {
    z-index: 1040;
  }

  .modal {
    z-index: 1050;
  }
</style>
