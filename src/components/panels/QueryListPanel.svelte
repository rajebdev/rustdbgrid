<script>
  import { queryListStore } from "../../stores/queryList";

  let searchTerm = "";
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
    if (confirm("Delete this query?")) {
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

  function handleCopyToEditor(query) {
    // Dispatch event to parent/editor
    window.dispatchEvent(
      new CustomEvent("load-query", { detail: { query: query.content } })
    );
  }

  $: filteredQueries = $queryListStore.filter(
    (q) =>
      q.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
      q.content.toLowerCase().includes(searchTerm.toLowerCase())
  );
</script>

<div class="query-list-panel">
  <div class="panel-header">
    <h6 class="mb-0">
      <i class="fas fa-star"></i> Saved Queries
    </h6>
  </div>

  <div class="panel-toolbar">
    <input
      type="text"
      class="form-control form-control-sm"
      placeholder="Search queries..."
      bind:value={searchTerm}
    />
    <button
      class="btn btn-sm btn-primary"
      on:click={() => (showNewForm = !showNewForm)}
      title="Add new query"
    >
      <i class="fas fa-plus"></i>
    </button>
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

  <div class="queries-list">
    {#if filteredQueries.length === 0}
      <div class="empty-state">
        <p class="text-muted text-center mb-0">No saved queries</p>
      </div>
    {:else}
      {#each filteredQueries as query (query.id)}
        <div class="query-item">
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
            <div class="query-header">
              <button class="query-title" on:dblclick={() => handleEdit(query)}>
                {query.title}
              </button>
              <div class="query-actions">
                <button
                  class="btn btn-sm btn-link p-0"
                  on:click={() => handleCopyToEditor(query)}
                  title="Load to editor"
                >
                  <i class="fas fa-arrow-left"></i>
                </button>
                <button
                  class="btn btn-sm btn-link p-0"
                  on:click={() => handleEdit(query)}
                  title="Edit"
                >
                  <i class="fas fa-pencil"></i>
                </button>
                <button
                  class="btn btn-sm btn-link p-0 text-danger"
                  on:click={() => handleDelete(query.id)}
                  title="Delete"
                >
                  <i class="fas fa-trash"></i>
                </button>
              </div>
            </div>
            <div class="query-preview">
              {query.content.substring(0, 60)}...
            </div>
            <div class="query-meta">
              <small class="text-muted">
                {new Date(query.createdAt).toLocaleDateString()}
              </small>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .query-list-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
  }

  .panel-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-tertiary);
  }

  .panel-header h6 {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .panel-toolbar {
    display: flex;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
  }

  .panel-toolbar input {
    flex: 1;
  }

  .queries-list {
    flex: 1;
    overflow-y: auto;
  }

  .empty-state {
    padding: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .query-item {
    padding: 8px;
    border-bottom: 1px solid var(--border-light);
    transition: background-color 0.15s ease;
  }

  .query-item:hover {
    background-color: var(--hover-bg);
  }

  .query-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 4px;
  }

  .query-title {
    flex: 1;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
    word-break: break-word;
    background: none;
    border: none;
    padding: 0;
    text-align: left;
  }

  .query-title:hover {
    text-decoration: underline;
  }

  .query-actions {
    display: flex;
    gap: 2px;
  }

  .query-actions button {
    color: var(--text-muted);
    font-size: 10px;
  }

  .query-actions button:hover {
    color: var(--text-primary);
  }

  .query-preview {
    font-size: 11px;
    color: var(--text-secondary);
    margin-top: 4px;
    padding: 4px;
    background: var(--bg-primary);
    border-radius: 2px;
    max-height: 40px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .query-meta {
    margin-top: 4px;
    font-size: 10px;
  }

  .edit-form {
    padding: 4px;
  }

  .new-form {
    background: var(--accent-blue-light);
  }
</style>
