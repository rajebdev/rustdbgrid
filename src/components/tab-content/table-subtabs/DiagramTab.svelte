<script>
  import { getTableRelationships, getTableSchema } from "../../../utils/tauri";
  import { activeConnection } from "../../../stores/connections";

  export let tableInfo;
  export let connection;

  let relationships = [];
  let loading = true;
  let error = null;
  let svgWidth = 1600;
  let svgHeight = 1000;
  let zoom = 0.7; // Default zoom 70%
  let panX = 0;
  let panY = 0;
  let isPanning = false;
  let startPanX = 0;
  let startPanY = 0;
  let currentTableSchema = null;
  let relatedTables = new Map();

  // Track dragging state for individual tables
  let draggingTable = null;
  let dragStartX = 0;
  let dragStartY = 0;
  let tableOffsets = new Map(); // Store custom positions for tables

  // Reactive statement to reload data when tableInfo changes
  $: if (tableInfo) {
    loadDiagramData();
  }

  async function loadDiagramData() {
    try {
      loading = true;
      error = null;
      const conn = connection || $activeConnection;

      if (!conn || !tableInfo) {
        error = "No connection or table information available";
        return;
      }

      // Reset data
      currentTableSchema = null;
      relatedTables = new Map();
      relationships = [];
      tableOffsets = new Map(); // Reset positions when loading new table

      // Load current table schema
      currentTableSchema = await getTableSchema(
        conn,
        tableInfo.database,
        tableInfo.name
      );

      // Load relationships
      relationships = await getTableRelationships(
        conn,
        tableInfo.database,
        tableInfo.name
      );

      console.log(
        `✅ Loaded ${relationships.length} relationships for ${tableInfo.name}`
      );

      // Load schemas for related tables
      const relatedTableNames = new Set();
      relationships.forEach((rel) => {
        if (
          rel.referenced_table_name &&
          rel.referenced_table_name !== tableInfo.name
        ) {
          relatedTableNames.add(rel.referenced_table_name);
        }
        if (rel.table_name && rel.table_name !== tableInfo.name) {
          relatedTableNames.add(rel.table_name);
        }
      });

      console.log(
        `✅ Found ${relatedTableNames.size} related tables: ${Array.from(relatedTableNames).join(", ")}`
      );

      // Fetch schemas for related tables
      for (const tableName of relatedTableNames) {
        try {
          const schema = await getTableSchema(
            conn,
            tableInfo.database,
            tableName
          );
          relatedTables.set(tableName, schema);
        } catch (e) {
          console.warn(`Could not load schema for ${tableName}:`, e);
        }
      }

      console.log(`✅ Loaded ${relatedTables.size} related table schemas`);
    } catch (e) {
      console.error("Error loading diagram data:", e);
      error = e.message || "Failed to load diagram data";
    } finally {
      loading = false;
    }
  }

  function handleZoomIn() {
    zoom = Math.min(zoom * 1.2, 3);
  }

  function handleZoomOut() {
    zoom = Math.max(zoom / 1.2, 0.3);
  }

  function handleFitToScreen() {
    zoom = 0.7;
    panX = 0;
    panY = 0;
  }

  function handleReset() {
    zoom = 0.7;
    panX = 0;
    panY = 0;
  }

  function handleMouseDown(event) {
    // Check if clicking on a table (handled separately)
    if (event.target.closest(".table-node")) {
      return;
    }

    if (event.button === 0) {
      // Left mouse button for canvas panning
      isPanning = true;
      startPanX = event.clientX - panX;
      startPanY = event.clientY - panY;
    }
  }

  function handleTableMouseDown(event, table) {
    event.stopPropagation();
    draggingTable = table.tableName;
    const rect = event.currentTarget.getBoundingClientRect();
    const svgRect = event.currentTarget.closest("svg").getBoundingClientRect();
    dragStartX = (event.clientX - svgRect.left - panX) / zoom - table.x;
    dragStartY = (event.clientY - svgRect.top - panY) / zoom - table.y;
  }

  function handleMouseMove(event) {
    if (draggingTable) {
      const svgRect = event.currentTarget.getBoundingClientRect();
      const x = (event.clientX - svgRect.left - panX) / zoom - dragStartX;
      const y = (event.clientY - svgRect.top - panY) / zoom - dragStartY;

      // Calculate offset from original position
      const table = tablePositions.find((t) => t.tableName === draggingTable);
      if (table) {
        const centerX = svgWidth / 2;
        const centerY = svgHeight / 2;
        const radius = 280; // Match calculateTablePositions radius

        let originalX, originalY;
        if (draggingTable === tableInfo.name) {
          originalX = centerX - 100;
          originalY = centerY - 80;
        } else {
          const relatedTableArray = Array.from(relatedTables.entries());
          const index = relatedTableArray.findIndex(
            ([name]) => name === draggingTable
          );
          if (index >= 0) {
            const angle = (2 * Math.PI * index) / relatedTableArray.length;
            originalX = centerX + radius * Math.cos(angle) - 100;
            originalY = centerY + radius * Math.sin(angle) - 80;
          }
        }

        if (originalX !== undefined) {
          tableOffsets.set(draggingTable, {
            x: x - originalX,
            y: y - originalY,
          });
          tableOffsets = tableOffsets; // Trigger reactivity
          tablePositions = calculateTablePositions();
          relationshipLines = calculateRelationshipLines(tablePositions);
        }
      }
    } else if (isPanning) {
      panX = event.clientX - startPanX;
      panY = event.clientY - startPanY;
    }
  }

  function handleMouseUp() {
    draggingTable = null;
    isPanning = false;
  }

  function handleWheel(event) {
    event.preventDefault();
    const delta = event.deltaY > 0 ? 0.9 : 1.1;
    zoom = Math.max(0.3, Math.min(3, zoom * delta));
  }

  function getTransform() {
    return `translate(${panX}, ${panY}) scale(${zoom})`;
  }

  function renderTable(tableName, schema, x, y) {
    const columns = schema?.columns || [];
    const headerHeight = 30; // Reduced from 40
    const rowHeight = 20; // Reduced from 25
    const maxVisibleColumns = 6; // Show fewer columns
    const visibleColumns = columns.slice(0, maxVisibleColumns);
    const tableHeight =
      headerHeight +
      visibleColumns.length * rowHeight +
      (columns.length > maxVisibleColumns ? 20 : 0);
    const tableWidth = 200; // Reduced from 250

    return {
      x,
      y,
      width: tableWidth,
      height: tableHeight,
      columns: visibleColumns,
      allColumns: columns,
      tableName,
    };
  }

  function calculateTablePositions() {
    console.log("📊 Calculating table positions...");
    console.log(
      "Current table schema:",
      currentTableSchema ? "✅ EXISTS" : "❌ NULL"
    );
    console.log("Related tables size:", relatedTables?.size || 0);

    const centerX = svgWidth / 2;
    const centerY = svgHeight / 2;
    const radius = 280; // Reduced from 450 for closer spacing

    const tables = [];

    // Center table (current table)
    if (currentTableSchema) {
      const offsetX = tableOffsets.get(tableInfo.name)?.x || 0;
      const offsetY = tableOffsets.get(tableInfo.name)?.y || 0;
      const table = renderTable(
        tableInfo.name,
        currentTableSchema,
        centerX - 100 + offsetX,
        centerY - 80 + offsetY
      );
      tables.push(table);
      console.log("✅ Added center table:", tableInfo.name);
    } else {
      console.warn("❌ No currentTableSchema available!");
    }

    // Position related tables in a circle around the center
    const relatedTableArray = Array.from(relatedTables.entries());
    console.log(
      "Related tables array:",
      relatedTableArray.map(([name]) => name)
    );

    relatedTableArray.forEach(([tableName, schema], index) => {
      const angle = (2 * Math.PI * index) / relatedTableArray.length;
      const offsetX = tableOffsets.get(tableName)?.x || 0;
      const offsetY = tableOffsets.get(tableName)?.y || 0;
      const x = centerX + radius * Math.cos(angle) - 100 + offsetX;
      const y = centerY + radius * Math.sin(angle) - 80 + offsetY;
      const table = renderTable(tableName, schema, x, y);
      tables.push(table);
      console.log(`✅ Added related table: ${tableName}`);
    });

    console.log(`📊 Total tables in diagram: ${tables.length}`);
    return tables;
  }

  function calculateRelationshipLines(tables) {
    const lines = [];
    const centerTable = tables.find((t) => t.tableName === tableInfo.name);

    if (!centerTable) {
      console.warn("⚠️ No center table found!");
      return lines;
    }

    relationships.forEach((rel) => {
      const sourceTable =
        rel.table_name === tableInfo.name
          ? centerTable
          : tables.find((t) => t.tableName === rel.table_name);
      const targetTable =
        rel.referenced_table_name === tableInfo.name
          ? centerTable
          : tables.find((t) => t.tableName === rel.referenced_table_name);

      if (sourceTable && targetTable) {
        // Find the column positions in both tables
        const headerHeight = 30;
        const rowHeight = 20;

        // Find FK column in source table
        const sourceColumnIndex = sourceTable.allColumns.findIndex(
          (col) => col.name === rel.column_name
        );
        const sourceColumnY =
          sourceColumnIndex >= 0 && sourceColumnIndex < 6
            ? sourceTable.y +
              headerHeight +
              sourceColumnIndex * rowHeight +
              rowHeight / 2
            : sourceTable.y + sourceTable.height / 2;

        // Find PK column in target table
        const targetColumnIndex = targetTable.allColumns.findIndex(
          (col) => col.name === rel.referenced_column_name
        );
        const targetColumnY =
          targetColumnIndex >= 0 && targetColumnIndex < 6
            ? targetTable.y +
              headerHeight +
              targetColumnIndex * rowHeight +
              rowHeight / 2
            : targetTable.y + targetTable.height / 2;

        // Determine which side to connect from based on positions
        let x1,
          y1,
          x2,
          y2,
          sourcePosition,
          targetPosition,
          sourceAngle,
          targetAngle;

        const sourceCenterX = sourceTable.x + sourceTable.width / 2;
        const targetCenterX = targetTable.x + targetTable.width / 2;

        // ALWAYS use horizontal connection (from side, never from top/bottom)
        const dx = targetCenterX - sourceCenterX;

        if (dx > 0) {
          // Source right to target left
          x1 = sourceTable.x + sourceTable.width;
          y1 = sourceColumnY;
          x2 = targetTable.x;
          y2 = targetColumnY;
          sourcePosition = "right";
          targetPosition = "left";
          sourceAngle = 0; // Arrow points right
          targetAngle = 180; // Arrow points left
        } else {
          // Source left to target right
          x1 = sourceTable.x;
          y1 = sourceColumnY;
          x2 = targetTable.x + targetTable.width;
          y2 = targetColumnY;
          sourcePosition = "left";
          targetPosition = "right";
          sourceAngle = 180; // Arrow points left
          targetAngle = 0; // Arrow points right
        }

        // Determine cardinality (for crow's foot notation)
        // FK side is "many" (crow's foot), PK side is "one"
        // rel.table_name has the FK, rel.referenced_table_name has the PK
        const sourceCardinality = "many"; // Source has FK = many
        const targetCardinality = "one"; // Target has PK = one

        // Calculate elbow points for the connector
        const midX1 = x1 + (x2 - x1) * 0.5;
        const midX2 = x1 + (x2 - x1) * 0.5;

        lines.push({
          x1,
          y1,
          x2,
          y2,
          midX1,
          midX2,
          sourcePosition,
          targetPosition,
          sourceAngle,
          targetAngle,
          sourceCardinality,
          targetCardinality,
          label: `${rel.column_name}`,
          type: rel.relationship_type,
        });
      } else {
        console.warn("⚠️ Could not find tables for relationship:", {
          constraint: rel.constraint_name,
          from: rel.table_name,
          to: rel.referenced_table_name,
          sourceFound: !!sourceTable,
          targetFound: !!targetTable,
        });
      }
    });

    console.log(`✅ Generated ${lines.length} relationship lines`);
    return lines;
  }

  // Reactive calculations that depend on loaded data
  // Calculate when schema changes or loading completes
  $: if (!loading && currentTableSchema) {
    console.log("🔄 Triggering reactive calculations...");
    tablePositions = calculateTablePositions();
    relationshipLines =
      tablePositions.length > 0
        ? calculateRelationshipLines(tablePositions)
        : [];
  } else if (loading) {
    tablePositions = [];
    relationshipLines = [];
  }

  // Initialize variables
  let tablePositions = [];
  let relationshipLines = [];
