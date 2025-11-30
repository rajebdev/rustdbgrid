<script>
  import { invoke } from "@tauri-apps/api/core";
  import {
    getTableSchema,
    getTableRelationships,
    getTriggers,
  } from "../../../utils/tauri";
  import { activeConnection } from "../../../stores/connections";
  import { tabDataStore } from "../../../stores/tabData";

  export let tabId;
  export let tableInfo;
  export let connection;

  let tableSchema = null;
  let tableStatistics = null;
  let tableReferences = null;
  let tableTriggers = null;
  let loading = true;
  let loadingReferences = false;
  let loadingTriggers = false;
  let error = null;

  // PostgreSQL-specific data
  let pgConstraints = null;
  let pgForeignKeys = null;
  let pgIndexes = null;
  let pgReferences = null;
  let pgPartitions = null;
  let loadingPgData = false;

  // Get database type
  $: conn = connection || $activeConnection;
  $: isMssql = conn?.db_type === "MSSQL";
  $: isPostgres = conn?.db_type === "PostgreSQL";

  // Define tabs based on database type
  $: tabs = isMssql
    ? [
        // MSSQL specific tabs
        { id: "Columns", label: "Columns", icon: "fas fa-columns" },
        { id: "Keys", label: "Keys", icon: "fas fa-key" },
        { id: "Foreign Keys", label: "Foreign Keys", icon: "fas fa-link" },
        { id: "Indexes", label: "Indexes", icon: "fas fa-sort-amount-down" },
        { id: "References", label: "References", icon: "fas fa-sitemap" },
        { id: "Triggers", label: "Triggers", icon: "fas fa-bolt" },
        { id: "Statistics", label: "Statistics", icon: "fas fa-chart-bar" },
        { id: "DDL", label: "DDL", icon: "fas fa-code" },
        { id: "Virtual", label: "Virtual", icon: "fas fa-cube" },
      ]
    : isPostgres
      ? [
          // PostgreSQL specific tabs
          { id: "Columns", label: "Columns", icon: "fas fa-columns" },
          { id: "Constraints", label: "Constraints", icon: "fas fa-lock" },
          { id: "Foreign Keys", label: "Foreign Keys", icon: "fas fa-key" },
          { id: "Indexes", label: "Indexes", icon: "fas fa-sort-amount-down" },
          {
            id: "Dependencies",
            label: "Dependencies",
            icon: "fas fa-project-diagram",
          },
          { id: "References", label: "References", icon: "fas fa-link" },
          { id: "Partitions", label: "Partitions", icon: "fas fa-th-large" },
          { id: "Child tables", label: "Child tables", icon: "fas fa-table" },
          { id: "Triggers", label: "Triggers", icon: "fas fa-bolt" },
          { id: "Rules", label: "Rules", icon: "fas fa-gavel" },
          { id: "Policies", label: "Policies", icon: "fas fa-shield-alt" },
          { id: "Statistics", label: "Statistics", icon: "fas fa-chart-bar" },
          { id: "Permissions", label: "Permissions", icon: "fas fa-user-lock" },
          { id: "DDL", label: "DDL", icon: "fas fa-code" },
          { id: "Virtual", label: "Virtual", icon: "fas fa-cube" },
        ]
      : [
          // MySQL and other databases (default)
          { id: "Columns", label: "Columns", icon: "fas fa-columns" },
          { id: "Constraints", label: "Constraints", icon: "fas fa-lock" },
          { id: "Foreign Keys", label: "Foreign Keys", icon: "fas fa-key" },
          { id: "References", label: "References", icon: "fas fa-link" },
          { id: "Triggers", label: "Triggers", icon: "fas fa-bolt" },
          { id: "Indexes", label: "Indexes", icon: "fas fa-sort-amount-down" },
          { id: "Partitions", label: "Partitions", icon: "fas fa-th-large" },
          { id: "Statistics", label: "Statistics", icon: "fas fa-chart-bar" },
          { id: "DDL", label: "DDL", icon: "fas fa-code" },
          { id: "Virtual", label: "Virtual", icon: "fas fa-cube" },
        ];

  // Subscribe to tabDataStore to get reactive updates
  $: tabData = $tabDataStore[tabId] || {};
  $: activeTab = tabData.activePropertiesTab || "Columns";

  function handlePropertiesTabChange(newTab) {
    tabDataStore.setActivePropertiesTab(tabId, newTab);
  }

  // Reactive statement to reload data when tableInfo changes
  $: if (tableInfo) {
    loadTableSchema();
    loadTableStatistics();
    loadTableReferences();
    loadTableTriggers();
    if (isPostgres) {
      loadPgData();
    }
  }

  async function loadPgData() {
    if (!isPostgres) return;

    try {
      loadingPgData = true;
      const conn = connection || $activeConnection;

      if (!conn || !tableInfo) {
        return;
      }

      // Build table identifier with schema for PostgreSQL
      let tableIdentifier = tableInfo.name;
      if (tableInfo.schema) {
        tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
      }

      // Load all PostgreSQL-specific data in parallel
      const [constraints, foreignKeys, indexes, references, partitions] =
        await Promise.all([
          invoke("get_pg_constraints", {
            config: conn,
            database: tableInfo.database,
            table: tableIdentifier,
          }),
          invoke("get_pg_foreign_keys", {
            config: conn,
            database: tableInfo.database,
            table: tableIdentifier,
          }),
          invoke("get_pg_indexes", {
            config: conn,
            database: tableInfo.database,
            table: tableIdentifier,
          }),
          invoke("get_pg_references", {
            config: conn,
            database: tableInfo.database,
            table: tableIdentifier,
          }),
          invoke("get_pg_partitions", {
            config: conn,
            database: tableInfo.database,
            table: tableIdentifier,
          }),
        ]);

      pgConstraints = constraints;
      pgForeignKeys = foreignKeys;
      pgIndexes = indexes;
      pgReferences = references;
      pgPartitions = partitions;

      console.log("PostgreSQL data loaded:", {
        constraints,
        foreignKeys,
        indexes,
        references,
        partitions,
      });
    } catch (e) {
      console.error("Error loading PostgreSQL data:", e);
      pgConstraints = [];
      pgForeignKeys = [];
      pgIndexes = [];
      pgReferences = [];
      pgPartitions = [];
    } finally {
      loadingPgData = false;
    }
  }

  async function loadTableSchema() {
    try {
      loading = true;
      error = null;
      const conn = connection || $activeConnection;

      if (!conn || !tableInfo) {
        error = "No connection or table information available";
        return;
      }

      // Build table identifier with schema for PostgreSQL and MSSQL
      let tableIdentifier = tableInfo.name;
      if (conn.db_type === "PostgreSQL" && tableInfo.schema) {
        tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
      } else if (conn.db_type === "MSSQL" && tableInfo.schema) {
        tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
      }

      // Call Tauri command to get table schema
      tableSchema = await getTableSchema(
        conn,
        tableInfo.database,
        tableIdentifier
      );
    } catch (e) {
      console.error("Error loading table schema:", e);
      error = e.message || "Failed to load table schema";
    } finally {
      loading = false;
    }
  }

  async function loadTableStatistics() {
    try {
      const conn = connection || $activeConnection;

      if (!conn || !tableInfo) {
        return;
      }

      // Build table identifier with schema for PostgreSQL and MSSQL
      let tableIdentifier = tableInfo.name;
      if (conn.db_type === "PostgreSQL" && tableInfo.schema) {
        tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
      } else if (conn.db_type === "MSSQL" && tableInfo.schema) {
        tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
      }

      // Call Tauri command to get table statistics
      tableStatistics = await invoke("get_table_statistics", {
        config: conn,
        database: tableInfo.database,
        table: tableIdentifier,
      });

      console.log("Table statistics loaded:", tableStatistics);
    } catch (e) {
      console.error("Error loading table statistics:", e);
      // Don't set error, just log it - statistics is optional
    }
  }

  async function loadTableReferences() {
    try {
      loadingReferences = true;
      const conn = connection || $activeConnection;

      if (!conn || !tableInfo) {
        tableReferences = [];
        return;
      }

      // Build table identifier with schema for PostgreSQL and MSSQL
      let tableIdentifier = tableInfo.name;
      if (conn.db_type === "PostgreSQL" && tableInfo.schema) {
        tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
      } else if (conn.db_type === "MSSQL" && tableInfo.schema) {
        tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
      }

      // Call Tauri command to get table relationships (references)
      tableReferences = await getTableRelationships(
        conn,
        tableInfo.database,
        tableIdentifier
      );

      console.log("Table references loaded:", tableReferences);
    } catch (e) {
      console.error("Error loading table references:", e);
      tableReferences = [];
      // Don't set error, just log it - references is optional
    } finally {
      loadingReferences = false;
    }
  }

  async function loadTableTriggers() {
    try {
      loadingTriggers = true;
      const conn = connection || $activeConnection;

      if (!conn || !tableInfo) {
        tableTriggers = [];
        return;
      }

      // Call Tauri command to get triggers for this database
      const allTriggers = await getTriggers(conn, tableInfo.database, null);

      // Filter triggers for this specific table
      tableTriggers = allTriggers.filter(
        (trigger) => trigger.table_name === tableInfo.name
      );

      console.log("Table triggers loaded:", tableTriggers);
    } catch (e) {
      console.error("Error loading table triggers:", e);
      tableTriggers = [];
      // Don't set error, just log it - triggers is optional
    } finally {
      loadingTriggers = false;
    }
  }

  function formatBytes(bytes) {
    if (bytes === null || bytes === undefined) return "N/A";
    if (bytes === 0) return "0";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  }

  function formatNumber(num) {
    if (num === null || num === undefined) return "N/A";
    return num.toLocaleString();
  }

  function getDataTypeDisplay(column) {
    return column.data_type || "unknown";
  }

  function getColumnKey(column) {
    if (column.is_primary_key) return "PRI";
    return "";
  }
