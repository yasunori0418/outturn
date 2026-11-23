//! outturn 規格（specVersion 1）の Rust 参照実装。
//!
//! エンベロープの型と、item の id の導出（spec §5）を提供する。

mod envelope;
mod id;

pub use envelope::{
    Change, ChangeKind, Envelope, Error, Generation, Item, ItemStatus, ResultBody, Status,
    Subject, SubjectResult, Tool, Warning,
};
pub use id::{derive_id, IdError, Identity, Key};
