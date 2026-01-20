// https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html#no-more-modrs

pub mod config;
pub mod error;
pub mod git;
pub mod ops;
pub mod utils;

pub use ansi_term::{Colour, Style};
