// Server module declaration
pub mod handlers;
pub mod routes;

// Re-export necessary items
// pub use handlers::*; // Not directly used in main.rs
#[allow(unused_imports)] // Used indirectly via main.rs
pub use routes::configure_routes;
