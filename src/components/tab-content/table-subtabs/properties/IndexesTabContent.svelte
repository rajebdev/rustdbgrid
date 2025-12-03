<script>
  export let tableSchema;
  export let pgIndexes;
  export let tableInfo;
  export let isPostgres = false;
  export let loadingPgData = false;
</script>

<div class="columns-grid">
  <table class="schema-table">
    <thead>
      <tr>
        {#if isPostgres}
          <th class="col-type">Column</th>
          <th class="col-name">Index Name</th>
          <th class="col-type">Table</th>
          <th class="col-check">Ascending</th>
          <th class="col-check">Nullable</th>
          <th class="col-check">Unique</th>
          <th class="col-type">Operator Class</th>
          <th class="col-comment">Predicate</th>
        {:else}
          <th class="col-name">Index Name</th>
          <th class="col-type">Column</th>
          <th class="col-type">Table</th>
          <th class="col-type">Index Type</th>
          <th class="col-check">Ascending</th>
          <th class="col-check">Nullable</th>
          <th class="col-check">Unique</th>
          <th class="col-comment">Extra</th>
        {/if}
      </tr>
    </thead>
    <tbody>
      {#if isPostgres}
        {#if loadingPgData}
          <tr>
            <td colspan="8" class="no-data">
              <i class="fas fa-spinner fa-spin"></i> Loading indexes...
            </td>
          </tr>
        {:else if pgIndexes && pgIndexes.length > 0}
          {#each pgIndexes as index}
            <tr>
              <td class="cell-type">{index.column}</td>
              <td class="cell-name">{index.idx_name}</td>
              <td class="cell-type">{index.table}</td>
              <td class="cell-check">
                <input
                  type="checkbox"
                  checked={index.ascending !== false}
                  readonly
                  style="pointer-events: none;"
                />
              </td>
              <td class="cell-check">
                <input
                  type="checkbox"
                  checked={index.nullable === true}
                  readonly
                  style="pointer-events: none;"
                />
              </td>
              <td class="cell-check">
                <input
                  type="checkbox"
                  checked={index.unique}
                  readonly
                  style="pointer-events: none;"
                />
              </td>
              <td class="cell-type">{index.operator_class || "N/A"}</td>
              <td class="cell-comment">{index.predicate || ""}</td>
            </tr>
          {/each}
        {:else}
          <tr>
            <td colspan="8" class="no-data">No indexes found</td>
          </tr>
        {/if}
      {:else if tableSchema?.indexes && tableSchema.indexes.length > 0}
        {#each tableSchema.indexes as index}
          <tr>
            <td class="cell-name">{index.name}</td>
            <td class="cell-type">{index.columns.join(", ")}</td>
            <td class="cell-type">{tableInfo?.name || "N/A"}</td>
            <td class="cell-type">{index.index_type || "BTREE"}</td>
            <td class="cell-check">
              <input
                type="checkbox"
                checked={index.ascending !== false}
                readonly
                style="pointer-events: none;"
              />
            </td>
            <td class="cell-check">
              <input
                type="checkbox"
                checked={index.nullable === true}
                readonly
                style="pointer-events: none;"
              />
            </td>
            <td class="cell-check">
              <input
                type="checkbox"
                checked={index.is_unique}
                readonly
                style="pointer-events: none;"
              />
            </td>
            <td class="cell-comment">{index.extra || ""}</td>
          </tr>
        {/each}
      {:else}
        <tr>
          <td colspan="8" class="no-data">No indexes found</td>
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

  .col-check {
    width: 100px;
    text-align: center;
  }

  .col-comment {
    min-width: 150px;
  }

  .cell-check {
    text-align: center;
  }

  .no-data {
    text-align: center;
    color: var(--text-secondary);
    padding: 24px !important;
  }
</style>
