<script>
  export let activeTab;
</script>

<div class="empty-state">
  <i class="fas fa-info-circle"></i>
  <span>{activeTab} view is not yet implemented</span>
</div>

<style>
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    color: var(--text-secondary);
  }

  .empty-state i {
    font-size: 32px;
  }
</style>
