/**
 * Apache Ignite Bridge - IPC Version
 *
 * Menggunakan Named Pipe (Windows) / Unix Socket untuk komunikasi
 * yang lebih secure dan efisien daripada HTTP.
 *
 * Security: Pipe name includes random suffix passed from parent process
 * sehingga proses lain tidak bisa menebak nama pipe.
 */

const net = require("net");
const IgniteClient = require("apache-ignite-client");
const IgniteClientConfiguration = IgniteClient.IgniteClientConfiguration;
const ScanQuery = IgniteClient.ScanQuery;
const SqlFieldsQuery = IgniteClient.SqlFieldsQuery;

// Pipe/Socket path - requires IGNITE_BRIDGE_PIPE from parent process
const PIPE_NAME = process.env.IGNITE_BRIDGE_PIPE;
if (!PIPE_NAME) {
  console.error("Error: IGNITE_BRIDGE_PIPE environment variable is required");
  process.exit(1);
}
const PIPE_PATH =
  process.platform === "win32"
    ? `\\\\.\\pipe\\${PIPE_NAME}`
    : `/tmp/${PIPE_NAME}.sock`;

// Store active connections
const connections = new Map();

/**
 * Extract fields from BinaryObject
 */
async function extractFields(binaryObj) {
  const result = {};

  try {
    result._type = binaryObj.getTypeName();
  } catch (e) {}

  try {
    for (const fieldName of binaryObj.getFieldNames()) {
      const value = await binaryObj.getField(fieldName);
      if (value && typeof value.getFieldNames === "function") {
        result[fieldName] = await extractFields(value);
      } else {
        result[fieldName] = value;
      }
    }
  } catch (e) {}

  return result;
}

/**
 * Convert value to JSON-safe format
 */
function toJsonSafe(value) {
  if (value === null || value === undefined) return null;
  if (typeof value === "bigint") return value.toString();
  if (Buffer.isBuffer(value)) return value.toString("base64");
  if (value instanceof Date) return value.toISOString();
  if (Array.isArray(value)) return value.map(toJsonSafe);
  if (typeof value === "object") {
    const result = {};
    for (const [k, v] of Object.entries(value)) {
      result[k] = toJsonSafe(v);
    }
    return result;
  }
  return value;
}

/**
 * Handle incoming request
 */
async function handleRequest(request) {
  const { action, id, ...params } = request;

  try {
    switch (action) {
      case "health":
        return { success: true, connections: connections.size };

      case "connect":
        return await handleConnect(params);

      case "disconnect":
        return await handleDisconnect(params);

      case "test":
        return await handleTest(params);

      case "caches":
        return await handleGetCaches(params);

      case "tables":
        return await handleGetTables(params);

      case "query":
        return await handleQuery(params);

      case "scan":
        return await handleScan(params);

      case "schema":
        return await handleGetSchema(params);

      case "shutdown":
        setTimeout(() => process.exit(0), 100);
        return { success: true, message: "Shutting down..." };

      default:
        return { success: false, message: `Unknown action: ${action}` };
    }
  } catch (e) {
    return { success: false, message: e.message };
  }
}

async function handleConnect({ connectionId, host, port, username, password }) {
  // Disconnect existing connection with same ID
  if (connections.has(connectionId)) {
    try {
      connections.get(connectionId).disconnect();
    } catch (e) {}
    connections.delete(connectionId);
  }

  const client = new IgniteClient();
  const addr = `${host}:${port}`;
  const config = new IgniteClientConfiguration(addr);

  if (username && password) {
    config.setUserName(username);
    config.setPassword(password);
  }

  await client.connect(config);
  connections.set(connectionId, client);
  console.error(`✅ Connected: ${connectionId}`);

  return { success: true, message: "Connected to Ignite" };
}

