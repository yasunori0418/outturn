//! `testdata/v1/id-vectors.json` の全ベクタで id の導出を検証する。
//!
//! key は `RawValue` で生の JSON 文字列のまま取り出し、`Key::from_json` に渡す。
//! 数値の字句（`1.0`・`1e3`・`-0`）を保つためである。

use outturn::{derive_id, Identity, Key};
use serde::Deserialize;
use serde_json::value::RawValue;

#[derive(Deserialize)]
struct Vectors {
    vectors: Vec<Accepted>,
    rejected: Vec<Rejected>,
}

#[derive(Deserialize)]
struct RawIdentity {
    kind: String,
    key: Box<RawValue>,
}

#[derive(Deserialize)]
struct Accepted {
    identity: RawIdentity,
    expected: String,
}

#[derive(Deserialize)]
struct Rejected {
    identity: RawIdentity,
    reason: String,
}

fn load() -> Vectors {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../testdata/v1/id-vectors.json"
    );
    let text = std::fs::read_to_string(path).expect("read id-vectors.json");
    serde_json::from_str(&text).expect("parse id-vectors.json")
}

#[test]
fn accepted_vectors_derive_expected_id() {
    let vectors = load().vectors;
    assert!(!vectors.is_empty(), "vectors is empty");
    for v in vectors {
        let raw = v.identity.key.get();
        let key = Key::from_json(raw).unwrap_or_else(|e| panic!("from_json({raw}): {e}"));
        let identity = Identity {
            kind: v.identity.kind,
            key,
        };
        let id = derive_id(&identity).unwrap_or_else(|e| panic!("derive_id({raw}): {e}"));
        assert_eq!(id, v.expected, "key {raw}");
    }
}

#[test]
fn rejected_vectors_are_rejected() {
    let rejected = load().rejected;
    assert!(!rejected.is_empty(), "rejected is empty");
    for v in rejected {
        let raw = v.identity.key.get();
        let result = Key::from_json(raw).and_then(|key| {
            derive_id(&Identity {
                kind: v.identity.kind,
                key,
            })
        });
        assert!(result.is_err(), "key {raw} ({}) was accepted", v.reason);
    }
}
