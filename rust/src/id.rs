//! item の id の導出（spec §5）。
//!
//! id = lowercase-hex( sha256( JCS( identity ) ) )
//!
//! identity の値域は spec §5 に従う。文字列 / 整数 ±(2^53−1) / bool / null /
//! 配列 / オブジェクト（メンバー名は ASCII）に限り、域外は id の導出時に拒む。

use std::collections::BTreeMap;
use std::fmt;

use sha2::{Digest, Sha256};

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

impl Key {
    /// JSON 文字列から key を読む。
    ///
    /// 数値は字句で判定する（spec §5）。小数部か指数部を持つ数値と、i64 に
    /// 収まらない整数は `Err` を返す。`-0` は `0` として読む。値の後ろに
    /// 空白以外の文字があれば `Err` を返す。
    pub fn from_json(input: &str) -> Result<Key, IdError> {
        let mut p = Parser {
            src: input.as_bytes(),
            pos: 0,
        };
        p.skip_ws();
        let key = p.value()?;
        p.skip_ws();
        if p.pos != p.src.len() {
            return Err(p.error("unexpected trailing characters"));
        }
        Ok(key)
    }
}

/// 整数の値域の上限 2^53−1（spec §5）。
const MAX_SAFE_INTEGER: i64 = (1 << 53) - 1;

