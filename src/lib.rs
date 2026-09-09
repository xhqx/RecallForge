//! Evidence-backed local memory. SQLite is authoritative; all clients share validation.
pub mod embedding;
pub mod mcp;
pub mod model;
pub mod search;
pub mod service;
pub mod store;

mod graph;
mod maintenance;
mod mcp_schema;
mod registry;
