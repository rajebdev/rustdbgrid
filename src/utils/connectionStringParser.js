/**
 * Parse connection string and extract connection parameters
 * Supports multiple database types: MySQL, PostgreSQL, MongoDB, Redis, MSSQL
 */

export function parseConnectionString(connectionString, dbType) {
  try {
    const str = connectionString.trim();
    const result = {
      host: "localhost",
      port: getDefaultPort(dbType),
      username: "",
      password: "",
      database: "",
    };

    if (dbType === "MySQL") {
      // JDBC: jdbc:mysql://host:port/database or mysql://user:password@host:port/database
      let match = str.match(
        /^jdbc:mysql:\/\/([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
      );
      if (match) {
        result.host = match[1];
        result.port = match[2] ? parseInt(match[2]) : 3306;
        if (match[3]) result.database = match[3];
      } else {
        match = str.match(
          /^mysql:\/\/(?:([^:]+):([^@]+)@)?([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
        );
        if (match) {
          if (match[1]) result.username = decodeURIComponent(match[1]);
          if (match[2]) result.password = decodeURIComponent(match[2]);
          result.host = match[3];
          result.port = match[4] ? parseInt(match[4]) : 3306;
          if (match[5]) result.database = match[5];
        }
      }
    } else if (dbType === "PostgreSQL") {
      // JDBC: jdbc:postgresql://host:port/database or postgresql://user:password@host:port/database
      let match = str.match(
        /^jdbc:postgresql:\/\/([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
      );
      if (match) {
        result.host = match[1];
        result.port = match[2] ? parseInt(match[2]) : 5432;
        if (match[3]) result.database = match[3];
      } else {
        match = str.match(
          /^postgres(?:ql)?:\/\/(?:([^:]+):([^@]+)@)?([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
        );
        if (match) {
          if (match[1]) result.username = decodeURIComponent(match[1]);
          if (match[2]) result.password = decodeURIComponent(match[2]);
          result.host = match[3];
          result.port = match[4] ? parseInt(match[4]) : 5432;
          if (match[5]) result.database = match[5];
        }
      }
    } else if (dbType === "MongoDB") {
      // mongodb://user:password@host:port/database or mongodb+srv://...
      const match = str.match(
        /^mongodb(?:\+srv)?:\/\/(?:([^:]+):([^@]+)@)?([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
      );
      if (match) {
        if (match[1]) result.username = decodeURIComponent(match[1]);
        if (match[2]) result.password = decodeURIComponent(match[2]);
        result.host = match[3];
        result.port = match[4] ? parseInt(match[4]) : 27017;
        if (match[5]) result.database = match[5];
      }
    } else if (dbType === "Redis") {
      // redis://[:password@]host:port[/database]
      const match = str.match(
        /^redis:\/\/(?::([^@]+)@)?([^:\/]+)(?::(\d+))?(?:\/(\d+))?/
      );
      if (match) {
        if (match[1]) result.password = decodeURIComponent(match[1]);
        result.host = match[2];
        result.port = match[3] ? parseInt(match[3]) : 6379;
        if (match[4]) result.database = match[4];
      }
    } else if (dbType === "MSSQL") {
      // mssql://user:password@host:port/database or jdbc:sqlserver://host:port;databaseName=database
      let match = str.match(
        /^jdbc:sqlserver:\/\/([^:;]+)(?::(\d+))?(?:;databaseName=([^;]+))?/
      );
      if (match) {
        result.host = match[1];
        result.port = match[2] ? parseInt(match[2]) : 1433;
        if (match[3]) result.database = match[3];
      } else {
        match = str.match(
          /^mssql:\/\/(?:([^:]+):([^@]+)@)?([^:\/]+)(?::(\d+))?(?:\/([^?]+))?/
        );
        if (match) {
          if (match[1]) result.username = decodeURIComponent(match[1]);
          if (match[2]) result.password = decodeURIComponent(match[2]);
          result.host = match[3];
          result.port = match[4] ? parseInt(match[4]) : 1433;
          if (match[5]) result.database = match[5];
        }
      }
    }

    return { success: true, data: result };
  } catch (error) {
    return {
      success: false,
      error: "Failed to parse connection string: " + error.message,
    };
  }
}

/**
 * Get default port for database type
 */
export function getDefaultPort(dbType) {
  const ports = {
    MySQL: 3306,
    PostgreSQL: 5432,
    MongoDB: 27017,
    Redis: 6379,
    Ignite: 10800,
    MSSQL: 1433,
  };
  return ports[dbType] || 3306;
}

/**
 * Get connection string format examples for a database type
 */
export function getConnectionStringFormats(dbType) {
  const formats = {
    MySQL:
      "mysql://user:password@host:port/database or jdbc:mysql://host:port/database",
    PostgreSQL:
      "postgresql://user:password@host:port/database or jdbc:postgresql://host:port/database",
    MongoDB: "mongodb://user:password@host:port/database",
    Redis: "redis://:password@host:port/database",
    MSSQL:
      "mssql://user:password@host:port/database or jdbc:sqlserver://host:port;databaseName=database",
    Ignite: "Connection string not supported for Ignite",
  };
  return formats[dbType] || "Connection string format varies by database type";
}
