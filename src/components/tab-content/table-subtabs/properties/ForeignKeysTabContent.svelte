<script>
  export let tableSchema;
  export let pgForeignKeys;
  export let tableInfo;
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
          <th class="col-type">Reference Column</th>
          <th class="col-type">Associated Entity</th>
          <th class="col-type">Match Type</th>
          <th class="col-type">Delete Rule</th>
          <th class="col-type">Update Rule</th>
          <th class="col-comment">Comment</th>
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
            <td colspan="10" class="no-data">
              <i class="fas fa-spinner fa-spin"></i> Loading foreign keys...
            </td>
          </tr>
        {:else if pgForeignKeys && pgForeignKeys.length > 0}
          {#each pgForeignKeys as fk}
            <tr>
              <td class="cell-name">{fk.name}</td>
              <td class="cell-type">{fk.attribute}</td>
              <td class="cell-type">{fk.owner || "N/A"}</td>
              <td class="cell-type">{fk.fk_type}</td>
              <td class="cell-type">{fk.reference_column}</td>
              <td class="cell-type">{fk.associated_entity}</td>
              <td class="cell-type">{fk.match_type || "SIMPLE"}</td>
              <td class="cell-type">{fk.delete_rule || "NO ACTION"}</td>
              <td class="cell-type">{fk.update_rule || "NO ACTION"}</td>
              <td class="cell-comment">{fk.comment || ""}</td>
            </tr>
          {/each}
        {:else}
          <tr>
            <td colspan="10" class="no-data">No foreign keys found</td>
          </tr>
        {/if}
      {:else if tableSchema?.foreign_keys && tableSchema.foreign_keys.length > 0}
        {#each tableSchema.foreign_keys as fk}
          <tr>
            <td class="cell-name">{fk.name}</td>
            <td class="cell-type">{fk.column}</td>
            <td class="cell-type">{fk.owner || tableInfo?.database || "N/A"}</td
            >
            <td class="cell-type">{fk.referenced_table}</td>
            <td class="cell-type">{fk.referenced_column}</td>
            <td class="cell-type">{fk.ref_object_type || "TABLE"}</td>
            <td class="cell-type"
              >{fk.on_delete ? fk.on_delete.toUpperCase() : "NO ACTION"}</td
            >
            <td class="cell-type"
              >{fk.on_update ? fk.on_update.toUpperCase() : "NO ACTION"}</td
            >
          </tr>
        {/each}
      {:else}
        <tr>
          <td colspan="8" class="no-data">No foreign keys found</td>
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
