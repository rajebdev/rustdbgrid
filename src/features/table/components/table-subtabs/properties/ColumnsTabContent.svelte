<script>
  import {
    getDataTypeDisplay,
    getColumnKey,
  } from "../../../../utils/propertiesFormatters";

  export let tableSchema;
</script>

<div class="columns-grid">
  <table class="schema-table">
    <thead>
      <tr>
        <th class="col-icon"></th>
        <th class="col-name">Column Name</th>
        <th class="col-number">#</th>
        <th class="col-type">Data Type</th>
        <th class="col-check">Not Null</th>
        <th class="col-check">Auto Increment</th>
        <th class="col-check">Key</th>
        <th class="col-default">Default</th>
        <th class="col-extra">Extra</th>
        <th class="col-expression">Expression</th>
        <th class="col-comment">Comment</th>
      </tr>
    </thead>
    <tbody>
      {#if tableSchema?.columns && tableSchema.columns.length > 0}
        {#each tableSchema.columns as column, index}
          <tr>
            <td class="cell-icon">
              {#if column.is_primary_key}
                <i class="fas fa-key key-icon" title="Primary Key"></i>
              {:else}
                <i class="fas fa-circle default-icon"></i>
              {/if}
            </td>
            <td class="cell-name">{column.name}</td>
            <td class="cell-number">{index + 1}</td>
            <td class="cell-type">{getDataTypeDisplay(column)}</td>
            <td class="cell-check">
              <input
                type="checkbox"
                checked={!column.nullable}
                readonly
                style="pointer-events: none;"
              />
            </td>
            <td class="cell-check">
              <input
                type="checkbox"
                checked={column.is_auto_increment}
                readonly
                style="pointer-events: none;"
              />
            </td>
            <td class="cell-default">{getColumnKey(column)}</td>
            <td class="cell-default">{column.default_value || ""}</td>
            <td class="cell-extra"
              >{column.is_auto_increment ? "auto_increment" : ""}</td
            >
            <td class="cell-expression"></td>
            <td class="cell-comment"></td>
          </tr>
        {/each}
      {:else}
        <tr>
          <td colspan="11" class="no-data">No columns found</td>
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

  .col-icon {
    width: 30px;
    text-align: center;
  }

  .col-number {
    width: 40px;
    text-align: center;
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

  .col-default,
  .col-extra,
  .col-expression {
    min-width: 100px;
  }

  .col-comment {
    min-width: 150px;
  }

  .cell-icon {
    text-align: center;
  }

  .cell-number {
    text-align: center;
    color: var(--text-secondary);
  }

  .cell-check {
    text-align: center;
  }

  .key-icon {
    color: #ffd700;
  }

  .default-icon {
    color: var(--text-secondary);
    font-size: 6px;
  }

  .no-data {
    text-align: center;
    color: var(--text-secondary);
    padding: 24px !important;
  }
</style>
