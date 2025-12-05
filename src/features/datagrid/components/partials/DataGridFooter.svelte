<script>
  export let totalRows = 0;
  export let displayRowsLength = 0;
  export let columnCount = 0;
  export let executionTime = "0";
  export let displayData = null;
  export let onCancelChanges = null;

  let refreshDropdownOpen = false;
  let saveDropdownOpen = false;
  let paginateLimit = 100;
  let fetchTime = "0.0";
  let fetchTimeFetch = "0.0";
  let lastFetchTime = new Date().toLocaleString();
  let dropdownContainer;

  const handleRefresh = (type) => {
    console.log("Refresh:", type);
    closeDropdowns();
  };

  const handleSave = (type) => {
    console.log("Save:", type);
    closeDropdowns();
  };

  const handleEditCell = () => {
    console.log("Edit Cell");
    closeDropdowns();
  };

  const handleAddRow = () => {
    console.log("Add Row");
    closeDropdowns();
  };

  const handleDuplicateRow = () => {
    console.log("Duplicate Row");
    closeDropdowns();
  };

  const handleDeleteRow = () => {
    console.log("Delete Row");
    closeDropdowns();
  };

  const handleCancel = () => {
    if (onCancelChanges) {
      onCancelChanges();
    }
    closeDropdowns();
  };

  const handlePaginateLimitChange = (e) => {
    if (e.key === "Enter") {
      console.log("Paginate limit:", paginateLimit);
    }
  };

  const handleClickOutside = (e) => {
    if (dropdownContainer && !dropdownContainer.contains(e.target)) {
      refreshDropdownOpen = false;
      saveDropdownOpen = false;
    }
  };

  const closeDropdowns = () => {
    refreshDropdownOpen = false;
    saveDropdownOpen = false;
  };
</script>

<svelte:window on:click={handleClickOutside} />

{#if displayData}
  <div
    class="sticky-bottom data-footer border-top shadow-sm"
    style="position: sticky; bottom: 0; z-index: 20;"
  >
    <div
      class="d-flex align-items-center justify-content-between gap-3 p-2"
      bind:this={dropdownContainer}
    >
      <!-- Left Toolbar -->
      <div class="d-flex align-items-center gap-2">
        <!-- Refresh Button -->
        <div class="dropdown-up">
          {#if refreshDropdownOpen}
            <div class="dropdown-menu-up show">
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("instant")}
              >
                <i class="fas fa-bolt"></i> Instant
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("1s")}
              >
                Every 1s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("5s")}
              >
                Every 5s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("10s")}
              >
                Every 10s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("15s")}
              >
                Every 15s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("30s")}
              >
                Every 30s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("60s")}
              >
                Every 60s
              </button>
              <div class="dropdown-divider"></div>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("custom")}
              >
                <i class="fas fa-cog"></i> Custom
              </button>
            </div>
          {/if}
          <div class="btn-group-refresh">
            <button
              class="btn btn-sm btn-outline-primary d-flex align-items-center gap-2"
              title="Refresh Data"
              on:click={() => handleRefresh("instant")}
            >
              <i class="fas fa-sync-alt"></i>
              <span>Refresh</span>
            </button>
            <button
              class="btn btn-sm btn-outline-primary"
              on:click={() => {
                refreshDropdownOpen = !refreshDropdownOpen;
                if (refreshDropdownOpen) saveDropdownOpen = false;
              }}
            >
              <i class="fas fa-chevron-up" style="font-size: 0.65rem;"></i>
            </button>
          </div>
        </div>

        <!-- Save Button -->
        <div class="dropdown-up">
          {#if saveDropdownOpen}
            <div class="dropdown-menu-up show">
              <button
                class="dropdown-item"
                on:click={() => handleSave("generateScript")}
              >
                <i class="fas fa-file-code"></i> Generate Script
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleSave("instant")}
              >
                <i class="fas fa-bolt"></i> Instant
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleSave("instantConfirm")}
              >
                <i class="fas fa-check-circle"></i> Instant with Confirmation
              </button>
            </div>
          {/if}
          <div class="btn-group-save">
            <button
              class="btn btn-sm btn-outline-success d-flex align-items-center gap-2"
              title="Save Data"
              on:click={() => handleSave("instant")}
            >
              <i class="fas fa-save"></i>
              <span>Save</span>
            </button>
            <button
              class="btn btn-sm btn-outline-success"
              on:click={() => {
                saveDropdownOpen = !saveDropdownOpen;
                if (saveDropdownOpen) refreshDropdownOpen = false;
              }}
            >
              <i class="fas fa-chevron-up" style="font-size: 0.65rem;"></i>
            </button>
          </div>
        </div>

        <!-- Cancel Button -->
        <button
          class="btn btn-sm btn-outline-danger d-flex align-items-center gap-2"
          title="Cancel Changes"
          on:click={handleCancel}
        >
          <i class="fas fa-times"></i>
          <span>Cancel</span>
        </button>

        <div class="vr"></div>

        <!-- Edit Cell -->
        <button
          class="btn btn-sm btn-outline-secondary"
          title="Edit Cell"
          on:click={handleEditCell}
        >
          <i class="fas fa-edit"></i>
        </button>

        <!-- Add Row -->
        <button
          class="btn btn-sm btn-outline-secondary"
          title="Add Row"
          on:click={handleAddRow}
        >
          <i class="fas fa-plus"></i>
        </button>

        <!-- Duplicate Row -->
        <button
          class="btn btn-sm btn-outline-secondary"
          title="Duplicate Row"
          on:click={handleDuplicateRow}
        >
          <i class="fas fa-copy"></i>
        </button>

        <!-- Delete Row -->
        <button
          class="btn btn-sm btn-outline-danger"
          title="Delete Current Row"
          on:click={handleDeleteRow}
        >
          <i class="fas fa-trash"></i>
        </button>

        <div class="vr"></div>

        <!-- Paginate Limit -->
        <div class="d-flex align-items-center gap-2">
          <label for="paginate-limit" class="mb-0" style="font-size: 0.85rem;"
            >Limit:</label
          >
          <input
            id="paginate-limit"
            type="number"
            class="form-control form-control-sm"
            style="width: 70px;"
            bind:value={paginateLimit}
            on:keydown={handlePaginateLimitChange}
            min="1"
          />
        </div>
      </div>

      <!-- Right Detail Info -->
      <div class="text-muted" style="font-size: 0.85rem;">
        {displayRowsLength.toLocaleString()} rows fetched - {fetchTime}ms ({fetchTimeFetch}ms
        fetch), pada {lastFetchTime}
      </div>
    </div>
  </div>
{/if}

<style>
  .data-footer {
    background: var(--bg-tertiary);
  }

  .dropdown-up {
    position: relative;
  }

  .btn-group-refresh,
  .btn-group-save {
    display: flex;
    gap: 0;
  }

  .btn-group-refresh .btn:first-child,
  .btn-group-save .btn:first-child {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
    border-right: none;
  }

  .btn-group-refresh .btn:last-child,
  .btn-group-save .btn:last-child {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
  }

  .dropdown-menu-up {
    position: absolute;
    bottom: 100%;
    left: 0;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    margin-bottom: 2px;
    min-width: 150px;
    box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.15);
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: 0.5rem 1rem;
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    font-size: 0.875rem;
    color: inherit;
  }

  .dropdown-item:hover {
    background: var(--bg-tertiary);
  }

  .dropdown-divider {
    margin: 0.5rem 0;
    border-top: 1px solid var(--border-color);
  }
</style>
