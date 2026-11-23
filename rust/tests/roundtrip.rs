//! エンベロープの型の読み書きを検証する。
//!
//! JSON の比較は `serde_json::Value` として等しいかで判定する。

use outturn::{
    Change, Envelope, Error, Generation, Item, ResultBody, Status, Subject, SubjectResult, Tool,
    Warning,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

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
        let envelope: Envelope =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
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

/// `minimal` に未知のフィールドを加えて `T` へ読み、書き戻した JSON が `minimal` と等しいことを確かめる。
fn assert_ignores_unknown<T: DeserializeOwned + Serialize>(name: &str, minimal: Value) {
    let mut input = minimal.clone();
    input
        .as_object_mut()
        .expect("minimal is an object")
        .insert("unknownField".to_string(), json!(true));
    let read: T = serde_json::from_value(input).unwrap_or_else(|e| panic!("{name}: {e}"));
    let written = serde_json::to_value(&read).expect("write value");
    assert_eq!(written, minimal, "{name}");
}

#[test]
fn unknown_fields_are_ignored() {
    let id = "0".repeat(64);
    let tool = json!({ "name": "example", "version": "0.1.0" });
    let subject = json!({ "name": "home" });
    let error = json!({ "code": "E_EXAMPLE", "message": "failed" });
    let warning = json!({ "code": "W_EXAMPLE", "message": "warned" });
    let generation = json!({ "profile": "/nix/var/nix/profiles/per-user/alice/home" });
    let change = json!({ "kind": "add", "itemId": &id, "reversible": true });
    let item = json!({ "id": &id, "kind": "entry", "status": "success" });
    let body = json!({ "items": [item.clone()] });
    let subject_result = json!({
        "subject": subject.clone(),
        "status": "success",
        "startedAt": "2026-01-01T00:00:00Z",
        "finishedAt": "2026-01-01T00:00:01Z",
        "result": body.clone(),
    });
    let envelope = json!({
        "specVersion": 1,
        "tool": tool.clone(),
        "command": "check",
        "status": "success",
        "dryRun": false,
        "startedAt": "2026-01-01T00:00:00Z",
        "finishedAt": "2026-01-01T00:00:01Z",
        "workingDirectory": "/tmp",
        "results": [subject_result.clone()],
    });

    assert_ignores_unknown::<Envelope>("Envelope", envelope);
    assert_ignores_unknown::<SubjectResult>("SubjectResult", subject_result);
    assert_ignores_unknown::<ResultBody>("ResultBody", body);
    assert_ignores_unknown::<Item>("Item", item);
    assert_ignores_unknown::<Change>("Change", change);
    assert_ignores_unknown::<Tool>("Tool", tool);
    assert_ignores_unknown::<Subject>("Subject", subject);
    assert_ignores_unknown::<Error>("Error", error);
    assert_ignores_unknown::<Warning>("Warning", warning);
    assert_ignores_unknown::<Generation>("Generation", generation);
}
