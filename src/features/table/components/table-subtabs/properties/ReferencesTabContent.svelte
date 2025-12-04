<script>
  export let tableReferences;
  export let pgReferences;
  export let isPostgres = false;
  export let loadingReferences = false;
  export let loadingPgData = false;
</script>

<div class="columns-grid">
  <table class="schema-table">
    <thead>
      <tr>
        {#if isPostgres}
          <th class="col-name">Name</th>
          <th class="col-type">Owner</th>
          <th class="col-type">Type</th>
          <th class="col-comment">Comment</th>
          <th class="col-type">Associated Entity</th>
          <th class="col-type">Sequence Num</th>
        {:else}
          <th class="col-name">Name</th>
          <th class="col-type">Column</th>
          <th class="col-type">Owner</th>
          <th class="col-type">Ref Table</th>
          <th class="col-type">Type</th>
          <th class="col-type">Ref Object</th>
          <th class="col-type">On Delete</th>
          <th class="col-type">On Update</th>
        {/if}
      </tr>
    </thead>
    <tbody>
      {#if isPostgres}
        {#if loadingPgData}
          <tr>
            <td colspan="6" class="no-data">
              <i class="fas fa-spinner fa-spin"></i> Loading references...
            </td>
          </tr>
        {:else if pgReferences && pgReferences.length > 0}
          {#each pgReferences as ref}
            <tr>
              <td class="cell-name">{ref.name}</td>
              <td class="cell-type">{ref.owner || "N/A"}</td>
              <td class="cell-type">{ref.ref_type}</td>
              <td class="cell-comment">{ref.comment || ""}</td>
              <td class="cell-type">{ref.associated_entity}</td>
              <td class="cell-type">{ref.sequence_num || "N/A"}</td>
            </tr>
          {/each}
        {:else}
          <tr>
            <td colspan="6" class="no-data">No references found</td>
          </tr>
        {/if}
      {:else if loadingReferences}
        <tr>
          <td colspan="8" class="no-data">
            <i class="fas fa-spinner fa-spin"></i> Loading references...
          </td>
        </tr>
      {:else if tableReferences && tableReferences.length > 0}
        {#each tableReferences.filter((ref) => ref.relationship_type === "REFERENCED_BY") as ref}
          <tr>
            <td class="cell-name">{ref.constraint_name}</td>
            <td class="cell-type">{ref.referenced_column_name}</td>
            <td class="cell-type">{ref.owner || "N/A"}</td>
            <td class="cell-type">{ref.table_name}</td>
            <td class="cell-type">{ref.column_name}</td>
            <td class="cell-type">{ref.ref_object_type || "TABLE"}</td>
            <td class="cell-type">{ref.on_delete || "NO ACTION"}</td>
            <td class="cell-type">{ref.on_update || "NO ACTION"}</td>
          </tr>
        {:else}
          <tr>
            <td colspan="8" class="no-data">No tables reference this table</td>
          </tr>
        {/each}
      {:else}
        <tr>
          <td colspan="8" class="no-data">No references found</td>
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
