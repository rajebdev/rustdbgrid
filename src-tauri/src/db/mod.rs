pub mod ignite;
pub mod mongodb;
pub mod mssql;
pub mod mysql;
pub mod postgres;
pub mod redis;
pub mod traits;

// Re-export traits and factory functions for easy access
pub use ignite::shutdown_bridge;