</script>

<div class="properties-container">
  <!-- Table Info Header -->
  <div class="table-info-header">
    <div class="info-row">
      <span class="info-label">Table Name:</span>
      <span class="info-value">{tableInfo?.name || "N/A"}</span>
    </div>
    <div class="info-row">
      <span class="info-label">Engine:</span>
      <span class="info-value">{tableSchema?.engine || "InnoDB"}</span>
    </div>
    <div class="info-row">
      <span class="info-label">Collation:</span>
      <span class="info-value"
        >{tableSchema?.collation || "latin1_swedish_ci"}</span
      >
    </div>
    <div class="info-row">
      <label>
        <input
          type="checkbox"
          checked={tableSchema?.partitioned || false}
          readonly
          style="pointer-events: none;"
        />
        Partitioned
      </label>
    </div>
    <div class="info-row">
      <span class="info-label">Auto Increment:</span>
      <input
        type="text"
        class="info-input"
        value={tableSchema?.auto_increment || "11"}
      />
    </div>
  </div>

  <!-- Main content with sidebar -->
  <div class="properties-main">
    <!-- Left Sidebar with vertical tabs -->
    <div class="properties-sidebar">
      {#each tabs as tab}
        <button
          class="sidebar-tab-btn"
          class:active={activeTab === tab.id}
          on:click={() => handlePropertiesTabChange(tab.id)}
        >
          <i class={tab.icon}></i>
          <span>{tab.label}</span>
        </button>
      {/each}
    </div>

    <!-- Content Area -->
    <div class="properties-content">
      {#if loading}
        <div class="loading-state">
          <i class="fas fa-spinner fa-spin"></i>
          <span>Loading schema...</span>
        </div>
      {:else if error}
        <div class="error-state">
          <i class="fas fa-exclamation-triangle"></i>
          <span>{error}</span>
        </div>
      {:else if activeTab === "Columns"}
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
      {:else if activeTab === "Constraints"}
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
                      <td class="cell-comment">{constraint.expression || ""}</td
                      >
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
                      >{constraint.name === "PRIMARY"
                        ? "PRIMARY KEY"
                        : "UNIQUE"}</td
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
      {:else if activeTab === "References"}
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
                    <td colspan="8" class="no-data"
                      >No tables reference this table</td
                    >
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
      {:else if activeTab === "Triggers"}
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
              {#if tableTriggers && tableTriggers.length > 0}
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
                  <td colspan="5" class="no-data"
                    >No triggers found for this table</td
                  >
                </tr>
              {/if}
            </tbody>
          </table>
        </div>
      {:else if activeTab === "Partitions"}
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
                      <td class="cell-type"
                        >{partition.tablespace || "default"}</td
                      >
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
                      <td class="cell-comment"
                        >{partition.partition_by || "N/A"}</td
                      >
                      <td class="cell-comment"
                        >{partition.partitions_expression || "N/A"}</td
                      >
                      <td class="cell-comment"
                        >{partition.extra_options || "N/A"}</td
                      >
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
      {:else if activeTab === "Statistics"}
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
                    {tableStatistics?.table_size
                      ? tableStatistics.table_size
                      : "N/A"}
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
                    {tableStatistics?.table_size
                      ? tableStatistics.table_size
                      : "N/A"}
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
                  <td class="cell-type">
                    {tableStatistics?.row_format || "N/A"}
                  </td>
                </tr>
                <tr>
                  <td class="cell-name">Create Time</td>
                  <td class="cell-type">
                    {tableStatistics?.create_time || "N/A"}
                  </td>
                </tr>
                <tr>
                  <td class="cell-name">Update time</td>
                  <td class="cell-type">
                    {tableStatistics?.update_time || "N/A"}
                  </td>
                </tr>
                <tr>
                  <td class="cell-name">Check time</td>
                  <td class="cell-type">
                    {tableStatistics?.check_time || "N/A"}
                  </td>
                </tr>
                <tr>
                  <td class="cell-name">Collation</td>
                  <td class="cell-type">
                    {tableStatistics?.collation ||
                      tableSchema?.collation ||
                      "N/A"}
                  </td>
                </tr>
                <tr>
                  <td class="cell-name">Checksum</td>
                  <td class="cell-type">
                    {tableStatistics?.checksum || "N/A"}
                  </td>
                </tr>
                <tr>
                  <td class="cell-name">Engine</td>
                  <td class="cell-type">
                    {tableStatistics?.engine || tableSchema?.engine || "N/A"}
                  </td>
                </tr>
                <tr>
                  <td class="cell-name">Comment</td>
                  <td class="cell-type">
                    {tableStatistics?.comment || ""}
                  </td>
                </tr>
              {/if}
            </tbody>
          </table>
        </div>
      {:else if activeTab === "Virtual"}
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
                    <td class="cell-comment"
                      >{column.generation_expression || "N/A"}</td
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
      {:else if activeTab === "Indexes"}
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
      {:else if activeTab === "Foreign Keys"}
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
                    <td class="cell-type"
                      >{fk.owner || tableInfo?.database || "N/A"}</td
                    >
                    <td class="cell-type">{fk.referenced_table}</td>
                    <td class="cell-type">{fk.referenced_column}</td>
                    <td class="cell-type">{fk.ref_object_type || "TABLE"}</td>
                    <td class="cell-type"
                      >{fk.on_delete
                        ? fk.on_delete.toUpperCase()
                        : "NO ACTION"}</td
                    >
                    <td class="cell-type"
                      >{fk.on_update
                        ? fk.on_update.toUpperCase()
                        : "NO ACTION"}</td
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
      {:else if activeTab === "DDL"}
        <div class="ddl-content">
          <pre
            class="ddl-code">{`CREATE TABLE \`${tableInfo?.name || "table"}\` (
${tableSchema?.columns?.map((col, i) => `  \`${col.name}\` ${col.data_type}${!col.nullable ? " NOT NULL" : ""}${col.is_auto_increment ? " AUTO_INCREMENT" : ""}${col.default_value ? ` DEFAULT ${col.default_value}` : ""}`).join(",\n") || ""}
)${tableSchema?.engine ? ` ENGINE=${tableSchema.engine}` : ""}${tableSchema?.collation ? ` COLLATE=${tableSchema.collation}` : ""};`}</pre>
        </div>
      {:else}
        <div class="empty-state">
          <i class="fas fa-info-circle"></i>
          <span>{activeTab} view is not yet implemented</span>
        </div>
      {/if}
    </div>
  </div>

  <!-- Footer with item count -->
  <div class="properties-footer">
    <div class="footer-left">
      <button class="footer-btn" title="Refresh">
        <i class="fas fa-sync-alt"></i>
        <span>Refresh</span>
      </button>
      <button class="footer-btn" title="Save">
        <i class="fas fa-save"></i>
        <span>Save</span>
      </button>
      <button class="footer-btn" title="Revert">
        <i class="fas fa-undo"></i>
        <span>Revert</span>
      </button>
      <button class="footer-btn" title="Add Column">
        <i class="fas fa-plus"></i>
        <span>Add</span>
      </button>
      <button class="footer-btn" title="Delete">
        <i class="fas fa-trash"></i>
        <span>Delete</span>
      </button>
    </div>
    <div class="footer-right">
      <span>{tableSchema?.columns?.length || 0} items</span>
    </div>
  </div>
</div>

<style>
  .properties-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }

  .table-info-header {
    display: flex;
    gap: 16px;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    font-size: 12px;
  }

  .info-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .info-label {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .info-value {
    color: var(--text-primary);
  }

  .info-input {
    width: 60px;
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
    font-size: 12px;
  }

  .properties-main {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .properties-sidebar {
    display: flex;
    flex-direction: column;
    width: 160px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    overflow-y: auto;
  }

  .sidebar-tab-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-left: 3px solid transparent;
    color: var(--text-secondary);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.2s ease;
    height: 28px;
  }

  .sidebar-tab-btn i {
    font-size: 11px;
    width: 16px;
    text-align: center;
  }

  .sidebar-tab-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .sidebar-tab-btn.active {
    background: var(--bg-primary);
    color: var(--accent-blue);
    border-left-color: var(--accent-blue);
    font-weight: 500;
  }

  .properties-content {
    flex: 1;
    overflow: auto;
    background: var(--bg-primary);
  }

  .loading-state,
  .error-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    color: var(--text-secondary);
  }

  .loading-state i,
  .error-state i,
  .empty-state i {
    font-size: 32px;
  }

  .error-state {
    color: var(--error-color);
  }

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

  /* Partition table specific styling */
  .schema-table td.cell-comment,
  .schema-table td.cell-type {
    word-break: break-word;
    white-space: normal;
    max-width: 300px;
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

  .ddl-content {
    padding: 16px;
    height: 100%;
    overflow: auto;
  }

  .ddl-code {
    background: var(--bg-tertiary);
    padding: 16px;
    border-radius: 4px;
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 12px;
    line-height: 1.6;
    color: var(--text-primary);
    overflow-x: auto;
    margin: 0;
  }

  .properties-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 2px 8px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    font-size: 10px;
    height: 24px;
  }

  .footer-left {
    display: flex;
    gap: 1px;
  }

  .footer-btn {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 2px 8px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 10px;
    cursor: pointer;
    transition: all 0.15s ease;
    border-radius: 3px;
    height: 100%;
  }

  .footer-btn i {
    font-size: 11px;
  }

  .footer-btn span {
    font-size: 10px;
    white-space: nowrap;
  }

  .footer-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .footer-btn:active {
    background: var(--bg-primary);
  }

  .footer-right {
    color: var(--text-secondary);
    padding-right: 4px;
    font-size: 10px;
  }
</style>
