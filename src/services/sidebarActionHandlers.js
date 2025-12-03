import {
  getConnectionForEdit,
  saveConnection,
  deleteConnection,
  getDatabaseObject,
} from "../utils/tauri";
import { sidebarStore } from "../stores/sidebar";
import { refreshDatabase, refreshSchema } from "./sidebarDataService";

/**
 * Connection action handlers
 */
export const connectionHandlers = {
  async edit(conn, getConnectionsInfo, connections) {
    try {
      const fullConfig = await getConnectionForEdit(conn.id);
      sidebarStore.openConnectionModal(fullConfig);
    } catch (error) {
      console.error("Failed to load connection for edit:", error);
    }
  },

  async delete(conn, getConnectionsInfo, connections) {
    if (confirm(`Are you sure you want to delete connection "${conn.name}"?`)) {
      try {
        await deleteConnection(conn.id);
        const conns = await getConnectionsInfo();
        connections.set(conns);
      } catch (error) {
        console.error("Failed to delete connection:", error);
        alert(`Failed to delete connection: ${error}`);
      }
    }
  },

  copy(conn) {
    const connectionInfo = `Name: ${conn.name}\nType: ${conn.db_type}\nHost: ${conn.host}\nPort: ${conn.port}`;
    navigator.clipboard.writeText(connectionInfo).then(
      () => console.log("Connection info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
  },

  async rename(conn, getConnectionsInfo, connections) {
    sidebarStore.openRenameModal(
      "Rename Connection",
      conn.name,
      async (newName) => {
        try {
          const fullConfig = await getConnectionForEdit(conn.id);
          fullConfig.name = newName;
          await saveConnection(fullConfig);
          const conns = await getConnectionsInfo();
          connections.set(conns);
        } catch (error) {
          console.error("Failed to rename connection:", error);
          alert(`Failed to rename connection: ${error}`);
        }
      }
    );
  },
};

/**
 * Database action handlers
 */
export const databaseHandlers = {
  sqlEditor(database, connection, dispatch) {
    dispatch("openSqlEditorTab", { database, connection });
  },

  view(database, connection) {
    console.log("View database:", database.name);
    // TODO: Implement database view modal
  },

  copy(database, connection) {
    const databaseInfo = `Database: ${database.name}\nConnection: ${connection.name}\nType: ${connection.db_type}`;
    navigator.clipboard.writeText(databaseInfo).then(
      () => console.log("Database info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
  },

  paste(database, connection) {
    console.log("Paste to database:", database.name);
    // TODO: Implement paste functionality
  },

  copyAdvancedInfo(database, connection) {
    const advancedInfo = `Database Name: ${database.name}\nConnection: ${connection.name}\nConnection ID: ${connection.id}\nDB Type: ${connection.db_type}\nHost: ${connection.host}\nPort: ${connection.port}`;
    navigator.clipboard.writeText(advancedInfo).then(
      () => console.log("Advanced database info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
  },

  delete(database, connection) {
    if (
      confirm(
        `Are you sure you want to delete database "${database.name}"?\n\nWARNING: This will permanently delete all data in this database!`
      )
    ) {
      console.log("Delete database:", database.name);
      // TODO: Implement database deletion (requires backend command)
      alert("Database deletion is not yet implemented.");
    }
  },

  rename(database, connection) {
    sidebarStore.openRenameModal(
      "Rename Database",
      database.name,
      async (newName) => {
        console.log("Rename database from", database.name, "to", newName);
        // TODO: Implement database rename (requires backend command)
        alert("Database rename is not yet implemented.");
      }
    );
  },

  async refresh(database, connection) {
    try {
      await refreshDatabase(connection.id, database.name, connection.db_type);
    } catch (error) {
      console.error("Failed to refresh database:", error);
    }
  },
};

/**
 * Schema action handlers
 */
export const schemaHandlers = {
  sqlEditor(schema, database, connection, dispatch) {
    dispatch("openSqlEditorTab", { schema, database, connection });
  },

  view(schema, database, connection) {
    console.log("View schema:", schema);
    // TODO: Implement schema view modal
  },

  viewDiagram(schema, database, connection) {
    console.log("View diagram for schema:", schema);
    // TODO: Implement schema diagram view
  },

  importData(schema, database, connection) {
    console.log("Import data to schema:", schema);
    // TODO: Implement import data functionality
  },

  generateSql(schema, database, connection) {
    console.log("Generate SQL for schema:", schema);
    // TODO: Implement SQL generation
  },

  copy(schema, database, connection) {
    const schemaInfo = `Schema: ${schema}\nDatabase: ${database.name}\nConnection: ${connection.name}`;
    navigator.clipboard.writeText(schemaInfo).then(
      () => console.log("Schema info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
  },

  paste(schema, database, connection) {
    console.log("Paste to schema:", schema);
    // TODO: Implement paste functionality
  },

  copyAdvancedInfo(schema, database, connection) {
    const advancedInfo = `Schema: ${schema}\nDatabase: ${database.name}\nConnection: ${connection.name}\nConnection ID: ${connection.id}\nDB Type: ${connection.db_type}`;
    navigator.clipboard.writeText(advancedInfo).then(
      () => console.log("Advanced schema info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
  },

  delete(schema, database, connection) {
    if (confirm(`Are you sure you want to delete schema "${schema}"?`)) {
      console.log("Delete schema:", schema);
      // TODO: Implement schema deletion
      alert("Schema deletion is not yet implemented.");
    }
  },

  rename(schema, database, connection) {
    sidebarStore.openRenameModal("Rename Schema", schema, async (newName) => {
      console.log("Rename schema from", schema, "to", newName);
      // TODO: Implement schema rename
      alert("Schema rename is not yet implemented.");
    });
  },

  async refresh(schema, database, connection) {
    try {
      await refreshSchema(connection.id, database.name, schema);
    } catch (error) {
      console.error("Failed to refresh schema:", error);
    }
  },
};

/**
 * Table action handlers
 */
export const tableHandlers = {
  viewTable(table, database, connection) {
    console.log("View table structure:", table.name);
    // TODO: Implement table structure view
  },

  viewDiagram(table, database, connection) {
    console.log("View diagram for table:", table.name);
    // TODO: Implement table diagram
  },

  exportData(table, database, connection) {
    console.log("Export data from table:", table.name);
    // TODO: Implement data export
  },

  importData(table, database, connection) {
    console.log("Import data to table:", table.name);
    // TODO: Implement data import
  },

  readInConsole(table, database, connection, dispatch) {
    const schema = table.schema ? `${table.schema}.` : "";
    const query = `SELECT * FROM ${schema}${table.name} LIMIT 100;`;
    dispatch("openSqlEditorTab", { database, connection, initialQuery: query });
  },

  copy(table, database, connection) {
    const tableInfo = `Table: ${table.name}\nDatabase: ${database.name}\nConnection: ${connection.name}`;
    navigator.clipboard.writeText(tableInfo).then(
      () => console.log("Table info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
  },

  paste(table, database, connection) {
    console.log("Paste to table:", table.name);
    // TODO: Implement paste functionality
  },

  copyAdvancedInfo(table, database, connection) {
    const advancedInfo = `Table: ${table.name}\nSchema: ${
      table.schema || "N/A"
    }\nDatabase: ${database.name}\nConnection: ${
      connection.name
    }\nConnection ID: ${connection.id}\nDB Type: ${connection.db_type}`;
    navigator.clipboard.writeText(advancedInfo).then(
      () => console.log("Advanced table info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
  },

  delete(table, database, connection) {
    if (
      confirm(
        `Are you sure you want to delete table "${table.name}"?\n\nWARNING: This will permanently delete all data in this table!`
      )
    ) {
      console.log("Delete table:", table.name);
      // TODO: Implement table deletion
      alert("Table deletion is not yet implemented.");
    }
  },

  rename(table, database, connection) {
    sidebarStore.openRenameModal(
      "Rename Table",
      table.name,
      async (newName) => {
        console.log("Rename table from", table.name, "to", newName);
        // TODO: Implement table rename
        alert("Table rename is not yet implemented.");
      }
    );
  },

  async refresh(table, database, connection) {
    const dbKey = `${connection.id}-${database.name}`;
    try {
      const response = await getDatabaseObject(
        connection.id,
        "database_info",
        database.name
      );
      if (response && response.tables) {
        sidebarStore.updateCachedData(dbKey, (data) => ({
          ...data,
          tables: response.tables,
        }));
      }
    } catch (error) {
      console.error("Failed to refresh table:", error);
    }
  },
};

/**
 * View action handlers
 */
export const viewHandlers = {
  structure(view, database, connection) {
    console.log("View structure:", view.name);
    // TODO: Implement view structure modal
  },

  definition(view, database, connection) {
    console.log("View definition:", view.name);
    // TODO: Implement view definition modal
  },

  exportData(view, database, connection) {
    console.log("Export data from view:", view.name);
    // TODO: Implement data export
  },

  importData(view, database, connection) {
    console.log("Import data to view:", view.name);
    // TODO: Implement data import
  },

  readInConsole(view, database, connection, dispatch) {
    const schema = view.schema ? `${view.schema}.` : "";
    const query = `SELECT * FROM ${schema}${view.name} LIMIT 100;`;
    dispatch("openSqlEditorTab", { database, connection, initialQuery: query });
  },

  copy(view, database, connection) {
    const viewInfo = `View: ${view.name}\nDatabase: ${database.name}\nConnection: ${connection.name}`;
    navigator.clipboard.writeText(viewInfo).then(
      () => console.log("View info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
  },

  copyAdvancedInfo(view, database, connection) {
    const advancedInfo = `View: ${view.name}\nSchema: ${
      view.schema || "N/A"
    }\nDatabase: ${database.name}\nConnection: ${
      connection.name
    }\nConnection ID: ${connection.id}\nDB Type: ${connection.db_type}`;
    navigator.clipboard.writeText(advancedInfo).then(
      () => console.log("Advanced view info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
  },

  rename(view, database, connection) {
    sidebarStore.openRenameModal("Rename View", view.name, async (newName) => {
      console.log("Rename view from", view.name, "to", newName);
      // TODO: Implement view rename
      alert("View rename is not yet implemented.");
    });
  },

  delete(view, database, connection) {
    if (confirm(`Are you sure you want to delete view "${view.name}"?`)) {
      console.log("Delete view:", view.name);
      // TODO: Implement view deletion
      alert("View deletion is not yet implemented.");
    }
  },

  async refresh(view, database, connection) {
    const dbKey = `${connection.id}-${database.name}`;
    try {
      const response = await getDatabaseObject(
        connection.id,
        "database_info",
        database.name
      );
      if (response && response.views) {
        sidebarStore.updateCachedData(dbKey, (data) => ({
          ...data,
          views: response.views,
        }));
      }
    } catch (error) {
      console.error("Failed to refresh view:", error);
    }
  },
};