async function handleDisconnect({ connectionId }) {
  if (connections.has(connectionId)) {
    connections.get(connectionId).disconnect();
    connections.delete(connectionId);
    console.error(`🔌 Disconnected: ${connectionId}`);
  }

  // Auto-shutdown if no connections
  if (connections.size === 0) {
    console.error("📭 No active connections, shutting down...");
    setTimeout(() => process.exit(0), 100);
  }

  return { success: true };
}

async function handleTest({ host, port, username, password }) {
  const client = new IgniteClient();
  const addr = `${host}:${port}`;
  const config = new IgniteClientConfiguration(addr);

  if (username && password) {
    config.setUserName(username);
    config.setPassword(password);
  }

  await client.connect(config);
  const caches = await client.cacheNames();
  client.disconnect();

  return {
    success: true,
    message: `Connected successfully. Found ${caches.length} caches.`,
  };
}

async function handleGetCaches({ connectionId }) {
  const client = connections.get(connectionId);
  if (!client) {
    return { success: false, message: "Not connected" };
  }

  const caches = await client.cacheNames();
  return {
    success: true,
    caches: caches.map((name) => ({ name, type: "cache" })),
  };
}

async function handleGetTables({ connectionId, cacheName }) {
  const client = connections.get(connectionId);
  if (!client) {
    return { success: false, message: "Not connected" };
  }

  // Validate cache exists
  try {
    const cacheNames = await client.cacheNames();
    if (!cacheNames.includes(cacheName)) {
      return {
        success: true,
        tables: [],
        message: `Cache '${cacheName}' not found`,
      };
    }
  } catch (e) {
    return {
      success: false,
      message: `Failed to validate cache: ${e.message}`,
    };
  }

  try {
    const cache = client
      .getCache(cacheName)
      .setKeyType(IgniteClient.ObjectType.PRIMITIVE_TYPE.INTEGER);
    const query = new SqlFieldsQuery(
      "SELECT TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = ?"
    ).setArgs(cacheName.toUpperCase());

    const cursor = await cache.query(query);
    const rows = await cursor.getAll();

    return {
      success: true,
      tables: rows.map((row) => ({ name: row[0], type: "table" })),
    };
  } catch (sqlError) {
    return {
      success: true,
      tables: [{ name: cacheName, type: "cache" }],
    };
  }
}

async function handleQuery({ connectionId, query, cacheName }) {
  const client = connections.get(connectionId);
  if (!client) {
    return { success: false, message: "Not connected" };
  }

  const startTime = Date.now();
  const queryUpper = query.trim().toUpperCase();

  // Handle SHOW CACHES command
  if (queryUpper.includes("SHOW") && queryUpper.includes("CACHE")) {
    const caches = await client.cacheNames();
    return {
      success: true,
      result: {
        columns: ["cache_name"],
        rows: caches.map((name) => ({ cache_name: name })),
        rowsAffected: caches.length,
        executionTime: Date.now() - startTime,
      },
    };
  }

  const targetCache = cacheName || "PUBLIC";

  // Validate cache exists before querying
  let cache;
  try {
    const cacheNames = await client.cacheNames();
    if (!cacheNames.includes(targetCache)) {
      // Try PUBLIC cache as fallback, or first available cache
      const fallbackCache = cacheNames.includes("PUBLIC")
        ? "PUBLIC"
        : cacheNames[0];
      if (!fallbackCache) {
        return {
          success: false,
          message: `No caches available. Cache '${targetCache}' not found.`,
        };
      }
      cache = client.getCache(fallbackCache);
    } else {
      cache = client.getCache(targetCache);
    }
  } catch (cacheError) {
    return {
      success: false,
      message: `Failed to access cache: ${cacheError.message}`,
    };
  }

  const sqlQuery = new SqlFieldsQuery(query);
  sqlQuery.setIncludeFieldNames(true);

  const cursor = await cache.query(sqlQuery);
  const allRows = await cursor.getAll();

  let columns = [];
  const rows = [];

  if (allRows.length > 0) {
    try {
      columns = cursor.getFieldNames();
    } catch (e) {
      columns = allRows[0].map((_, i) => `column_${i}`);
    }

    for (const row of allRows) {
      const rowObj = {};
      for (let i = 0; i < columns.length; i++) {
        rowObj[columns[i]] = toJsonSafe(row[i]);
      }
      rows.push(rowObj);
    }
  }

  return {
    success: true,
    result: {
      columns,
      rows,
      rowsAffected: rows.length,
      executionTime: Date.now() - startTime,
      finalQuery: query,
    },
  };
}

