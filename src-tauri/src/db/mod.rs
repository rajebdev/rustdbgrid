pub mod mysql;
pub mod postgres;
pub mod mssql;
pub mod mongodb;
pub mod redis;
pub mod ignite;
pub mod traits;

// Re-export traits and factory functions for easy access
pub use ignite::shutdown_bridge;
