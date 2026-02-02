//! Async API Aggregator Library
//!
//! Provides concurrent fetching from multiple REST APIs with:
//! - Parallel execution using `futures::join_all`
//! - Proper error handling with custom error types
//! - Timeouts and retries
//! - Result aggregation with partial failure support

mod aggregator;

pub use aggregator::*;
