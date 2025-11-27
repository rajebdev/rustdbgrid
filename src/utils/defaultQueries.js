/**
 * Get default query template based on database type
 * @param {string} dbType - Type of database connection
 * @returns {string} Default query template
 */
export function getDefaultQuery(dbType) {
  const queries = {
    MySQL: `-- MySQL Query
SELECT * FROM table_name LIMIT 100;`,

    PostgreSQL: `-- PostgreSQL Query
SELECT * FROM table_name LIMIT 100;`,

    MSSQL: `-- SQL Server Query
SELECT TOP 100 * FROM table_name;`,

    MongoDB: `// MongoDB Query
db.collection_name.find({}).limit(100)`,

    Redis: `// Redis Commands
KEYS *
GET key_name
HGETALL hash_name`,

    "Apache Ignite": `-- Apache Ignite Query
SELECT * FROM table_name LIMIT 100;`,

    SQLite: `-- SQLite Query
SELECT * FROM table_name LIMIT 100;`,

    MariaDB: `-- MariaDB Query
SELECT * FROM table_name LIMIT 100;`,

    Oracle: `-- Oracle Query
SELECT * FROM table_name WHERE ROWNUM <= 100;`,
  };

  return queries[dbType] || `-- SQL Query\nSELECT * FROM table_name LIMIT 100;`;
}

/**
 * Get placeholder/example queries for different database types
 * @param {string} dbType - Type of database connection
 * @returns {object} Object with various query examples
 */
export function getQueryExamples(dbType) {
  const examples = {
    MySQL: {
      select: "SELECT * FROM users WHERE age > 18 LIMIT 10;",
      insert:
        "INSERT INTO users (name, email, age) VALUES ('John', 'john@example.com', 25);",
      update: "UPDATE users SET email = 'newemail@example.com' WHERE id = 1;",
      delete: "DELETE FROM users WHERE id = 1;",
      join: "SELECT u.name, o.order_date FROM users u JOIN orders o ON u.id = o.user_id LIMIT 10;",
    },

    PostgreSQL: {
      select: "SELECT * FROM users WHERE age > 18 LIMIT 10;",
      insert:
        "INSERT INTO users (name, email, age) VALUES ('John', 'john@example.com', 25);",
      update: "UPDATE users SET email = 'newemail@example.com' WHERE id = 1;",
      delete: "DELETE FROM users WHERE id = 1;",
      join: "SELECT u.name, o.order_date FROM users u JOIN orders o ON u.id = o.user_id LIMIT 10;",
    },

    MSSQL: {
      select: "SELECT TOP 10 * FROM users WHERE age > 18;",
      insert:
        "INSERT INTO users (name, email, age) VALUES ('John', 'john@example.com', 25);",
      update: "UPDATE users SET email = 'newemail@example.com' WHERE id = 1;",
      delete: "DELETE FROM users WHERE id = 1;",
      join: "SELECT TOP 10 u.name, o.order_date FROM users u JOIN orders o ON u.id = o.user_id;",
    },

    MongoDB: {
      find: "db.users.find({ age: { $gt: 18 } }).limit(10)",
      insert:
        "db.users.insertOne({ name: 'John', email: 'john@example.com', age: 25 })",
      update:
        "db.users.updateOne({ _id: ObjectId('...') }, { $set: { email: 'newemail@example.com' } })",
      delete: "db.users.deleteOne({ _id: ObjectId('...') })",
      aggregate:
        "db.orders.aggregate([{ $lookup: { from: 'users', localField: 'user_id', foreignField: '_id', as: 'user' } }])",
    },

    Redis: {
      get: "GET mykey",
      set: "SET mykey 'myvalue'",
      hgetall: "HGETALL myhash",
      hset: "HSET myhash field1 'value1' field2 'value2'",
      keys: "KEYS pattern*",
      del: "DEL mykey",
      exists: "EXISTS mykey",
      expire: "EXPIRE mykey 3600",
      ttl: "TTL mykey",
      list: "LRANGE mylist 0 -1",
      set_operations: "SADD myset member1 member2\nSMEMBERS myset",
    },

    "Apache Ignite": {
      select: "SELECT * FROM users WHERE age > 18 LIMIT 10;",
      insert:
        "INSERT INTO users (id, name, email, age) VALUES (1, 'John', 'john@example.com', 25);",
      update: "UPDATE users SET email = 'newemail@example.com' WHERE id = 1;",
      delete: "DELETE FROM users WHERE id = 1;",
      join: "SELECT u.name, o.order_date FROM users u JOIN orders o ON u.id = o.user_id LIMIT 10;",
    },
  };

  return examples[dbType] || examples.MySQL;
}