async function handleScan({
  connectionId,
  cacheName,
  limit = 100,
  offset = 0,
}) {
  const client = connections.get(connectionId);
  if (!client) {
    return { success: false, message: "Not connected" };
  }

  const startTime = Date.now();

  // Validate cache exists before accessing
  try {
    const cacheNames = await client.cacheNames();
    if (!cacheNames.includes(cacheName)) {
      return {
        success: false,
        message: `Cache '${cacheName}' does not exist. Available caches: ${
          cacheNames.join(", ") || "none"
        }`,
      };
    }
  } catch (validationError) {
    return {
      success: false,
      message: `Failed to validate cache: ${validationError.message}`,
    };
  }

  let cache, cursor, entries;
  try {
    cache = client.getCache(cacheName);
    cursor = await cache.query(new ScanQuery());
    entries = await cursor.getAll();
  } catch (scanError) {
    // Handle cache-related errors with retry using getOrCreateCache
    if (
      scanError.message &&
      scanError.message.includes("Cache does not exist")
    ) {
      try {
        cache = await client.getOrCreateCache(cacheName);
        cursor = await cache.query(new ScanQuery());
        entries = await cursor.getAll();
      } catch (retryError) {
        return {
          success: false,
          message: `Failed to access cache '${cacheName}': ${retryError.message}`,
        };
      }
    } else {
      return {
        success: false,
        message: `Scan failed: ${scanError.message}`,
      };
    }
  }

  const rows = [];
  const columnsSet = new Set(["_key"]);

  const startIdx = offset;
  const endIdx = Math.min(offset + limit, entries.length);

  for (let i = startIdx; i < endIdx; i++) {
    const entry = entries[i];
    const key = entry.getKey();
    const value = entry.getValue();

    let rowObj = { _key: toJsonSafe(key) };

    if (value && typeof value.getFieldNames === "function") {
      const extracted = await extractFields(value);
      for (const [k, v] of Object.entries(extracted)) {
        if (k !== "_type") {
          columnsSet.add(k);
          rowObj[k] = toJsonSafe(v);
        }
      }
    } else if (typeof value === "object" && value !== null) {
      for (const [k, v] of Object.entries(value)) {
        columnsSet.add(k);
        rowObj[k] = toJsonSafe(v);
      }
    } else {
      columnsSet.add("_value");
      rowObj["_value"] = toJsonSafe(value);
    }

    rows.push(rowObj);
  }

  return {
    success: true,
    result: {
      columns: Array.from(columnsSet),
      rows,
      totalCount: entries.length,
      rowsAffected: rows.length,
      executionTime: Date.now() - startTime,
    },
  };
}

