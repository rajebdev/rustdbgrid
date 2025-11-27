pub mod ignite_node;
pub mod mongodb;
pub mod mssql;
pub mod mysql;
pub mod postgres;
pub mod redis;
pub mod traits;

// Re-export shutdown function for easy access
pub use ignite_node::shutdown_bridge;
