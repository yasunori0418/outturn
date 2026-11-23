//! item の id を `derive_id` で導出してエンベロープを組み立て、stdout に JSON で出力する例。
//!
//! `cargo run --example conformance` で実行する。時刻と作業ディレクトリは例のための固定値で、
//! 実際のツールでは実行時に観測した値を入れる。

use std::collections::BTreeMap;

use outturn::{
    derive_id, Change, ChangeKind, Envelope, Identity, Item, ItemStatus, Key, ResultBody, Status,
    Subject, SubjectResult, Tool,
};

/// 配置先のパスを key の `path` に入れて、entry の id を導出する。
fn entry_id(path: &str) -> Result<String, outturn::IdError> {
    let key = Key::Object(BTreeMap::from([(
        "path".to_string(),
        Key::String(path.to_string()),
    )]));
    derive_id(&Identity {
        kind: "entry".to_string(),
        key,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let zshrc = entry_id(".zshrc")?;
    let init_lua = entry_id(".config/nvim/init.lua")?;

    let envelope: Envelope = Envelope {
        spec_version: 1,
        tool: Tool {
            name: "example".to_string(),
            version: "0.1.0".to_string(),
        },
        command: "apply".to_string(),
        status: Status::Success,
        dry_run: false,
        started_at: "2026-01-01T00:00:00Z".to_string(),
        finished_at: "2026-01-01T00:00:01Z".to_string(),
        working_directory: "/home/alice/dotfiles".to_string(),
        errors: Vec::new(),
        warnings: Vec::new(),
        info: None,
        results: vec![SubjectResult {
            subject: Subject {
                name: "home".to_string(),
            },
            status: Status::Success,
            generation: None,
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            errors: Vec::new(),
            warnings: Vec::new(),
            result: ResultBody {
                items: vec![
                    Item {
                        id: zshrc.clone(),
                        kind: "entry".to_string(),
                        label: Some(".zshrc".to_string()),
                        status: ItemStatus::Success,
                        error: None,
                        warnings: Vec::new(),
                        info: None,
                    },
                    Item {
                        id: init_lua,
                        kind: "entry".to_string(),
                        label: Some("init.lua".to_string()),
                        status: ItemStatus::Skipped,
                        error: None,
                        warnings: Vec::new(),
                        info: None,
                    },
                ],
                changes: vec![Change {
                    kind: ChangeKind::Modify,
                    item_id: zshrc,
                    reversible: true,
                    info: None,
                }],
                info: None,
            },
        }],
    };

    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}