</script>

<div class="diagram-container">
  {#if loading}
    <div class="loading-state">
      <i class="fas fa-spinner fa-spin"></i>
      <span>Loading diagram...</span>
    </div>
  {:else if error}
    <div class="error-state">
      <i class="fas fa-exclamation-triangle"></i>
      <span>{error}</span>
    </div>
  {:else if !currentTableSchema}
    <div class="error-state">
      <i class="fas fa-info-circle"></i>
      <span>No table schema available</span>
    </div>
  {:else}
    <div class="diagram-content">
      <div class="diagram-toolbar">
        <div class="toolbar-left">
          <button class="toolbar-btn" title="Zoom In" on:click={handleZoomIn}>
            <i class="fas fa-search-plus"></i>
          </button>
          <button class="toolbar-btn" title="Zoom Out" on:click={handleZoomOut}>
            <i class="fas fa-search-minus"></i>
          </button>
          <button
            class="toolbar-btn"
            title="Fit to Screen"
            on:click={handleFitToScreen}
          >
            <i class="fas fa-compress"></i>
          </button>
          <button class="toolbar-btn" title="Reset" on:click={handleReset}>
            <i class="fas fa-redo"></i>
          </button>
          <span class="zoom-level">{Math.round(zoom * 100)}%</span>
        </div>
        <div class="toolbar-right">
          <span class="info-text">
            {tablePositions.length} table{tablePositions.length !== 1
              ? "s"
              : ""} ·
            {relationshipLines.length} relationship{relationshipLines.length !==
            1
              ? "s"
              : ""}
          </span>
          <button
            class="toolbar-btn"
            title="Refresh"
            on:click={loadDiagramData}
          >
            <i class="fas fa-sync-alt"></i>
          </button>
        </div>
      </div>

      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div
        class="diagram-canvas"
        on:mousedown={handleMouseDown}
        on:mousemove={handleMouseMove}
        on:mouseup={handleMouseUp}
        on:mouseleave={handleMouseUp}
        on:wheel={handleWheel}
        style="cursor: {isPanning ? 'grabbing' : 'grab'}"
      >
        <svg width={svgWidth} height={svgHeight}>
          <g transform={getTransform()}>
            <!-- Crow's foot markers definition -->
            <defs>
              <!-- One side marker (single line) -->
              <marker
                id="one-marker"
                markerWidth="12"
                markerHeight="12"
                refX="6"
                refY="6"
                orient="auto"
              >
                <line
                  x1="6"
                  y1="0"
                  x2="6"
                  y2="12"
                  stroke="var(--border-color)"
                  stroke-width="2"
                />
              </marker>

              <!-- Many side marker (crow's foot - three lines) -->
              <marker
                id="many-marker"
                markerWidth="15"
                markerHeight="15"
                refX="0"
                refY="7.5"
                orient="auto"
              >
                <path
                  d="M 0,7.5 L 10,0 M 0,7.5 L 10,7.5 M 0,7.5 L 10,15"
                  stroke="var(--border-color)"
                  stroke-width="2"
                  fill="none"
                />
              </marker>
            </defs>

            <!-- Relationship lines with crow's foot notation -->
            <g class="relationships">
              {#each relationshipLines as line}
                <g>
                  <!-- Elbow connector (3-segment line) -->
                  <path
                    d={`M ${line.x1},${line.y1} L ${line.midX1},${line.y1} L ${line.midX2},${line.y2} L ${line.x2},${line.y2}`}
                    stroke="var(--border-color)"
                    stroke-width="1.5"
                    fill="none"
                  />

                  <!-- Markers at endpoints -->
                  <!-- Source marker (FK side = many) -->
                  <g
                    transform="translate({line.x1}, {line.y1}) rotate({line.sourceAngle})"
                  >
                    {#if line.sourceCardinality === "many"}
                      <!-- Crow's foot pointing outward from table (larger, more visible) -->
                      <path
                        d="M -12,-6 L 0,0 M -12,0 L 0,0 M -12,6 L 0,0"
                        stroke="var(--text-primary)"
                        stroke-width="2"
                        fill="none"
                        stroke-linecap="round"
                      />
                    {:else}
                      <line
                        x1="0"
                        y1="-6"
                        x2="0"
                        y2="6"
                        stroke="var(--text-primary)"
                        stroke-width="2"
                        stroke-linecap="round"
                      />
                    {/if}
                  </g>

                  <!-- Target marker (PK side = one) -->
                  <g
                    transform="translate({line.x2}, {line.y2}) rotate({line.targetAngle})"
                  >
                    {#if line.targetCardinality === "many"}
                      <!-- Crow's foot pointing outward from table (larger, more visible) -->
                      <path
                        d="M -12,-6 L 0,0 M -12,0 L 0,0 M -12,6 L 0,0"
                        stroke="var(--text-primary)"
                        stroke-width="2"
                        fill="none"
                        stroke-linecap="round"
                      />
                    {:else}
                      <!-- Single perpendicular line for "one" side -->
                      <line
                        x1="0"
                        y1="-6"
                        x2="0"
                        y2="6"
                        stroke="var(--text-primary)"
                        stroke-width="2"
                        stroke-linecap="round"
                      />
                    {/if}
                  </g>

                  <!-- Label for relationship (smaller text) -->
                  <text
                    x={(line.x1 + line.x2) / 2}
                    y={(line.y1 + line.y2) / 2 - 8}
                    fill="var(--text-muted)"
                    font-size="9"
                    text-anchor="middle"
                    class="relationship-label"
                  >
                    {line.label}
                  </text>
                </g>
              {/each}
            </g>

            <!-- Tables (draggable) -->
            {#each tablePositions as table}
              <!-- svelte-ignore a11y-no-static-element-interactions -->
              <g
                class="table-node"
                transform="translate({table.x}, {table.y})"
                on:mousedown={(e) => handleTableMouseDown(e, table)}
                style="cursor: move;"
              >
                <!-- Table container -->
                <rect
                  x="0"
                  y="0"
                  width={table.width}
                  height={table.height}
                  fill="var(--bg-secondary)"
                  stroke={table.tableName === tableInfo.name
                    ? "var(--accent-blue)"
                    : "var(--border-color)"}
                  stroke-width={table.tableName === tableInfo.name ? "2" : "1"}
                  rx="4"
                />

                <!-- Table header -->
                <rect
                  x="0"
                  y="0"
                  width={table.width}
                  height="30"
                  fill={table.tableName === tableInfo.name
                    ? "var(--accent-blue)"
                    : "var(--bg-tertiary)"}
                  rx="4"
                />
                <text
                  x={table.width / 2}
                  y="20"
                  text-anchor="middle"
                  fill={table.tableName === tableInfo.name
                    ? "white"
                    : "var(--text-primary)"}
                  font-size="12"
                  font-weight="600"
                >
                  {table.tableName}
                </text>

                <!-- Columns -->
                {#each table.columns as column, idx}
                  <g transform="translate(0, {30 + idx * 20})">
                    <text
                      x="8"
                      y="14"
                      fill="var(--text-primary)"
                      font-size="10"
                    >
                      {#if column.is_primary_key}
                        <tspan fill="#ffd700" font-size="9">🔑</tspan>
                      {:else if column.is_foreign_key}
                        <tspan fill="#4a9eff" font-size="9">🔗</tspan>
                      {:else}
                        <tspan fill="var(--text-secondary)">•</tspan>
                      {/if}
                      <tspan dx="4">{column.name}</tspan>
                      <tspan dx="3" fill="var(--text-muted)" font-size="8"
                        >{column.data_type}</tspan
                      >
                    </text>
                  </g>
                {/each}

                {#if table.allColumns.length > table.columns.length}
                  <text
                    x={table.width / 2}
                    y={30 + table.columns.length * 20 + 14}
                    text-anchor="middle"
                    fill="var(--text-muted)"
                    font-size="9"
                  >
                    +{table.allColumns.length - table.columns.length} more
                  </text>
                {/if}
              </g>
            {/each}

            <!-- Info message when no relationships -->
            {#if tablePositions.length === 1 && relationshipLines.length === 0}
              <text
                x={svgWidth / 2}
                y={svgHeight - 50}
                text-anchor="middle"
                fill="var(--text-secondary)"
                font-size="14"
              >
                No relationships found for this table
              </text>
              <text
                x={svgWidth / 2}
                y={svgHeight - 30}
                text-anchor="middle"
                fill="var(--text-muted)"
                font-size="12"
              >
                This table has no foreign keys or references from other tables
              </text>
            {/if}
          </g>
        </svg>
      </div>
    </div>
  {/if}
</div>

<style>
  .diagram-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }

  .loading-state,
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    color: var(--text-secondary);
  }

  .loading-state i,
  .error-state i {
    font-size: 32px;
  }

  .error-state {
    color: var(--error-color);
  }

  .diagram-content {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .diagram-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    gap: 12px;
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .toolbar-btn {
    padding: 6px 10px;
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toolbar-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .zoom-level {
    font-size: 12px;
    color: var(--text-secondary);
    padding: 0 8px;
    min-width: 50px;
    text-align: center;
  }

  .info-text {
    font-size: 12px;
    color: var(--text-secondary);
    padding: 0 8px;
  }

  .diagram-canvas {
    flex: 1;
    position: relative;
    overflow: hidden;
    background: var(--bg-tertiary);
    user-select: none;
  }

  svg {
    display: block;
    width: 100%;
    height: 100%;
  }

  .table-node {
    cursor: move;
    transition: filter 0.2s ease;
  }

  .table-node:hover {
    filter: brightness(1.05);
  }

  .table-node:active {
    cursor: grabbing;
  }

  .relationship-label {
    pointer-events: none;
    user-select: none;
  }

  .relationships line {
    pointer-events: none;
  }
</style>