async function handleGetSchema({ connectionId, cacheName, tableName }) {
  const client = connections.get(connectionId);
  if (!client) {
    return { success: false, message: "Not connected" };
  }

  // Validate cache exists
  try {
    const cacheNames = await client.cacheNames();
    if (!cacheNames.includes(cacheName)) {
      return {
        success: false,
        message: `Cache '${cacheName}' does not exist`,
      };
    }
  } catch (e) {
    return {
      success: false,
      message: `Failed to validate cache: ${e.message}`,
    };
  }

  try {
    const cache = client.getCache(cacheName);
    const table = tableName || cacheName;

    const query = new SqlFieldsQuery(
      "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_DEFAULT FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = ?"
    ).setArgs(table.toUpperCase());

    const cursor = await cache.query(query);
    const rows = await cursor.getAll();

    return {
      success: true,
      schema: {
        tableName: table,
        columns: rows.map((row) => ({
          name: row[0],
          dataType: row[1],
          isNullable: row[2] === "YES",
          defaultValue: row[3],
          isPrimaryKey: false,
        })),
      },
    };
  } catch (sqlError) {
    // Infer schema from data
    const cache = client.getCache(cacheName);
    const cursor = await cache.query(new ScanQuery());
    const entries = await cursor.getAll();

    const columnsMap = new Map();
    columnsMap.set("_key", {
      name: "_key",
      dataType: "UNKNOWN",
      isNullable: false,
    });

    for (let i = 0; i < Math.min(10, entries.length); i++) {
      const value = entries[i].getValue();

      if (value && typeof value.getFieldNames === "function") {
        for (const fieldName of value.getFieldNames()) {
          if (!columnsMap.has(fieldName)) {
            const fieldValue = await value.getField(fieldName);
            columnsMap.set(fieldName, {
              name: fieldName,
              dataType: typeof fieldValue,
              isNullable: true,
            });
          }
        }
      } else if (typeof value === "object" && value !== null) {
        for (const [k, v] of Object.entries(value)) {
          if (!columnsMap.has(k)) {
            columnsMap.set(k, {
              name: k,
              dataType: typeof v,
              isNullable: true,
            });
          }
        }
      } else if (!columnsMap.has("_value")) {
        columnsMap.set("_value", {
          name: "_value",
          dataType: typeof value,
          isNullable: true,
        });
      }
    }

    return {
      success: true,
      schema: {
        tableName: cacheName,
        columns: Array.from(columnsMap.values()),
      },
    };
  }
}

/**
 * Message framing: Each message is prefixed with 4-byte length (big-endian)
 */
function createFramedMessage(data) {
  const json = JSON.stringify(data);
  const buf = Buffer.from(json, "utf8");
  const lenBuf = Buffer.alloc(4);
  lenBuf.writeUInt32BE(buf.length, 0);
  return Buffer.concat([lenBuf, buf]);
}

/**
 * Handle client connection
 */
function handleClient(socket) {
  let buffer = Buffer.alloc(0);

  socket.on("data", async (data) => {
    buffer = Buffer.concat([buffer, data]);

    // Process all complete messages
    while (buffer.length >= 4) {
      const msgLen = buffer.readUInt32BE(0);

      if (buffer.length < 4 + msgLen) {
        break; // Wait for more data
      }

      const msgData = buffer.slice(4, 4 + msgLen);
      buffer = buffer.slice(4 + msgLen);

      try {
        const request = JSON.parse(msgData.toString("utf8"));
        const response = await handleRequest(request);
        socket.write(createFramedMessage(response));
      } catch (e) {
        socket.write(
          createFramedMessage({
            success: false,
            message: `Parse error: ${e.message}`,
          })
        );
      }
    }
  });

  socket.on("error", (err) => {
    console.error("Socket error:", err.message);
  });

  socket.on("close", () => {
    // Client disconnected
  });
}

/**
 * Start the IPC server
 */
function startServer() {
  // Remove existing socket file on Unix
  if (process.platform !== "win32") {
    try {
      require("fs").unlinkSync(PIPE_PATH);
    } catch (e) {}
  }

  const server = net.createServer(handleClient);

  server.on("error", (err) => {
    console.error("Server error:", err.message);
    process.exit(1);
  });

  server.listen(PIPE_PATH, () => {
    console.error(`🚀 Ignite Bridge IPC server running on ${PIPE_PATH}`);
    // Signal ready to parent process
    if (process.send) {
      process.send({ ready: true, pipe: PIPE_PATH });
    }
  });

  // Graceful shutdown
  process.on("SIGINT", () => shutdown(server));
  process.on("SIGTERM", () => shutdown(server));
}

function shutdown(server) {
  console.error("⏹️ Shutting down...");
  for (const [id, client] of connections) {
    try {
      client.disconnect();
    } catch (e) {}
  }
  server.close();
  process.exit(0);
}

// Start
startServer();
