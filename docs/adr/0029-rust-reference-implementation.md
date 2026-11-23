# ADR-0029: Rust 参照実装の crate を rust/ に置く

- ステータス: 採用
- 日付: 2026-11-23
- 関連: `spec/v1/spec.md`(§2・§5), `testdata/v1/id-vectors.json`, `rust/`, ADR-0004, ADR-0010, ADR-0023, ADR-0024, ADR-0026, ADR-0027

## 背景

Rust で書くツール(mitodoke の CLI など)が、outturn のエンベロープを出力したい。これまでの参照実装は Go(`go/`)と Nix(`nix/lib.nix`)だけで、Rust には無かった。

item の id は spec §5 の手順で導出し、どのツールが計算しても同じ値にならなければならない。ツールごとに id の導出を書くと、JCS の正準化や値域の検査が実装ごとにずれるおそれがある。そこで、Rust で書くツールが共有する crate をこのリポジトリに置く。

## 決定

- crate は `rust/` に置き、名前は `outturn` とする。持つものはエンベロープの型と item の id の導出の 2 つに限る。
- 公開する型と関数は次のものに限る。id の導出は `Key`・`Key::from_json`・`Identity`・`derive_id`・`IdError`、エンベロープは `Envelope`・`SubjectResult`・`ResultBody`・`Item`・`Change`・`Tool`・`Subject`・`Error`・`Warning`・`Generation`・`Status`・`ItemStatus`・`ChangeKind`。結果本体の型名を `ResultBody` にするのは、`std::result::Result` を隠さないためである。
- identity の key は独自の `Key` 型(`Null` / `Bool` / `Int(i64)` / `String` / `Array` / `Object` の enum)で表す。浮動小数は型で表せない。
- JSON 文字列から key を読む `Key::from_json` は数値の字句を見る手書きのパーサにする。
- JCS(RFC 8785)の正準化は手書きする(Go と同じ方針)。`Key::Int` の値域 ±(2^53−1) とメンバー名の ASCII 制約は `derive_id` で検査する。`-0` は `0` に正規化する(→ ADR-0027)。
- エンベロープの型は型引数を 4 つ取る `Envelope<TItem, TChange, TInfo, TEnvInfo>` とし、既定はすべて `serde_json::Value` とする。info は `Option<T>` で持つ。
- 必須の配列(`results`・`items`)は空でも必ず出力する。任意の配列(`errors`・`warnings`・`changes`)は空のとき、`Option` は `None` のとき出力しない。
- 読み込みでは規格型に無い未知のフィールドを無視する。これは spec §2 の must-ignore(→ ADR-0010)に従うもので、`deny_unknown_fields` は付けない。出力の厳密な検証は flake の `lib.mkSchemaCheck` が担う。
- 依存は `serde`(derive)・`serde_json`・`sha2` と、dev 依存の `serde_json` の `raw_value` に限る。
- crate は crates.io に公開しない(`publish = false`)。利用側は git のコミットを指定して依存する。
- flake check には nixpkgs の `rustPlatform.buildRustPackage` と `cargoLock.lockFile` で載せ(`checks.rust`)、flake の入力は追加しない。`cargo test` で id-vectors の全ベクタの通過を検証する。
- 往復テストでは `testdata/v1/valid/` のサンプルを読んで書き戻した JSON が、`serde_json::Value` として元と等しいことを確かめる。

## 根拠

- 既定の `serde_json` は `-0` を `-0.0` として読むため(実測済み)、`-0`(域内)と `-0.0`(域外)を区別できない。spec §5 は数値を値でなく字句で判定する(→ ADR-0024)ので、字句を見るパーサが要る。
- 浮動小数を持たない `Key` 型にすると、値域外の数値を型の段階で表せなくなる。整数の範囲とメンバー名の ASCII 制約は型で表しにくいので、`derive_id` で検査する。
- 未知のフィールドを無視すると、規格に互換のフィールドが追加されても、古いバージョンの crate で新しい出力を読める。ADR-0010 が specVersion を上げずにフィールドを追加できるとしているのは、消費側が must-ignore に従うことが前提である。
- 型引数の数と並びは Go の `Envelope[TItem, TChange, TInfo, TEnvInfo]` に揃えた。既定を `serde_json::Value` にしたのは、ツールを知らない消費側が型引数を書かずに規格部分だけを型で扱えるようにするためである(Go は既定の型引数を持たず、`json.RawMessage` を明示する)。
- Rust のテストで id-vectors の全ベクタを検証することで、Go・Nix と同じ id を導出することを CI で示せる(→ ADR-0004)。

## 影響

- `rust/`: crate(`src/id.rs`・`src/envelope.rs`)、id-vectors と往復のテスト、組み立ての例(`examples/conformance.rs`)を追加。
- `flake.nix` と `nix/pkgs/outturn-rust.nix`: `checks.rust` を追加。
- `dev/flake.nix`: 開発シェルに `cargo`・`rustc`・`rust-analyzer`・`clippy`・`rustfmt` を追加。
- `.gitignore`: `rust/target/` を追加。
- `.github/actions/changes/action.yaml`: Rust の変更で flake check を実行するため、パスフィルタに `**.rs`・`rust/Cargo.toml`・`rust/Cargo.lock` を追加。
- `docs/guides/conformance.md`: cargo 経路の節を追加。
- `AGENTS.md`: 検証コマンドに `cd rust && cargo test` を追加。値域や JCS 実装を変えるときに同時に更新する対象に Rust を追加。
- `README.md`・`README.ja.md`・`docs/design.md`: 構成図に `rust/` を追加。
- spec・schema・testdata は変えない。Go と Nix の実装も変えない。

## 棄却した代替案

- **`serde_json` の `arbitrary_precision` で数値の字句を保つ**: cargo は依存グラフの中で feature を統一するため、crate がこの feature を有効にすると、同じ `serde_json` を使う利用側の数値の扱いまで変わる。参照実装の都合で利用側の挙動を変えるべきではない。
- **既存の JCS の crate を使う**: 正準化は Go と同じく手書きにし、id-vectors で 3 つの実装の一致を確かめる方針を取る。依存も `serde`・`serde_json`・`sha2` に限り、正準化の正しさを外部の crate に預けない。
- **未知のフィールドを拒む(`deny_unknown_fields`)**: spec §2 と ADR-0010 の must-ignore に反する。規格にフィールドが追加されると、古いバージョンの crate で新しい出力が読めなくなる。厳密な検証は `lib.mkSchemaCheck` で行える。
- **crates.io に公開する**: 公開するなら、Rust 用のタグ(`rust/v1.N.P` など)の運用を決める必要があり、ADR-0026 のタグ運用を改める変更になる。今回の範囲を越えるので、利用側は git のコミットで依存する。必要になったら別の ADR で扱う。
