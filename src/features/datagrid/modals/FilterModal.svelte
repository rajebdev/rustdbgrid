<script>
  import BaseModal from "../../../shared/components/base/BaseModal.svelte";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let column = null;
  export let selectedValues = new Set();
  export let availableValues = [];
  export let loading = false;
  export let searchQuery = "";
  export let position = { top: 0, left: 0 };

  function apply() {
    console.log("🔵 FilterModal apply clicked for column:", column);
    dispatch("apply", {
      column,
      selectedValues,
    });
    dispatch("close");
  }

  function clear() {
    console.log("🔵 FilterModal clear clicked for column:", column);
    dispatch("clear", {
      column,
    });
    dispatch("close");
  }

  function toggleValue(value) {
    if (selectedValues.has(value)) {
      selectedValues.delete(value);
    } else {
      selectedValues.add(value);
    }
    selectedValues = new Set(selectedValues);

    // Emit selection change event
    dispatch("selectionChange", {
      column,
      selectedValues,
    });
  }

  function selectAll() {
    selectedValues = new Set(availableValues);
  }

  function deselectAll() {
    selectedValues = new Set();
  }

  function handleSearchInput(event) {
    searchQuery = event.target.value;
    dispatch("search", {
      query: searchQuery,
    });
  }
</script>

{#if show && column}
  <BaseModal
    {show}
    backdrop={true}
    centered={false}
    keyboard={true}
    on:close
    style="--position-top: {position.top}px; --position-left: {position.left}px;"
  >
    <div slot="header">
      <h5 class="modal-title">
        <i class="fas fa-filter"></i> Filter: {column}
      </h5>
    </div>

    <div slot="body">
      <input
        type="text"
        class="form-control mb-3"
        placeholder="Search values..."
        value={searchQuery}
        on:input={handleSearchInput}
      />

      <div class="d-flex gap-2 mb-2">
        <button
          class="btn btn-sm btn-outline-primary flex-fill"
          on:click={selectAll}
        >
          <i class="fas fa-check-double"></i> Select All
        </button>
        <button
          class="btn btn-sm btn-outline-secondary flex-fill"
          on:click={deselectAll}
        >
          <i class="fas fa-times"></i> Deselect All
        </button>
      </div>

      <div class="border rounded" style="max-height: 300px; overflow-y: auto;">
        {#if loading}
          <div class="loading-state">
            <i class="fas fa-spinner fa-spin fa-2x mb-2"></i>
            <span>Loading values...</span>
          </div>
        {:else}
          <table class="table table-sm table-hover mb-0">
            <tbody>
              {#each availableValues as value}
                <tr>
                  <td class="text-center" style="width: 40px;">
                    <input
                      class="form-check-input"
                      type="checkbox"
                      id="filter-{value}"
                      checked={selectedValues.has(value)}
                      on:change={() => toggleValue(value)}
                    />
                  </td>
                  <td>
                    <label
                      class="form-check-label w-100 mb-0"
                      for="filter-{value}"
                      title={value}
                      style="cursor: pointer;"
                    >
                      {value}
                    </label>
                  </td>
                </tr>
              {:else}
                <tr>
                  <td colspan="2" class="text-center p-4 text-muted">
                    <i class="fas fa-info-circle fa-2x mb-2"></i>
                    <div>No values found</div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    </div>

    <div slot="footer">
      <button class="btn btn-primary" on:click={apply}>
        <i class="fas fa-check"></i> Apply
      </button>
      <button class="btn btn-danger" on:click={clear}>
        <i class="fas fa-eraser"></i> Clear
      </button>
      <button class="btn btn-secondary" on:click={() => dispatch("close")}>
        Cancel
      </button>
    </div>
  </BaseModal>
{/if}
