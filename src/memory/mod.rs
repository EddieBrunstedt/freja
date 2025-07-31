//! Memory management module for user conversation persistence.
//!
//! This module provides functionality to store and retrieve
//! AI-generated summaries of past conversations with users.

pub mod store;
pub use store::MemoryStore;
