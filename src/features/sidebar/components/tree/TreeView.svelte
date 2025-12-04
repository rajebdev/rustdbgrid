<script>
  export let searchable = false;
  export let searchPlaceholder = "Search...";
  export let title = "";
  export let searchQuery = "";

  function handleSearch(e) {
    searchQuery = e.target.value;
  }
</script>

<div class="tree-view">
  {#if title || searchable || $$slots.header}
    <div class="tree-header">
      {#if $$slots.header}
        <slot name="header"></slot>
      {:else}
        {#if title}
          <h6 class="tree-title">
            <slot name="title-icon"></slot>
            {title}
          </h6>
        {/if}

        {#if searchable || $$slots["header-actions"]}
          <div class="tree-search-row">
            {#if searchable}
              <input
                type="search"
                class="form-control form-control-sm flex-grow-1"
                placeholder={searchPlaceholder}
                value={searchQuery}
                on:input={handleSearch}
              />
            {/if}
            {#if $$slots["header-actions"]}
              <div class="tree-header-actions">
                <slot name="header-actions"></slot>
              </div>
            {/if}
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  <div class="tree-content" role="tree">
    <slot {searchQuery}></slot>
  </div>

  {#if $$slots.footer}
    <div class="tree-footer">
      <slot name="footer"></slot>
    </div>
  {/if}
</div>

<style>
  .tree-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .tree-header {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border-bottom: 1px solid var(--bs-border-color, #dee2e6);
    flex-shrink: 0;
  }

  .tree-title {
    margin: 0;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--bs-secondary-color, #6c757d);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .tree-search-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .tree-search-row :global(input) {
    font-size: 11px;
    height: 24px;
    padding: 2px 8px;
    flex: 1;
  }

  .tree-header-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .tree-header-actions :global(.btn) {
    font-size: 11px;
    padding: 2px 6px;
    height: 24px;
  }

  .tree-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 4px;
  }

  .tree-footer {
    padding: 8px;
    border-top: 1px solid var(--bs-border-color, #dee2e6);
    flex-shrink: 0;
  }
</style>