/**
 * Build query with pagination based on database type
 * @param {string} dbType - Type of database connection
 * @param {string} baseQuery - Original query without pagination
 * @param {number} limit - Number of rows to fetch
 * @param {number} offset - Number of rows to skip
 * @returns {string} Query with proper pagination syntax
 */
export function buildPaginatedQuery(
  dbType,
  baseQuery,
  limit = 200,
  offset = 0
) {
  // Remove trailing semicolon if present
  baseQuery = baseQuery.trim().replace(/;$/, "");

  // Remove existing pagination clauses
  switch (dbType) {
    case "MSSQL":
      // Remove existing TOP clause
      baseQuery = baseQuery.replace(/\s+TOP\s+\d+/gi, "");

      // Remove existing OFFSET-FETCH if present
      baseQuery = baseQuery.replace(
        /\s+OFFSET\s+\d+\s+ROWS(\s+FETCH\s+NEXT\s+\d+\s+ROWS\s+ONLY)?/gi,
        ""
      );

      // For first page, use TOP (simple and efficient)
      if (offset === 0) {
        return baseQuery.replace(/SELECT/i, `SELECT TOP ${limit}`);
      }

      // For subsequent pages, use ROW_NUMBER() window function
      // This works without requiring ORDER BY in the original query
      // Wrap the query in a subquery with ROW_NUMBER()
      const selectMatch = baseQuery.match(/SELECT\s+/i);
      if (selectMatch) {
        // Insert ROW_NUMBER() after SELECT
        const withRowNumber = baseQuery.replace(
          /SELECT\s+/i,
          `SELECT ROW_NUMBER() OVER (ORDER BY (SELECT NULL)) AS __RowNum, `
        );

        // Wrap in outer query to filter by row number
        const result = `SELECT * FROM (${withRowNumber}) AS __Paginated WHERE __RowNum > ${offset} AND __RowNum <= ${
          offset + limit
        }`;
        return result;
      }

      // Fallback: just add TOP if we can't parse
      const fallback = baseQuery.replace(/SELECT/i, `SELECT TOP ${limit}`);
      return fallback;

    case "Oracle":
      // Oracle uses ROWNUM or OFFSET-FETCH (12c+)
      // Remove existing ROWNUM conditions
      baseQuery = baseQuery.replace(/WHERE\s+ROWNUM\s*<=?\s*\d+/gi, "");

      // Use modern OFFSET-FETCH syntax (Oracle 12c+)
      if (offset === 0) {
        return `${baseQuery} FETCH FIRST ${limit} ROWS ONLY`;
      } else {
        return `${baseQuery} OFFSET ${offset} ROWS FETCH NEXT ${limit} ROWS ONLY`;
      }

    case "MongoDB":
      // MongoDB doesn't use SQL LIMIT, return as-is (handled differently in backend)
      return baseQuery;

    case "Redis":
      // Redis doesn't use SQL LIMIT, return as-is
      return baseQuery;

    case "MySQL":
    case "PostgreSQL":
    case "SQLite":
    case "Ignite":
    default:
      // Standard SQL LIMIT OFFSET syntax
      // Remove existing LIMIT/OFFSET
      baseQuery = baseQuery.replace(/\s+LIMIT\s+\d+(\s+OFFSET\s+\d+)?/gi, "");
      baseQuery = baseQuery.replace(/\s+OFFSET\s+\d+/gi, "");

      if (offset > 0) {
        return `${baseQuery} LIMIT ${limit} OFFSET ${offset}`;
      } else {
        return `${baseQuery} LIMIT ${limit}`;
      }
  }
}
