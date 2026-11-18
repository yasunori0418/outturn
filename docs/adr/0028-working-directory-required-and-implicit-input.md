# ADR-0028: workingDirectory を必須にし、standalone で暗黙の入力を記録付きで許容する

- ステータス: 採用
- 日付: 2026-11-18
- 関連: `spec/v1/spec.md`（§2, §3, §8）, `schema/v1/envelope.schema.json`, ADR-0007, ADR-0010, ADR-0013, ADR-0015, ADR-0021, ADR-0022
- 改訂対象: ADR-0014（入力の形を領分外とする決定は不変。spec §8 の入力の受け取り方の規律を「stdin の JSON か明示引数のみ」から、明示経路の必須と暗黙の入力の記録へ改める）

## 背景

spec §8 の standalone は「入力は stdin の JSON か明示引数のみとし、状態・設定の暗黙探索をしない」を MUST としていた。一方、適合済みの layat は `-f` を省くとエントリポイントを作業ディレクトリから探し、環境変数 `LAYAT_GENERATOR` と `$XDG_CONFIG_HOME/layat/config.toml` も読む。規範と適合ツールの動きが食い違っている。これから適合させる mitodoke も、作業ディレクトリの設定ファイルを読む作りを検討している。

エンベロープには実行時の作業ディレクトリも、ツールが自分で解決した入力も残らない。両方がエンベロープに残れば、「何を入力に何が起きたか」をエンベロープだけで追える。実装例が layat しか無いため、作業ディレクトリは必須のフィールドにする方針とした（issue #66）。

## 決定

- エンベロープ直下に必須の規格フィールド `workingDirectory` を追加する。optional にせず、`info` の推奨語彙にもしない。
- `workingDirectory` の値は実行時のプロセスの作業ディレクトリの観測記録で、絶対パスとする。結果への影響の有無によらず常に出力する。入力の解決の起点を切り替えるツール（`-C dir` の類）は切り替え後の値とする。シンボリックリンクの解決を含む正規化は求めない。取得できない場合の特例は設けない。
- `workingDirectory` にマスクの規定は設けない。spec §3（機微情報の扱い）は改訂しない。
- schema の `workingDirectory` は `type: string` と `minLength: 1` だけとし、`pattern` で絶対パスを強制しない。絶対パスであることは spec 本文の MUST で定める。
- 暗黙の入力とは明示引数か stdin で与えられる入力を、与えられなかったときにツールが自分で解決したものを指す。作業ディレクトリ起点の探索・環境変数・既定パスの設定ファイルのどの経路でも同じ扱いとする。用語の正名は「暗黙の入力」とし、「自動探索」「暗黙探索」は避ける語として `CONTEXT.md` に記す。
- spec §8 の standalone は、入力を stdin の JSON か明示引数だけで全て与えて実行できる経路を MUST とし、暗黙の入力を使った実行では解決結果の `info` への記録を MUST とする。
- 解決した入力は規格フィールドにしない。トップレベル `info`（主体ごとに解決結果が違う場合は `result.info`）に記録させ、キーと形はツールに任せる。記録するのは解決結果（ファイルは絶対パス、設定値はその値）で、環境変数の名前の記録は求めない。`info` に記録する解決結果には spec §3 を従来どおり適用する。
- 暗黙の入力の記録義務は spec §8 (2) のリント検査の対象外とする。`go/conformance` にリントを追加しない。
- specVersion は 1 のまま据え置く。必須フィールドの追加は規約上 specVersion を上げる変更に当たるが、ADR-0013 と同じく未リリース下書き段階の非互換修正として扱い、規約の文面とのズレを許容する。spec §2 の互換規則の文と `AGENTS.md` は改訂しない。
- ADR は本 ADR の 1 本にまとめる。改訂注記を追加する旧 ADR は ADR-0014 のみとする。
- 適合テストデータは既存 31 件に `workingDirectory` を追記し、新規は `valid/implicit-input.json`・`invalid/working-directory-missing.json`・`invalid/working-directory-empty.json` の 3 件とする。`valid/implicit-input.json` の `tool.name` は `layat` とする。
- Go の `Envelope` には `WorkingDirectory string` を `omitempty` なしで `FinishedAt` の直後に追加する。型パラメータは増やさない。
- 本改定で書く日本語の文章（成果物・コミットメッセージ・PR 本文）は yomiyasu スキルで確認する。英語の `README.md` は対象外とする。MUST・SHOULD・MAY と本 ADR で決めた用語は、この確認でも変えない。

