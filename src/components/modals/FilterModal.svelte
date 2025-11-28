<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let show = false;
  export let column = null;
  export let position = { top: 0, left: 0 };
  export let selectedValues = new Set();
  export let availableValues = [];
  export let loading = false;
  export let searchQuery = "";

  function close() {
    dispatch("close");
  }

  function apply() {
    dispatch("apply", { column, selectedValues });
  }

  function clear() {
    dispatch("clear", { column });
  }

  function toggleValue(value) {
    if (selectedValues.has(value)) {
      selectedValues.delete(value);
    } else {
      selectedValues.add(value);
    }
    selectedValues = new Set(selectedValues);
    dispatch("selectionChange", { column, selectedValues });
  }

  function selectAll() {
    selectedValues = new Set(availableValues);
    dispatch("selectionChange", { column, selectedValues });
  }

  function deselectAll() {
    selectedValues = new Set();
    dispatch("selectionChange", { column, selectedValues });
  }

  function handleSearchInput(event) {
    dispatch("search", { query: event.target.value });
  }
</script>

{#if show && column}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop show" on:click={close}></div>
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal d-block" tabindex="-1" on:click={close}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="modal-dialog"
      style="position: fixed; top: {position.top}px; left: {position.left}px; margin: 0;"
      on:click|stopPropagation
    >
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">
            <i class="fas fa-filter"></i> Filter: {column}
          </h5>
        </div>

        <div class="modal-body">
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

          <div
            class="border rounded"
            style="max-height: 300px; overflow-y: auto;"
          >
            {#if loading}
              <div
                class="d-flex flex-column align-items-center justify-content-center p-4 text-primary"
              >
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

        <div class="modal-footer">
          <button class="btn btn-primary" on:click={apply}>
            <i class="fas fa-check"></i> Apply
          </button>
          <button class="btn btn-danger" on:click={clear}>
            <i class="fas fa-eraser"></i> Clear
          </button>
          <button class="btn btn-secondary" on:click={close}> Cancel </button>
        </div>
      </div>
    </div>
  </div>
{/if}
