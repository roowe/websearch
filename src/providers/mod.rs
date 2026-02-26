//! Search provider implementations

pub mod arxiv;
pub mod duckduckgo;

// Re-export providers for convenience
pub use arxiv::ArxivProvider;
pub use duckduckgo::DuckDuckGoProvider;
