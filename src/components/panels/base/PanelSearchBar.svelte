<script>
  export let value = "";
  export let placeholder = "Search...";
  export let actionButton = null; // { icon, title, onClick, variant }
</script>

<input
  type="text"
  class="form-control form-control-sm flex-grow-1"
  {placeholder}
  bind:value
/>

{#if actionButton}
  <button
    class="btn btn-sm btn-{actionButton.variant || 'outline-secondary'}"
    on:click={actionButton.onClick}
    title={actionButton.title || ""}
  >
    <i class={actionButton.icon}></i>
  </button>
{/if}