/// identity から item の id を導出する。
///
/// id は `sha256(JCS({"kind": kind, "key": key}))` の 64 文字の小文字 16 進である。
/// key が spec §5 の値域の外にある（整数の絶対値が 2^53−1 を超える、メンバー名が
/// ASCII でない）ときは `Err` を返す。
pub fn derive_id(identity: &Identity) -> Result<String, IdError> {
    // JCS のメンバー順では "key" が "kind" より前に来る。
    let canonical = format!(
        "{{\"key\":{},\"kind\":{}}}",
        canonicalize(&identity.key)?,
        encode_string(&identity.kind)
    );
    let digest = Sha256::digest(canonical.as_bytes());
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// key を JCS（RFC 8785）で正準化し、spec §5 の値域を検査する。
///
/// メンバー名は ASCII に限るので、`BTreeMap` のバイト順は JCS が定める
/// UTF-16 code unit 順と一致する。
fn canonicalize(key: &Key) -> Result<String, IdError> {
    Ok(match key {
        Key::Null => "null".to_string(),
        Key::Bool(b) => b.to_string(),
        Key::Int(n) => {
            if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(n) {
                return Err(IdError {
                    message: format!("integer is out of identity domain ±(2^53-1) (got {n})"),
                });
            }
            n.to_string()
        }
        Key::String(s) => encode_string(s),
        Key::Array(items) => {
            let parts = items
                .iter()
                .map(canonicalize)
                .collect::<Result<Vec<_>, _>>()?;
            format!("[{}]", parts.join(","))
        }
        Key::Object(members) => {
            let mut parts = Vec::with_capacity(members.len());
            for (name, value) in members {
                if !name.is_ascii() {
                    return Err(IdError {
                        message: format!("object member name must be ASCII (got {name:?})"),
                    });
                }
                parts.push(format!("{}:{}", encode_string(name), canonicalize(value)?));
            }
            format!("{{{}}}", parts.join(","))
        }
    })
}

/// JCS の最小のエスケープで文字列を書く（`go/id.go` の `encodeString` と同じ）。
///
/// 短縮形のない 0x20 未満の文字は `\u00xx`（小文字の 16 進）で書き、非 ASCII は
/// エスケープしない。
fn encode_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// RFC 8259 の JSON を読む。数値の字句を保つため手書きする。
///
/// 既定の `serde_json` は `-0` を `-0.0` として読み、域内の `-0` と域外の
/// `-0.0` を区別できない。
struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn error(&self, message: &str) -> IdError {
        IdError {
            message: format!("invalid key JSON at byte {}: {}", self.pos, message),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), IdError> {
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.error(&format!("expected '{}'", byte as char)))
        }
    }

    fn value(&mut self) -> Result<Key, IdError> {
        match self.peek() {
            Some(b'n') => self.literal("null", Key::Null),
            Some(b't') => self.literal("true", Key::Bool(true)),
            Some(b'f') => self.literal("false", Key::Bool(false)),
            Some(b'"') => self.string().map(Key::String),
            Some(b'[') => self.array(),
            Some(b'{') => self.object(),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(self.error("unexpected character")),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn literal(&mut self, word: &str, key: Key) -> Result<Key, IdError> {
        if self.src[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(key)
        } else {
            Err(self.error("invalid literal"))
        }
    }

    fn array(&mut self) -> Result<Key, IdError> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Key::Array(items));
        }
        loop {
            self.skip_ws();
            items.push(self.value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Key::Array(items));
                }
                _ => return Err(self.error("expected ',' or ']'")),
            }
        }
    }

    /// 重複するメンバー名は後の値を採る（Go の `encoding/json` と同じ）。
    fn object(&mut self) -> Result<Key, IdError> {
        self.expect(b'{')?;
        let mut members = BTreeMap::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Key::Object(members));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.error("expected member name"));
            }
            let name = self.string()?;
            self.skip_ws();
            self.expect(b':')?;
            self.skip_ws();
            let value = self.value()?;
            members.insert(name, value);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Key::Object(members));
                }
                _ => return Err(self.error("expected ',' or '}'")),
            }
        }
    }

    /// 数値を字句で判定する。整数表記だけを受け付け、小数部か指数部があれば
    /// 文法として正しくても `Err` を返す（spec §5）。
    fn number(&mut self) -> Result<Key, IdError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => self.digits(),
            _ => return Err(self.error("expected digit")),
        }
        let int_end = self.pos;
        if self.peek() == Some(b'.') {
            self.pos += 1;
            self.require_digits()?;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            self.require_digits()?;
        }
        // 数字と符号だけを読んだので UTF-8 として正しい。
        let lexeme = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or_default();
        if self.pos != int_end {
            return Err(IdError {
                message: format!("number must use integer notation (got {lexeme})"),
            });
        }
        // "-0" は 0 になる（spec §5 の正規化）。
        lexeme.parse::<i64>().map(Key::Int).map_err(|_| IdError {
            message: format!("integer is out of identity domain (got {lexeme})"),
        })
    }

    fn digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
    }

    fn require_digits(&mut self) -> Result<(), IdError> {
        let start = self.pos;
        self.digits();
        if self.pos == start {
            return Err(self.error("expected digit"));
        }
        Ok(())
    }

    fn string(&mut self) -> Result<String, IdError> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let run_start = self.pos;
            while matches!(self.peek(), Some(b) if b != b'"' && b != b'\\' && b >= 0x20) {
                self.pos += 1;
            }
            // 区切りの文字はすべて ASCII なので、切り出した範囲は UTF-8 の文字境界で終わる。
            out.push_str(std::str::from_utf8(&self.src[run_start..self.pos]).unwrap_or_default());
            match self.peek() {
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    out.push(self.escape()?);
                }
                Some(_) => return Err(self.error("control character in string")),
                None => return Err(self.error("unterminated string")),
            }
        }
    }

    fn escape(&mut self) -> Result<char, IdError> {
        let c = match self.peek() {
            Some(b'"') => '"',
            Some(b'\\') => '\\',
            Some(b'/') => '/',
            Some(b'b') => '\x08',
            Some(b'f') => '\x0c',
            Some(b'n') => '\n',
            Some(b'r') => '\r',
            Some(b't') => '\t',
            Some(b'u') => {
                self.pos += 1;
                return self.unicode_escape();
            }
            _ => return Err(self.error("invalid escape")),
        };
        self.pos += 1;
        Ok(c)
    }

    /// 4 桁の 16 進を読む。上位と下位のサロゲートが続くときは 1 文字にまとめ、
    /// 片方だけのサロゲートは `Err` を返す。
    fn unicode_escape(&mut self) -> Result<char, IdError> {
        let high = self.hex4()?;
        let code = match high {
            0xD800..=0xDBFF => {
                if !self.src[self.pos..].starts_with(b"\\u") {
                    return Err(self.error("lone surrogate"));
                }
                self.pos += 2;
                let low = self.hex4()?;
                if !(0xDC00..=0xDFFF).contains(&low) {
                    return Err(self.error("lone surrogate"));
                }
                0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00)
            }
            0xDC00..=0xDFFF => return Err(self.error("lone surrogate")),
            _ => high,
        };
        char::from_u32(code).ok_or_else(|| self.error("invalid code point"))
    }

    fn hex4(&mut self) -> Result<u32, IdError> {
        let mut code = 0;
        for _ in 0..4 {
            let digit = self
                .peek()
                .and_then(|b| (b as char).to_digit(16))
                .ok_or_else(|| self.error("expected hex digit"))?;
            code = code * 16 + digit;
            self.pos += 1;
        }
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> Key {
        Key::String(v.to_string())
    }

    #[test]
    fn from_json_reads_string_escapes() {
        assert_eq!(Key::from_json(r#""\"\\\n\u0041""#), Ok(s("\"\\\nA")));
        assert_eq!(Key::from_json(r#""\/\b\f\r\t""#), Ok(s("/\u{8}\u{c}\r\t")));
    }

    #[test]
    fn from_json_reads_surrogate_pair_as_one_char() {
        assert_eq!(Key::from_json(r#""\ud83d\udd11""#), Ok(s("🔑")));
        assert_eq!(Key::from_json(r#""\uD83D\uDD11""#), Ok(s("🔑")));
    }

    #[test]
    fn from_json_rejects_lone_surrogate() {
        assert!(Key::from_json(r#""\ud83d""#).is_err());
        assert!(Key::from_json(r#""\udd11""#).is_err());
        assert!(Key::from_json(r#""\ud83dx""#).is_err());
    }

    #[test]
    fn from_json_rejects_trailing_characters() {
        assert!(Key::from_json("1 x").is_err());
        assert!(Key::from_json("{} }").is_err());
        assert!(Key::from_json(r#""a"b"#).is_err());
        assert!(Key::from_json("null null").is_err());
        assert_eq!(Key::from_json(" \t\r\n1 \t\r\n"), Ok(Key::Int(1)));
    }

    #[test]
    fn from_json_rejects_raw_control_characters() {
        assert!(Key::from_json("\"a\nb\"").is_err());
    }

    #[test]
    fn from_json_reads_numbers_by_lexeme() {
        assert_eq!(Key::from_json("0"), Ok(Key::Int(0)));
        assert_eq!(Key::from_json("-0"), Ok(Key::Int(0)));
        assert_eq!(Key::from_json("-12"), Ok(Key::Int(-12)));
        for bad in [
            "1.0",
            "1e3",
            "1E3",
            "-0.0",
            "1.5",
            "01",
            "+1",
            "-",
            "1.",
            ".5",
            "99999999999999999999",
        ] {
            assert!(Key::from_json(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn from_json_reads_literals_and_containers() {
        let mut obj = BTreeMap::new();
        obj.insert(
            "b".to_string(),
            Key::Array(vec![Key::Null, Key::Bool(true)]),
        );
        obj.insert("a".to_string(), Key::Bool(false));
        assert_eq!(
            Key::from_json(r#"{ "b" : [ null , true ] , "a" : false }"#),
            Ok(Key::Object(obj))
        );
        assert_eq!(Key::from_json("[]"), Ok(Key::Array(vec![])));
        assert_eq!(Key::from_json("{}"), Ok(Key::Object(BTreeMap::new())));
        for bad in [
            "",
            "[1,]",
            "{\"a\":1,}",
            "{a:1}",
            "tru",
            "nul",
            "[1 2]",
            "\"abc",
        ] {
            assert!(Key::from_json(bad).is_err(), "{bad:?}");
        }
    }

    fn identity(key: Key) -> Identity {
        Identity {
            kind: "n".to_string(),
            key,
        }
    }

    #[test]
    fn derive_id_matches_known_vector() {
        let key = Key::from_json(r#"{"target":".zshrc"}"#).unwrap();
        let id = Identity {
            kind: "entry".to_string(),
            key,
        };
        assert_eq!(
            derive_id(&id),
            Ok("2ba0b24fb64819dfe4bab60dd153fe5a636bdd5a7535d32039c1fe24736f97f8".to_string())
        );
    }

    #[test]
    fn derive_id_checks_integer_range() {
        const MAX: i64 = (1 << 53) - 1;
        assert!(derive_id(&identity(Key::Int(MAX))).is_ok());
        assert!(derive_id(&identity(Key::Int(-MAX))).is_ok());
        assert!(derive_id(&identity(Key::Int(MAX + 1))).is_err());
        assert!(derive_id(&identity(Key::Int(-MAX - 1))).is_err());
        assert!(derive_id(&identity(Key::Array(vec![Key::Int(i64::MIN)]))).is_err());
    }

    #[test]
    fn derive_id_rejects_non_ascii_member_name() {
        let mut obj = BTreeMap::new();
        obj.insert("キー".to_string(), Key::Null);
        let key = Key::Array(vec![Key::Object(obj)]);
        assert!(derive_id(&identity(key)).is_err());
    }

    #[test]
    fn canonicalize_escapes_strings_minimally() {
        let key = Key::String("\"\\\x08\x0c\n\r\t\x1f\x7f/é🔑".to_string());
        let expected = concat!(r#""\"\\\b\f\n\r\t"#, "\\", "u001f\x7f/é🔑\"");
        assert_eq!(canonicalize(&key), Ok(expected.to_string()));
    }

    #[test]
    fn canonicalize_sorts_members_and_keeps_array_order() {
        let key = Key::from_json(r#"{"b":[3,1],"a":{"z":null,"B":true},"A":-0}"#).unwrap();
        assert_eq!(
            canonicalize(&key),
            Ok(r#"{"A":0,"a":{"B":true,"z":null},"b":[3,1]}"#.to_string())
        );
    }
}
