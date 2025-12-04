<script>
  export let tableSchema;
</script>

<div class="columns-grid">
  <table class="schema-table">
    <thead>
      <tr>
        <th class="col-name">Column Name</th>
        <th class="col-type">Data Type</th>
        <th class="col-comment">Expression</th>
      </tr>
    </thead>
    <tbody>
      {#if tableSchema?.columns}
        {#each tableSchema.columns.filter((col) => col.is_virtual || col.is_stored) as column}
          <tr>
            <td class="cell-name">{column.name}</td>
            <td class="cell-type">{column.data_type}</td>
            <td class="cell-comment">{column.generation_expression || "N/A"}</td
            >
          </tr>
        {:else}
          <tr>
            <td colspan="3" class="no-data"
              >No virtual or generated columns found</td
            >
          </tr>
        {/each}
      {:else}
        <tr>
          <td colspan="3" class="no-data">No virtual columns found</td>
        </tr>
      {/if}
    </tbody>
  </table>
</div>

<style>
  .columns-grid {
    height: 100%;
    overflow: auto;
  }

  .schema-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  .schema-table thead {
    position: sticky;
    top: 0;
    background: var(--bg-secondary);
    z-index: 1;
  }

  .schema-table th {
    padding: 6px 12px;
    text-align: left;
    font-weight: 600;
    color: var(--text-primary);
    border-bottom: 1px solid var(--border-color);
    white-space: nowrap;
    height: 28px;
  }

  .schema-table td {
    padding: 2px 12px;
    border-bottom: 1px solid var(--border-color);
    color: var(--text-primary);
    height: 24px;
    line-height: 1.5;
    font-size: 12px;
  }

  .schema-table tbody tr:hover {
    background: var(--bg-secondary);
  }

  .col-name {
    min-width: 150px;
  }

  .col-type {
    min-width: 120px;
  }

  .col-comment {
    min-width: 150px;
  }

  .no-data {
    text-align: center;
    color: var(--text-secondary);
    padding: 24px !important;
  }
</style>
