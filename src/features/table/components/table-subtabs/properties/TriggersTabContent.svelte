<script>
  export let tableTriggers;
  export let loadingTriggers = false;
</script>

<div class="columns-grid">
  <table class="schema-table">
    <thead>
      <tr>
        <th class="col-name">Name</th>
        <th class="col-type">Timing</th>
        <th class="col-type">Type</th>
        <th class="col-type">Table</th>
        <th class="col-comment">Description</th>
      </tr>
    </thead>
    <tbody>
      {#if loadingTriggers}
        <tr>
          <td colspan="5" class="no-data">
            <i class="fas fa-spinner fa-spin"></i> Loading triggers...
          </td>
        </tr>
      {:else if tableTriggers && tableTriggers.length > 0}
        {#each tableTriggers as trigger}
          <tr>
            <td class="cell-name">{trigger.name}</td>
            <td class="cell-type">{trigger.timing}</td>
            <td class="cell-type">{trigger.event}</td>
            <td class="cell-type">{trigger.table_name}</td>
            <td class="cell-comment" title={trigger.description}>
              {trigger.description
                ? trigger.description.length > 50
                  ? trigger.description.substring(0, 50) + "..."
                  : trigger.description
                : "N/A"}
            </td>
          </tr>
        {/each}
      {:else}
        <tr>
          <td colspan="5" class="no-data">No triggers found for this table</td>
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
