import { getPropertiesObject } from "../../../core/integrations/tauri";
import { DatabaseType } from "../../../core/config/databaseTypes";

// Active abort controllers for cancellable operations
const abortControllers = new Map();

/**
 * Cancel an ongoing operation
 */
export function cancelOperation(key) {
  const controller = abortControllers.get(key);
  if (controller) {
    controller.abort();
    abortControllers.delete(key);
  }
}

/**
 * Cancel all ongoing operations
 */
export function cancelAllOperations() {
  abortControllers.forEach((controller) => controller.abort());
  abortControllers.clear();
}

/**
 * Build table identifier with schema for PostgreSQL and MSSQL
 */
function buildTableIdentifier(tableInfo, dbType) {
  if (!tableInfo) return null;

  let tableIdentifier = tableInfo.name;
  if (
    (dbType === DatabaseType.POSTGRESQL || dbType === DatabaseType.MSSQL) &&
    tableInfo.schema
  ) {
    tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
  }
  return tableIdentifier;
}

/**
 * Load table schema
 */
export async function loadTableSchema(connection, tableInfo) {
  const opKey = `schema-${connection.id}-${tableInfo?.database}-${tableInfo?.name}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  try {
    if (!connection || !tableInfo) {
      throw new Error("No connection or table information available");
    }

    const tableIdentifier = buildTableIdentifier(tableInfo, connection.db_type);

    const schema = await getPropertiesObject(
      connection.id,
      "schema",
      tableInfo.database,
      tableIdentifier
    );

    if (controller.signal.aborted) return null;

    return schema;
  } catch (error) {
    if (controller.signal.aborted) return null;
    console.error("Error loading table schema:", error);
    throw error;
  } finally {
    abortControllers.delete(opKey);
  }
}

/**
 * Load table statistics
 */
export async function loadTableStatistics(connection, tableInfo) {
  const opKey = `stats-${connection.id}-${tableInfo?.database}-${tableInfo?.name}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  try {
    if (!connection || !tableInfo) {
      return null;
    }

    const tableIdentifier = buildTableIdentifier(tableInfo, connection.db_type);

    const statistics = await getPropertiesObject(
      connection.id,
      "statistics",
      tableInfo.database,
      tableIdentifier
    );

    if (controller.signal.aborted) return null;

    return statistics;
  } catch (error) {
    if (controller.signal.aborted) return null;
    console.error("Error loading table statistics:", error);
    return null; // Statistics is optional
  } finally {
    abortControllers.delete(opKey);
  }
}

/**
 * Load table references
 */
export async function loadTableReferences(connection, tableInfo) {
  const opKey = `refs-${connection.id}-${tableInfo?.database}-${tableInfo?.name}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  try {
    if (!connection || !tableInfo) {
      return [];
    }

    const tableIdentifier = buildTableIdentifier(tableInfo, connection.db_type);

    const references = await getPropertiesObject(
      connection.id,
      "relationships",
      tableInfo.database,
      tableIdentifier
    );

    if (controller.signal.aborted) return null;

    return references;
  } catch (error) {
    if (controller.signal.aborted) return null;
    console.error("Error loading table references:", error);
    return []; // References is optional
  } finally {
    abortControllers.delete(opKey);
  }
}

/**
 * Load table triggers
 */
export async function loadTableTriggers(connection, tableInfo) {
  const opKey = `triggers-${connection.id}-${tableInfo?.database}-${tableInfo?.name}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  try {
    if (!connection || !tableInfo) {
      return [];
    }

    const allTriggers = await getPropertiesObject(
      connection.id,
      "triggers",
      tableInfo.database,
      tableInfo.name
    );

    if (controller.signal.aborted) return null;

    // Filter triggers for this specific table
    const tableTriggers = allTriggers.filter(
      (trigger) => trigger.table_name === tableInfo.name
    );

    return tableTriggers;
  } catch (error) {
    if (controller.signal.aborted) return null;
    console.error("Error loading table triggers:", error);
    return []; // Triggers is optional
  } finally {
    abortControllers.delete(opKey);
  }
}

/**
 * Load all PostgreSQL-specific data in parallel
 */
export async function loadPgData(connection, tableInfo) {
  const opKey = `pg-${connection.id}-${tableInfo?.database}-${tableInfo?.name}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  try {
    if (!connection || !tableInfo) {
      return null;
    }

    if (connection.db_type !== DatabaseType.POSTGRESQL) {
      return null;
    }

    const tableIdentifier = buildTableIdentifier(tableInfo, connection.db_type);

    // Load all PostgreSQL-specific data in parallel
    const [constraints, foreignKeys, indexes, references, partitions] =
      await Promise.all([
        getPropertiesObject(
          connection.id,
          "pg_constraints",
          tableInfo.database,
          tableIdentifier
        ),
        getPropertiesObject(
          connection.id,
          "pg_foreign_keys",
          tableInfo.database,
          tableIdentifier
        ),
        getPropertiesObject(
          connection.id,
          "pg_indexes",
          tableInfo.database,
          tableIdentifier
        ),
        getPropertiesObject(
          connection.id,
          "pg_references",
          tableInfo.database,
          tableIdentifier
        ),
        getPropertiesObject(
          connection.id,
          "pg_partitions",
          tableInfo.database,
          tableIdentifier
        ),
      ]);

    if (controller.signal.aborted) return null;

    return {
      constraints: constraints || [],
      foreignKeys: foreignKeys || [],
      indexes: indexes || [],
      references: references || [],
      partitions: partitions || [],
    };
  } catch (error) {
    if (controller.signal.aborted) return null;
    console.error("Error loading PostgreSQL data:", error);
    return {
      constraints: [],
      foreignKeys: [],
      indexes: [],
      references: [],
      partitions: [],
    };
  } finally {
    abortControllers.delete(opKey);
  }
}

/**
 * Load all table properties data
 */
export async function loadAllTableData(connection, tableInfo) {
  try {
    const isPostgres = connection?.db_type === DatabaseType.POSTGRESQL;

    // Load base data in parallel
    const [schema, statistics, references, triggers, pgData] =
      await Promise.all([
        loadTableSchema(connection, tableInfo),
        loadTableStatistics(connection, tableInfo),
        loadTableReferences(connection, tableInfo),
        loadTableTriggers(connection, tableInfo),
        isPostgres ? loadPgData(connection, tableInfo) : Promise.resolve(null),
      ]);

    return {
      schema,
      statistics,
      references,
      triggers,
      pgData,
    };
  } catch (error) {
    console.error("Error loading all table data:", error);
    throw error;
  }
}