## 根拠

- 作業ディレクトリは暗黙の入力を解決する起点になる。解決結果の `info` への記録と合わせれば、同じ実行を追うのに必要な情報がエンベロープにそろう。
- 値を「作業ディレクトリが結果に影響したことの宣言」ではなく観測記録とすれば、ツールは影響の有無を判定しなくてよい。`generation`（→ ADR-0015）と同じ観測型にそろう。
- 必須にすれば、消費側は `workingDirectory` の欠落を想定しなくてよい。実装例が layat だけの今なら、必須化で直す実装も少ない。
- 明示経路を MUST に残すので、「単体で・JSON を手で与えて試せる」性質（`docs/concept.md` 原則 2）は保たれる。そのうえで、layat のように作業ディレクトリや設定ファイルから入力を解決する動きを、記録を条件に規範と一致させられる。
- 解決の対象とその形（エントリポイントのパス、generator の名前、設定値など）はツールごとに違う。`info` に置けば、規格型は `additionalProperties: false` で閉じたまま記録を残せる（→ ADR-0007）。
- どの入力を暗黙に解決したかはツールしか知らず、エンベロープ単体のリント検査では記録の要否を判定できない。

## 影響

- `spec/v1/spec.md`: §2 の jsonc 例と規範文に `workingDirectory` を追加し、§8 の standalone を改定する。§2 の互換規則の文と §3 は無改訂。
- `schema/v1/envelope.schema.json`・`go/types.go`・`go/internal/spec/`・`testdata/v1/`: 後続の変更（issue #68）で本 ADR に追従させる。spec の改定だけがマージされた状態では、spec が必須とする `workingDirectory` を schema が拒否するため、両者は続けてマージする。
- `CONTEXT.md`・`README.md`・`README.ja.md`・`docs/concept.md`・`docs/ecosystem/distro-plan.md`・`docs/guides/conformance.md`: 記述を同期する。
- `docs/adr/0014-scope-result-envelope-only.md`: 改訂注記を追記する。
- id 導出系（`testdata/v1/id-vectors.json`・`go/id.go`・`nix/lib.nix`）と `go/conformance` のリント検査は無改訂。`workingDirectory` は id 導出に関与しない。
- 適合ツール（layat・mitodoke）の対応は各ツールのリポジトリで扱う。

## 棄却した代替案

- **`info` の推奨語彙にとどめる**: キー名と出力の有無がツールごとにばらつき、消費側はツールを知らないと作業ディレクトリを読めない。全ツールに共通の観測値なので規格フィールドとする。
- **optional にする**: 出力しないツールが残り、消費側は欠落を常に想定しなければならない。
- **解決した入力も規格フィールドにする**: 解決の対象と形はツールごとに違い、規格型にツール固有の語彙を持ち込むことになる（→ ADR-0007, ADR-0014）。
- **schema の pattern で絶対パスを強制する**: 絶対パスの書き方は環境によって違い、正規表現を 1 つに決めると特定の環境を前提にしかねない。絶対パスであることは spec 本文の MUST で定めれば足りる。
- **specVersion を 2 に上げる**: 規約の文面上は必須化なので v2 に当たる。しかしリリースタグは 1 本も無く（→ ADR-0026）、v1 は未リリースの下書き段階にある。ADR-0013 と同じく、運用前の下書きに `spec/v2/` を新設するのは過剰で、v1 内の修正にとどめる。
