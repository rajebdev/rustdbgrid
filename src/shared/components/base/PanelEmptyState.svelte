<script>
  export let message = "No items";
  export let icon = "fa-inbox";
  export let isSearchResult = false;
</script>

<div class="panel-empty-state">
  <div class="empty-content">
    {#if icon}
      <i class="fas {icon} empty-icon"></i>
    {/if}
    <p class="text-muted text-center mb-0">
      {message}
    </p>
    {#if isSearchResult}
      <small class="text-muted">Try a different search term</small>
    {/if}
  </div>
</div>

<style>
  .panel-empty-state {
    padding: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 120px;
  }

  .empty-content {
    text-align: center;
  }

  .empty-icon {
    font-size: 32px;
    color: var(--text-muted);
    opacity: 0.3;
    margin-bottom: 12px;
    display: block;
  }
</style>
