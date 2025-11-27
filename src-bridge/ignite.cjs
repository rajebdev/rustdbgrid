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

// =============================================================================
// Unified Logging - All logs go to stdout with consistent format
// =============================================================================
const LOG_LEVELS = {
  DEBUG: "DEBUG",
  INFO: "INFO",
  WARN: "WARN",
  ERROR: "ERROR",
};

/**
 * Get caller info (file and line number)
 */
function getCallerInfo() {
  const err = new Error();
  const stack = err.stack.split("\n");
  // stack[0] = "Error", stack[1] = getCallerInfo, stack[2] = formatLog, stack[3] = log.xxx, stack[4] = actual caller
  const callerLine = stack[4] || stack[3] || "";
  const match = callerLine.match(/at\s+(?:(.+?)\s+\()?(?:(.+?):(\d+):(\d+))/);
  if (match) {
    const fn = match[1] || "anonymous";
    const file = match[2] || "";
    const line = match[3] || "";
    // Extract just filename from path
    const fileName = file.split(/[/\\]/).pop() || file;
    return `${fileName};${fn}:${line}`;
  }
  return "ignite.cjs;unknown";
}

function formatLog(level, message) {
  const now = new Date();
  const timestamp =
    now.getFullYear() +
    "-" +
    String(now.getMonth() + 1).padStart(2, "0") +
    "-" +
    String(now.getDate()).padStart(2, "0") +
    " " +
    String(now.getHours()).padStart(2, "0") +
    ":" +
    String(now.getMinutes()).padStart(2, "0") +
    ":" +
    String(now.getSeconds()).padStart(2, "0") +
    "." +
    String(now.getMilliseconds()).padStart(3, "0");

  const caller = getCallerInfo();
  // Format: [TIMESTAMP][file;fn:line][BRIDGE][LEVEL] message
  return `[${timestamp}][${caller}][BRIDGE][${level}] ${message}`;
}

const log = {
  debug: (msg) => console.log(formatLog(LOG_LEVELS.DEBUG, msg)),
  info: (msg) => console.log(formatLog(LOG_LEVELS.INFO, msg)),
  warn: (msg) => console.log(formatLog(LOG_LEVELS.WARN, msg)),
  error: (msg) => console.log(formatLog(LOG_LEVELS.ERROR, msg)),
};

// Pipe/Socket path - requires IGNITE_BRIDGE_PIPE from parent process
const PIPE_NAME = process.env.IGNITE_BRIDGE_PIPE;
if (!PIPE_NAME) {
  log.error("IGNITE_BRIDGE_PIPE environment variable is required");
  process.exit(1);
}
const PIPE_PATH =
  process.platform === "win32"
    ? `\\\\.\\pipe\\${PIPE_NAME}`
    : `/tmp/${PIPE_NAME}.sock`;

// Store active connections
const connections = new Map();

// Track shutdown timer to avoid multiple timers
let shutdownTimer = null;

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
  // Cancel any pending shutdown timer since we're getting a new connection
  if (shutdownTimer) {
    clearTimeout(shutdownTimer);
    shutdownTimer = null;
    log.info("📊 [CONNECT] Cancelled pending shutdown timer");
  }

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
  log.info(`✅ Connected: ${connectionId} (total: ${connections.size})`);

  return { success: true, message: "Connected to Ignite" };
}

