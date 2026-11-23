//! outturn 規格（specVersion 1）の Rust 参照実装。
//!
//! item の id の導出（spec §5）を提供する。

mod id;

pub use id::{IdError, Identity, Key};
