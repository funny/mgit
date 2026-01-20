//! MGIT Core Library
//!
//! This library provides core functionality for managing multiple Git repositories.
//! It includes configuration management, Git operations, and repository synchronization.


pub mod config;
pub mod error;
pub mod git;
pub mod ops;
pub mod utils;

pub use ansi_term::{Colour, Style};
