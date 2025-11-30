/**
 * Unified Logger for Frontend
 * Sends all logs to Rust's log system via custom invoke command
 * so they appear in stdout alongside Rust and Bridge logs
 */
import { invoke } from "@tauri-apps/api/core";

// Store original console methods for fallback (non-Tauri env)
const originalConsole = {
  log: console.log.bind(console),
  debug: console.debug.bind(console),
  info: console.info.bind(console),
  warn: console.warn.bind(console),
  error: console.error.bind(console),
};

/**
 * Get timestamp in format: yyyy-mm-dd hh:ii:ss.zzz
 */
function getTimestamp() {
  const now = new Date();
  // ISO string with milliseconds and Z
  // Example: 2025-11-30T19:39:16.549299Z
  // JS only supports milliseconds, so pad with 000 for microseconds
  const iso = now.toISOString(); // 2025-11-30T19:39:16.549Z
  // Add extra zeros for microseconds
  return iso.replace("Z", "000Z");
}

/**
 * Get caller info from stack trace
 */
function getCallerInfo() {
  const err = new Error();
  const stack = err.stack.split("\n");
  // Find first non-logger caller
  for (let i = 3; i < stack.length; i++) {
    const line = stack[i];
    if (!line.includes("logger.js")) {
      const match = line.match(/at\s+(?:(.+?)\s+\()?(?:(.+?):(\d+):(\d+))/);
      if (match) {
        const fn = match[1] || "anonymous";
        const file = match[2] || "";
        const lineNum = match[3] || "";
        // Extract just filename from path/URL
        let fileName = file.split(/[/\\]/).pop() || file;
        // Remove query params if any
        fileName = fileName.split("?")[0];
        return `${fileName}::${fn}:${lineNum}`;
      }
    }
  }
  return "unknown::unknown";
}

/**
 * Format message for logging (handles objects, arrays, etc.)
 */
function formatMessage(...args) {
  return args
    .map((arg) => {
      if (typeof arg === "string") return arg;
      if (arg instanceof Error) return `${arg.name}: ${arg.message}`;
      try {
        return JSON.stringify(arg);
      } catch {
        return String(arg);
      }
    })
    .join(" ");
}

/**
 * Format full log line
 */
function formatLogLine(level, message) {
  const timestamp = getTimestamp();
  const caller = getCallerInfo();
  // Pad level to 5 chars (Rust style)
  const paddedLevel = level.toUpperCase().padStart(5, " ");
  // Format: 2025-11-30T19:39:16.549299Z  INFO  rustdbgrid::utils::FE: 147: message
  return `${timestamp} ${paddedLevel} ${caller}: ${message}`;
}

/**
 * Send log to Rust via invoke
 */
function sendToRust(level, formattedMessage) {
  if (window.__TAURI__) {
    // Send already formatted message, Rust just prints it
    invoke("log_from_frontend", { level, message: formattedMessage }).catch(
      () => {}
    );
  }
}

/**
 * Log functions that send to Rust stdout
 */
export const log = {
  trace: (...args) => {
    const msg = formatLogLine("TRACE", formatMessage(...args));
    sendToRust("trace", msg);
    originalConsole.log(msg);
  },
  debug: (...args) => {
    const msg = formatLogLine("DEBUG", formatMessage(...args));
    sendToRust("debug", msg);
    originalConsole.debug(msg);
  },
  info: (...args) => {
    const msg = formatLogLine("INFO", formatMessage(...args));
    sendToRust("info", msg);
    originalConsole.info(msg);
  },
  warn: (...args) => {
    const msg = formatLogLine("WARN", formatMessage(...args));
    sendToRust("warn", msg);
    originalConsole.warn(msg);
  },
  error: (...args) => {
    const msg = formatLogLine("ERROR", formatMessage(...args));
    sendToRust("error", msg);
    originalConsole.error(msg);
  },
};

/**
 * Override global console to redirect all logs to Rust
 * Call this early in app initialization
 */
export function overrideConsole() {
  if (!window.__TAURI__) return;

  console.log = (...args) => {
    const msg = formatLogLine("INFO", formatMessage(...args));
    sendToRust("info", msg);
    originalConsole.log(...args);
  };

  console.debug = (...args) => {
    const msg = formatLogLine("DEBUG", formatMessage(...args));
    sendToRust("debug", msg);
    originalConsole.debug(...args);
  };

  console.info = (...args) => {
    const msg = formatLogLine("INFO", formatMessage(...args));
    sendToRust("info", msg);
    originalConsole.info(...args);
  };

  console.warn = (...args) => {
    const msg = formatLogLine("WARN", formatMessage(...args));
    sendToRust("warn", msg);
    originalConsole.warn(...args);
  };

  console.error = (...args) => {
    const msg = formatLogLine("ERROR", formatMessage(...args));
    sendToRust("error", msg);
    originalConsole.error(...args);
  };

  // Log that console override is active
  const initMsg = formatLogLine(
    "INFO",
    "✅ Console override active - all logs go to stdout"
  );
  sendToRust("info", initMsg);
}

export default log;