async function handleDisconnect({ connectionId }) {
  log.info(`🔌 [DISCONNECT] Received disconnect request for: ${connectionId}`);

  if (connections.has(connectionId)) {
    try {
      const client = connections.get(connectionId);
      if (client) {
        client.disconnect();
      }
    } catch (e) {
      log.warn(`⚠️ [DISCONNECT] Error during client disconnect: ${e.message}`);
    }
    connections.delete(connectionId);
    log.info(`✅ [DISCONNECT] Removed connection: ${connectionId}`);
  } else {
    log.warn(`⚠️ [DISCONNECT] Connection not found: ${connectionId}`);
  }

  log.info(`📊 [DISCONNECT] Active connections remaining: ${connections.size}`);

  // Log all remaining connections for debugging
  if (connections.size > 0) {
    log.debug(
      `📋 [DISCONNECT] Remaining connection IDs: ${Array.from(
        connections.keys()
      ).join(", ")}`
    );
  }

  // Auto-shutdown if no connections after a delay (allows reconnection)
  if (connections.size === 0) {
    // Clear any existing shutdown timer to avoid multiple timers
    if (shutdownTimer) {
      clearTimeout(shutdownTimer);
      shutdownTimer = null;
    }

    log.info(
      "📭 [DISCONNECT] No active connections, will shutdown in 10 seconds if no new connections..."
    );
    shutdownTimer = setTimeout(() => {
      if (connections.size === 0) {
        log.info(
          "📭 [DISCONNECT] Still no connections, shutting down bridge process..."
        );
        process.exit(0);
      } else {
        log.info(
          `📊 [DISCONNECT] New connection established (${connections.size}), staying alive`
        );
      }
      shutdownTimer = null;
    }, 10000); // 10 second grace period (reduced from 30)
  } else if (shutdownTimer) {
    // Cancel shutdown timer if new connections exist
    clearTimeout(shutdownTimer);
    shutdownTimer = null;
    log.info(
      "📊 [DISCONNECT] Shutdown timer cancelled, connections still active"
    );
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

  // Retry logic for transient Ignite errors
  const maxRetries = 3;
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
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
      // Check if this is a transient schema/thread error
      const isSchemaError =
        sqlError.message &&
        (sqlError.message.includes("Failed to set schema") ||
          sqlError.message.includes("for thread"));

      if (isSchemaError && attempt < maxRetries) {
        log.warn(
          `⚠️ Schema error on getTables attempt ${attempt}, retrying in ${
            attempt * 100
          }ms...`
        );
        await new Promise((resolve) => setTimeout(resolve, attempt * 100));
        continue;
      }

      // Return cache as fallback table
      return {
        success: true,
        tables: [{ name: cacheName, type: "cache" }],
      };
    }
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

  // Get available caches for smart schema detection
  let availableCaches = [];
  try {
    availableCaches = await client.cacheNames();
  } catch (e) {
    log.warn(`⚠️ [QUERY] Failed to get cache names: ${e.message}`);
  }

  // If query is a simple SELECT without schema prefix, try to find the correct cache/schema
  let modifiedQuery = query;
  let targetCache = cacheName || "PUBLIC";

  if (queryUpper.startsWith("SELECT") && queryUpper.includes("FROM")) {
    const fromMatch = query.match(/\bFROM\s+["']?(\w+)["']?/i);
    if (fromMatch) {
      const tableName = fromMatch[1];
      // Check if tableName doesn't already have schema prefix (no dot in name)
      if (
        !query.includes(".") ||
        !query.match(/\bFROM\s+["']?\w+["']?\.["']?\w+["']?/i)
      ) {
        // Try to find a cache with the same name as the table
        // In Ignite, cache name often equals table name
        const matchingCache = availableCaches.find(
          (c) => c.toUpperCase() === tableName.toUpperCase()
        );

        if (matchingCache) {
          // Use the matching cache as schema
          targetCache = matchingCache;
          modifiedQuery = query.replace(
            new RegExp(`\\bFROM\\s+["']?${tableName}["']?`, "i"),
            `FROM "${matchingCache}"."${tableName}"`
          );
          log.debug(
            `📝 [QUERY] Found matching cache, using: "${matchingCache}"."${tableName}"`
          );
        } else if (cacheName) {
          // Use provided cacheName as schema
          modifiedQuery = query.replace(
            new RegExp(`\\bFROM\\s+["']?${tableName}["']?`, "i"),
            `FROM "${cacheName}"."${tableName}"`
          );
          log.debug(
            `📝 [QUERY] Added provided cache as schema: "${cacheName}"."${tableName}"`
          );
        } else {
          // No matching cache found, try using table name as both cache and table
          // This is common in Ignite where cache name = table name
          if (availableCaches.length > 0) {
            // First try: use first cache as schema (might work for some setups)
            targetCache = availableCaches[0];
            modifiedQuery = query.replace(
              new RegExp(`\\bFROM\\s+["']?${tableName}["']?`, "i"),
              `FROM "${tableName}"."${tableName}"`
            );
            log.debug(
              `📝 [QUERY] No matching cache, trying: "${tableName}"."${tableName}"`
            );
          }
        }
      }
    }
  } else {
    // Query already has schema prefix like "SCHEMA"."TABLE"
    // Check if schema exists, if not try to fix it
    const schemaTableMatch = query.match(
      /\bFROM\s+["']?(\w+)["']?\.["']?(\w+)["']?/i
    );
    if (schemaTableMatch) {
      const [, schemaName, tableName] = schemaTableMatch;

      // Check if schema (cache) exists
      if (
        !availableCaches.some(
          (c) => c.toUpperCase() === schemaName.toUpperCase()
        )
      ) {
        // Schema doesn't exist, try to find matching cache by table name
        const matchingCache = availableCaches.find(
          (c) => c.toUpperCase() === tableName.toUpperCase()
        );

        if (matchingCache) {
          // Replace with correct schema
          modifiedQuery = query.replace(
            new RegExp(
              `\\bFROM\\s+["']?${schemaName}["']?\\.["']?${tableName}["']?`,
              "i"
            ),
            `FROM "${matchingCache}"."${tableName}"`
          );
          targetCache = matchingCache;
          log.debug(
            `📝 [QUERY] Fixed schema: "${schemaName}"."${tableName}" -> "${matchingCache}"."${tableName}"`
          );
        }
      }
    }
  }

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

  // Use modified query with schema prefix
  const sqlQuery = new SqlFieldsQuery(modifiedQuery);
  sqlQuery.setIncludeFieldNames(true);

  // Retry logic for transient Ignite errors (schema/thread errors)
  const maxRetries = 3;
  let cursor = null;
  let allRows = null;
  let lastError = null;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      cursor = await cache.query(sqlQuery);
      allRows = await cursor.getAll();
      lastError = null;
      break; // Success
    } catch (queryError) {
      lastError = queryError;

      // Check if this is a transient schema/thread error or table not found
      const isSchemaError =
        queryError.message &&
        (queryError.message.includes("Failed to set schema") ||
          queryError.message.includes("for thread"));
      const isTableNotFound =
        queryError.message &&
        queryError.message.includes("Table") &&
        queryError.message.includes("not found");
      const isSchemaNotFound =
        queryError.message &&
        queryError.message.includes("Schema") &&
        queryError.message.includes("not found");

      if (isSchemaError && attempt < maxRetries) {
        log.warn(
          `⚠️ Schema error on query attempt ${attempt}, retrying in ${
            attempt * 100
          }ms...`
        );
        await new Promise((resolve) => setTimeout(resolve, attempt * 100));
        continue;
      }

      // For schema not found, this cache might not have SQL tables
      // Suggest using SCAN instead
      if (isSchemaNotFound) {
        const cacheNames = await client.cacheNames();
        return {
          success: false,
          message: `${
            queryError.message
          }\n\nThis cache may not have SQL tables defined. Try using SCAN instead:\nSCAN ${targetCache}\n\nAvailable caches: ${cacheNames.join(
            ", "
          )}`,
        };
      }

      // For table not found, provide helpful error message
      if (isTableNotFound) {
        const cacheNames = await client.cacheNames();
        return {
          success: false,
          message: `${
            queryError.message
          }\n\nAvailable caches/schemas: ${cacheNames.join(
            ", "
          )}\n\nTip: Use fully qualified table name like "CACHE_NAME"."TABLE_NAME" or use SCAN for key-value access`,
        };
      }

      // Non-retryable error or last attempt
      break;
    }
  }

  if (lastError) {
    return {
      success: false,
      message: `Query failed: ${lastError.message}`,
    };
  }

  let columns = [];
  const rows = [];

  if (allRows && allRows.length > 0) {
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
      finalQuery: modifiedQuery, // Return the actual query executed (with schema prefix if added)
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

  // Retry logic for transient Ignite errors
  const maxRetries = 3;
  let lastError = null;

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      cache = client.getCache(cacheName);
      const scanQuery = new ScanQuery();
      // Set a reasonable page size to avoid timeout on large datasets
      scanQuery.setPageSize(Math.min(limit + offset + 100, 5000));
      cursor = await cache.query(scanQuery);

      // Use getAll but with proper error handling
      entries = await cursor.getAll();
      lastError = null;
      break; // Success, exit retry loop
    } catch (scanError) {
      lastError = scanError;

      // Check if this is a transient schema/thread error that can be retried
      const isSchemaError =
        scanError.message &&
        (scanError.message.includes("Failed to set schema") ||
          scanError.message.includes("for thread"));

      // Handle cache-related errors with retry using getOrCreateCache
      if (
        scanError.message &&
        scanError.message.includes("Cache does not exist")
      ) {
        try {
          cache = await client.getOrCreateCache(cacheName);
          const scanQuery = new ScanQuery();
          scanQuery.setPageSize(Math.min(limit + offset + 100, 5000));
          cursor = await cache.query(scanQuery);
          entries = await cursor.getAll();
          lastError = null;
          break; // Success
        } catch (retryError) {
          lastError = retryError;
        }
      } else if (isSchemaError && attempt < maxRetries) {
        // Wait a bit before retry for schema/thread errors
        log.warn(
          `⚠️ Schema error on attempt ${attempt}, retrying in ${
            attempt * 100
          }ms...`
        );
        await new Promise((resolve) => setTimeout(resolve, attempt * 100));
        continue;
      }

      // If not a retryable error or last attempt, break
      if (!isSchemaError || attempt === maxRetries) {
        break;
      }
    }
  }

  // Check if we succeeded after retries
  if (lastError) {
    return {
      success: false,
      message: `Scan failed: ${lastError.message}`,
    };
  }

  if (!entries) {
    return {
      success: false,
      message: `Scan failed: No entries returned`,
    };
  }

  const rows = [];
  const columnsSet = new Set(["_key"]);

  // Calculate slice indices
  const startIdx = Math.min(offset, entries.length);
  const endIdx = Math.min(offset + limit, entries.length);

  for (let i = startIdx; i < endIdx; i++) {
    try {
      const entry = entries[i];
      if (!entry) continue;

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
    } catch (entryError) {
      log.warn(`Error processing entry at index ${i}: ${entryError.message}`);
      // Continue with next entry instead of failing completely
      continue;
    }
  }

  return {
    success: true,
    result: {
      columns: Array.from(columnsSet),
      rows,
      totalCount: entries.length,
      hasMore: endIdx < entries.length,
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

  // Helper function to execute query with retry
  async function executeWithRetry(cache, queryObj, maxRetries = 3) {
    let lastError = null;
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        const cursor = await cache.query(queryObj);
        const rows = await cursor.getAll();
        return rows;
      } catch (error) {
        lastError = error;
        const isSchemaError =
          error.message &&
          (error.message.includes("Failed to set schema") ||
            error.message.includes("for thread"));

        if (isSchemaError && attempt < maxRetries) {
          log.warn(
            `⚠️ Schema error on attempt ${attempt}, retrying in ${
              attempt * 100
            }ms...`
          );
          await new Promise((resolve) => setTimeout(resolve, attempt * 100));
          continue;
        }
        break;
      }
    }
    throw lastError;
  }

  try {
    const cache = client.getCache(cacheName);
    const table = tableName || cacheName;

    const query = new SqlFieldsQuery(
      "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_DEFAULT FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_NAME = ?"
    ).setArgs(table.toUpperCase());

    const rows = await executeWithRetry(cache, query);

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
    // Infer schema from data using ScanQuery with retry
    try {
      const cache = client.getCache(cacheName);
      const entries = await executeWithRetry(cache, new ScanQuery());

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
    } catch (scanError) {
      return {
        success: false,
        message: `Failed to get schema: ${scanError.message}`,
      };
    }
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
    log.error(`Socket error: ${err.message}`);
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
    log.error(`Server error: ${err.message}`);
    process.exit(1);
  });

  server.listen(PIPE_PATH, () => {
    log.info(`🚀 Ignite Bridge IPC server running on ${PIPE_PATH}`);
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
  log.info("⏹️ Shutting down...");
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
