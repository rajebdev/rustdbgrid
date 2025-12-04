<script>
  import { formatNumber } from "../../../../../shared/utils/data/propertiesFormatters";

  export let pgPartitions;
  export let tableSchema;
  export let isPostgres = false;
  export let loadingPgData = false;
</script>

<div class="columns-grid">
  <table class="schema-table">
    <thead>
      <tr>
        {#if isPostgres}
          <th style="min-width: 150px;">Table Name</th>
          <th style="min-width: 100px;">Object ID</th>
          <th style="min-width: 100px;">Owner</th>
          <th style="min-width: 100px;">Tablespace</th>
          <th style="min-width: 120px;">Row Count Estimate</th>
          <th style="min-width: 120px; text-align: center;"
            >Has Row Level Security</th
          >
          <th style="min-width: 80px;">Partitions</th>
          <th style="min-width: 200px;">Partition By</th>
          <th style="min-width: 200px;">Partitions Expression</th>
          <th style="min-width: 150px;">Extra Options</th>
          <th style="min-width: 150px;">Comment</th>
        {:else}
          <th class="col-name">Partition Name</th>
          <th class="col-type">Method</th>
          <th class="col-type">Expression</th>
          <th class="col-comment">Rows</th>
          <th class="col-comment">Size</th>
        {/if}
      </tr>
    </thead>
    <tbody>
      {#if isPostgres}
        {#if loadingPgData}
          <tr>
            <td colspan="11" class="no-data">
              <i class="fas fa-spinner fa-spin"></i> Loading partitions...
            </td>
          </tr>
        {:else if pgPartitions && pgPartitions.length > 0}
          {#each pgPartitions as partition}
            <tr>
              <td class="cell-name">{partition.table_name}</td>
              <td class="cell-type">{partition.object_id || "N/A"}</td>
              <td class="cell-type">{partition.owner || "N/A"}</td>
              <td class="cell-type">{partition.tablespace || "default"}</td>
              <td class="cell-type">
                {partition.rowcount_estimate
                  ? formatNumber(partition.rowcount_estimate)
                  : "N/A"}
              </td>
              <td class="cell-check">
                <input
                  type="checkbox"
                  checked={partition.has_row_level_security}
                  readonly
                  style="pointer-events: none;"
                />
              </td>
              <td class="cell-type">{partition.partitions || 0}</td>
              <td class="cell-comment">{partition.partition_by || "N/A"}</td>
              <td class="cell-comment"
                >{partition.partitions_expression || "N/A"}</td
              >
              <td class="cell-comment">{partition.extra_options || "N/A"}</td>
              <td class="cell-comment">{partition.comment || ""}</td>
            </tr>
          {/each}
        {:else}
          <tr>
            <td colspan="11" class="no-data"
              >Table is not partitioned or no partitions found</td
            >
          </tr>
        {/if}
      {:else}
        <tr>
          <td colspan="5" class="no-data">
            {tableSchema?.partitioned
              ? "Partition data requires additional query"
              : "Table is not partitioned"}
          </td>
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
    word-break: break-word;
    white-space: normal;
    max-width: 300px;
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
