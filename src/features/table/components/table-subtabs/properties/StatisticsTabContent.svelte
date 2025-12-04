<script>
  import {
    formatBytes,
    formatNumber,
  } from "../../../../../shared/utils/data/propertiesFormatters";

  export let tableStatistics;
  export let tableSchema;
  export let isMssql = false;
  export let isPostgres = false;
</script>

<div class="columns-grid">
  <table class="schema-table">
    <thead>
      <tr>
        <th class="col-name">Name</th>
        <th class="col-type">Value</th>
      </tr>
    </thead>
    <tbody>
      {#if isMssql}
        <!-- MSSQL specific statistics -->
        <tr>
          <td class="cell-name">Table size</td>
          <td class="cell-type">
            {tableStatistics?.table_size ? tableStatistics.table_size : "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Row Count</td>
          <td class="cell-type">
            {tableStatistics?.row_count
              ? formatNumber(tableStatistics.row_count)
              : "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Pages</td>
          <td class="cell-type">
            {tableStatistics?.pages
              ? formatNumber(tableStatistics.pages)
              : "N/A"}
          </td>
        </tr>
      {:else if isPostgres}
        <!-- PostgreSQL specific statistics -->
        <tr>
          <td class="cell-name">Row Count Estimate</td>
          <td class="cell-type">
            {tableStatistics?.row_count
              ? formatNumber(tableStatistics.row_count)
              : "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Disk Space</td>
          <td class="cell-type">
            {tableStatistics?.table_size ? tableStatistics.table_size : "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Rel Size</td>
          <td class="cell-type">
            {tableStatistics?.data_length
              ? formatBytes(tableStatistics.data_length)
              : "N/A"}
          </td>
        </tr>
      {:else}
        <!-- MySQL and other databases statistics -->
        <tr>
          <td class="cell-name">Row Count</td>
          <td class="cell-type">
            {tableStatistics?.row_count
              ? formatNumber(tableStatistics.row_count)
              : "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Avg Row Length</td>
          <td class="cell-type">
            {tableStatistics?.avg_row_length
              ? formatNumber(tableStatistics.avg_row_length)
              : "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Data Length</td>
          <td class="cell-type">
            {tableStatistics?.data_length
              ? formatBytes(tableStatistics.data_length)
              : "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Max data length</td>
          <td class="cell-type">
            {tableStatistics?.max_data_length
              ? formatBytes(tableStatistics.max_data_length)
              : "0"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Data free</td>
          <td class="cell-type">
            {tableStatistics?.data_free
              ? formatBytes(tableStatistics.data_free)
              : "0"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Index length</td>
          <td class="cell-type">
            {tableStatistics?.index_length
              ? formatBytes(tableStatistics.index_length)
              : "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Row format</td>
          <td class="cell-type">{tableStatistics?.row_format || "N/A"}</td>
        </tr>
        <tr>
          <td class="cell-name">Create Time</td>
          <td class="cell-type">{tableStatistics?.create_time || "N/A"}</td>
        </tr>
        <tr>
          <td class="cell-name">Update time</td>
          <td class="cell-type">{tableStatistics?.update_time || "N/A"}</td>
        </tr>
        <tr>
          <td class="cell-name">Check time</td>
          <td class="cell-type">{tableStatistics?.check_time || "N/A"}</td>
        </tr>
        <tr>
          <td class="cell-name">Collation</td>
          <td class="cell-type">
            {tableStatistics?.collation || tableSchema?.collation || "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Checksum</td>
          <td class="cell-type">{tableStatistics?.checksum || "N/A"}</td>
        </tr>
        <tr>
          <td class="cell-name">Engine</td>
          <td class="cell-type">
            {tableStatistics?.engine || tableSchema?.engine || "N/A"}
          </td>
        </tr>
        <tr>
          <td class="cell-name">Comment</td>
          <td class="cell-type">{tableStatistics?.comment || ""}</td>
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
</style>
