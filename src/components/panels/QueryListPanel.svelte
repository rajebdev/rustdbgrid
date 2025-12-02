<script>
  import { queryListStore } from "../../stores/queryList";
  import BasePanel from "./base/BasePanel.svelte";
  import PanelSearchBar from "./base/PanelSearchBar.svelte";
  import PanelEmptyState from "./base/PanelEmptyState.svelte";
  import { useSearch } from "../../composables/useSearch";
  import {
    formatDate,
    truncateText,
    showConfirmation,
    dispatchLoadQuery,
  } from "../../utils/panelHelpers";

  const { searchTerm, filteredItems: filteredQueries } = useSearch(
    queryListStore,
    ["title", "content"]
  );

  let editingId = null;
  let editTitle = "";
  let showNewForm = false;
  let newTitle = "";

  function handleEdit(query) {
    editingId = query.id;
    editTitle = query.title;
  }

  function handleSaveEdit() {
    if (editTitle.trim()) {
      queryListStore.updateQuery(editingId, { title: editTitle });
      editingId = null;
      editTitle = "";
    }
  }

  function handleCancelEdit() {
    editingId = null;
    editTitle = "";
  }

  function handleDelete(id) {
    if (showConfirmation("Delete this query?")) {
      queryListStore.deleteQuery(id);
    }
  }

  function handleAddNew() {
    if (newTitle.trim()) {
      queryListStore.addQuery(newTitle, "", "");
      newTitle = "";
      showNewForm = false;
    }
  }
</script>

<BasePanel title="Saved Queries" icon="fas fa-star">
  <div slot="toolbar">
    <PanelSearchBar
      bind:value={$searchTerm}
      placeholder="Search queries..."
      actionButton={{
        icon: "fas fa-plus",
        variant: "primary",
        title: "Add new query",
        onClick: () => (showNewForm = !showNewForm),
      }}
    />
  </div>

  {#if showNewForm}
    <div class="new-form p-2 border-bottom">
      <input
        type="text"
        class="form-control form-control-sm mb-2"
        placeholder="Query name..."
        bind:value={newTitle}
      />
      <div class="d-flex gap-1">
        <button
          class="btn btn-sm btn-success flex-grow-1"
          on:click={handleAddNew}
          disabled={!newTitle.trim()}
        >
          Save
        </button>
        <button
          class="btn btn-sm btn-outline-secondary flex-grow-1"
          on:click={() => (showNewForm = false)}
        >
          Cancel
        </button>
      </div>
    </div>
  {/if}

  {#if $filteredQueries.length === 0}
    <PanelEmptyState
      message={$queryListStore.length === 0
        ? "No saved queries"
        : "No matching queries"}
      icon="fa-star"
      isSearchResult={$queryListStore.length > 0}
    />
  {:else}
    {#each $filteredQueries as query (query.id)}
      <div class="panel-item">
        {#if editingId === query.id}
          <div class="edit-form">
            <input
              type="text"
              class="form-control form-control-sm mb-2"
              bind:value={editTitle}
            />
            <div class="d-flex gap-1">
              <button
                class="btn btn-sm btn-success flex-grow-1"
                on:click={handleSaveEdit}
              >
                Save
              </button>
              <button
                class="btn btn-sm btn-outline-secondary flex-grow-1"
                on:click={handleCancelEdit}
              >
                Cancel
              </button>
            </div>
          </div>
        {:else}
          <div class="item-header">
            <button
              class="item-title-button"
              on:dblclick={() => handleEdit(query)}
            >
              {query.title}
            </button>
            <div class="item-actions">
              <button
                on:click={() => dispatchLoadQuery(query.content)}
                title="Load to editor"
              >
                <i class="fas fa-arrow-left"></i>
              </button>
              <button on:click={() => handleEdit(query)} title="Edit">
                <i class="fas fa-pencil"></i>
              </button>
              <button
                class="text-danger"
                on:click={() => handleDelete(query.id)}
                title="Delete"
              >
                <i class="fas fa-trash"></i>
              </button>
            </div>
          </div>
          <div class="item-content">
            {truncateText(query.content, 60)}
          </div>
          <div class="item-meta">
            <small class="text-muted">
              {formatDate(query.createdAt)}
            </small>
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</BasePanel>

<style>
  .new-form {
    background: var(--accent-blue-light);
  }

  .edit-form {
    padding: 4px;
  }
</style>
