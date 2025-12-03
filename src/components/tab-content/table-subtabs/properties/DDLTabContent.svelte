<script>
  export let tableInfo;
  export let tableSchema;
</script>

<div class="ddl-content">
  <pre class="ddl-code">{`CREATE TABLE \`${tableInfo?.name || "table"}\` (
${tableSchema?.columns?.map((col, i) => `  \`${col.name}\` ${col.data_type}${!col.nullable ? " NOT NULL" : ""}${col.is_auto_increment ? " AUTO_INCREMENT" : ""}${col.default_value ? ` DEFAULT ${col.default_value}` : ""}`).join(",\n") || ""}
)${tableSchema?.engine ? ` ENGINE=${tableSchema.engine}` : ""}${tableSchema?.collation ? ` COLLATE=${tableSchema.collation}` : ""};`}</pre>
</div>

<style>
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
</style>
