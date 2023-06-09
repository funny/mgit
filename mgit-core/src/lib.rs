// https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html#no-more-modrs

pub mod core;
pub mod ops;
pub mod option;
pub mod utils;

#[cfg(feature = "test-helper")]
pub mod tests;
