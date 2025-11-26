//! Service composer module for ZenTao MCP Server

pub mod service_composer;
pub mod module_registry;

pub use service_composer::ServiceComposer;
pub use module_registry::{ServiceRegistry, DynamicModule};