<script>
  export let tableSchema;
  export let pgConstraints;
  export let isPostgres = false;
  export let loadingPgData = false;
</script>

<div class="columns-grid">
  <table class="schema-table">
    <thead>
      <tr>
        {#if isPostgres}
          <th class="col-name">Name</th>
          <th class="col-type">Attribute</th>
          <th class="col-type">Owner</th>
          <th class="col-type">Type</th>
          <th class="col-comment">Expression</th>
          <th class="col-comment">Comment</th>
        {:else}
          <th class="col-name">Constraint Name</th>
          <th class="col-type">Type</th>
          <th class="col-type">Columns</th>
          <th class="col-comment">Details</th>
        {/if}
      </tr>
    </thead>
    <tbody>
      {#if isPostgres}
        {#if loadingPgData}
          <tr>
            <td colspan="6" class="no-data">
              <i class="fas fa-spinner fa-spin"></i> Loading constraints...
            </td>
          </tr>
        {:else if pgConstraints && pgConstraints.length > 0}
          {#each pgConstraints as constraint}
            <tr>
              <td class="cell-name">{constraint.name}</td>
              <td class="cell-type">{constraint.attribute}</td>
              <td class="cell-type">{constraint.owner || "N/A"}</td>
              <td class="cell-type">{constraint.constraint_type}</td>
              <td class="cell-comment">{constraint.expression || ""}</td>
              <td class="cell-comment">{constraint.comment || ""}</td>
            </tr>
          {/each}
        {:else}
          <tr>
            <td colspan="6" class="no-data">No constraints found</td>
          </tr>
        {/if}
      {:else if tableSchema?.indexes && tableSchema.indexes.length > 0}
        {#each tableSchema.indexes.filter((idx) => idx.name === "PRIMARY" || idx.is_unique) as constraint}
          <tr>
            <td class="cell-name">{constraint.name}</td>
            <td class="cell-type"
              >{constraint.name === "PRIMARY" ? "PRIMARY KEY" : "UNIQUE"}</td
            >
            <td class="cell-type">{constraint.columns.join(", ")}</td>
            <td class="cell-comment"
              >{constraint.is_unique
                ? "Unique constraint"
                : "Primary key constraint"}</td
            >
          </tr>
        {/each}
      {:else}
        <tr>
          <td colspan="4" class="no-data">No constraints found</td>
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

  .cell-name,
  .cell-type,
  .cell-comment {
    word-break: break-word;
  }

  .no-data {
    text-align: center;
    color: var(--text-secondary);
    padding: 24px !important;
  }
</style>
