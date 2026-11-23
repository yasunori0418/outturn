//! item の id の導出（spec §5）。
//!
//! id = lowercase-hex( sha256( JCS( identity ) ) )
//!
//! identity の値域は spec §5 に従う。文字列 / 整数 ±(2^53−1) / bool / null /
//! 配列 / オブジェクト（メンバー名は ASCII）に限り、域外は id の導出時に拒む。

use std::collections::BTreeMap;
use std::fmt;

/// identity の key。spec §5 の値域にある JSON 値を表す。
///
/// 浮動小数は型で表せない。整数の値域 ±(2^53−1) とメンバー名の ASCII 制約は
/// 型では表さず、[`derive_id`] が検査する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Null,
    Bool(bool),
    Int(i64),
    String(String),
    Array(Vec<Key>),
    Object(BTreeMap<String, Key>),
}

/// item の id の導出元。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub kind: String,
    pub key: Key,
}

/// key の読み込みと id の導出の失敗。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdError {
    message: String,
}

impl fmt::Display for IdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "outturn: {}", self.message)
    }
}

impl std::error::Error for IdError {}
