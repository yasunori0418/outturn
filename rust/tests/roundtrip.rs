//! エンベロープの型の読み書きを検証する。
//!
//! JSON の比較は `serde_json::Value` として等しいかで判定する。

use outturn::{Envelope, ResultBody, Status, Tool};
use serde_json::Value;

#[test]
fn valid_samples_roundtrip() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../testdata/v1/valid");
    let mut count = 0;
    for entry in std::fs::read_dir(dir).expect("read testdata/v1/valid") {
        let path = entry.expect("read dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("read sample");
        let original: Value = serde_json::from_str(&text).expect("parse sample as Value");
        let envelope: Envelope = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let written = serde_json::to_value(&envelope).expect("write envelope");
        assert_eq!(written, original, "{}", path.display());
        count += 1;
    }
    assert!(count > 0, "no samples in {dir}");
}

#[test]
fn empty_required_fields_are_written() {
    let envelope: Envelope = Envelope {
        spec_version: 1,
        tool: Tool {
            name: "example".to_string(),
            version: "0.1.0".to_string(),
        },
        command: "check".to_string(),
        status: Status::Success,
        dry_run: false,
        started_at: "2026-01-01T00:00:00Z".to_string(),
        finished_at: "2026-01-01T00:00:01Z".to_string(),
        working_directory: "/tmp".to_string(),
        errors: Vec::new(),
        warnings: Vec::new(),
        info: None,
        results: Vec::new(),
    };
    let text = serde_json::to_string(&envelope).expect("write envelope");
    assert!(text.contains(r#""results":[]"#), "{text}");
    assert!(text.contains(r#""dryRun":false"#), "{text}");

    let body: ResultBody = ResultBody {
        items: Vec::new(),
        changes: Vec::new(),
        info: None,
    };
    let text = serde_json::to_string(&body).expect("write result body");
    assert!(text.contains(r#""items":[]"#), "{text}");
}
