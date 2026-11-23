//! エンベロープと構成型（specVersion 1）。
//!
//! フィールドの必須と任意は `schema/v1/envelope.schema.json` に合わせる。
//! ツール固有の情報（`info`）の型は型引数で受け、既定は `serde_json::Value` とする。
//! 読み込みでは規格型に無いフィールドを無視する（spec §2 の must-ignore）。
//! 書き出しでは、任意の配列は空のとき、`Option` は `None` のとき出力しない。
//! 必須の配列（`results`・`items`）は空でも出力する。

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// エンベロープと主体ごとの結果の成否。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Error,
}

/// item の成否。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Success,
    Failed,
    Skipped,
}

/// 変更の種類。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Add,
    Remove,
    Modify,
}

/// 結果を出力したツール。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub version: String,
}

/// 操作の主体を名指す弱い識別子。id の導出には関わらない。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
}

/// エラー。`code` は `E_` で始まる。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Error {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Map<String, Value>>,
}

/// 警告。構造は [`Error`] と同じで、`code` は `W_` で始まる。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warning {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Map<String, Value>>,
}

/// 処理の単位。`id` は [`crate::derive_id`] で導出する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item<TItem = Value> {
    pub id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub status: ItemStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Warning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<TItem>,
}

/// item に加えた変更。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change<TChange = Value> {
    pub kind: ChangeKind,
    pub item_id: String,
    pub reversible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<TChange>,
}

/// 1 主体分の結果の本体（`result`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultBody<TItem = Value, TChange = Value, TInfo = Value> {
    pub items: Vec<Item<TItem>>,
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<Change<TChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<TInfo>,
}

/// profile の世代の移り変わりの観測記録。観測できない世代番号は `None` とする。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generation {
    pub profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<u64>,
}

/// `results` の要素。1 主体分の実行結果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectResult<TItem = Value, TChange = Value, TInfo = Value> {
    pub subject: Subject,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<Generation>,
    pub started_at: String,
    pub finished_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Error>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Warning>,
    pub result: ResultBody<TItem, TChange, TInfo>,
}

/// 結果エンベロープ。ツールが stdout に出力する単一の JSON 文書。
///
/// `info` は実行全体のツール固有情報で、主体ごとの情報は [`ResultBody`] の `info` に置く。
/// `working_directory` には実行時の作業ディレクトリを絶対パスで入れる。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope<TItem = Value, TChange = Value, TInfo = Value, TEnvInfo = Value> {
    pub spec_version: u32,
    pub tool: Tool,
    pub command: String,
    pub status: Status,
    pub dry_run: bool,
    pub started_at: String,
    pub finished_at: String,
    pub working_directory: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Error>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Warning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<TEnvInfo>,
    pub results: Vec<SubjectResult<TItem, TChange, TInfo>>,
}
