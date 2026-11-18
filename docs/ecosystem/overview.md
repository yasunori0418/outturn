# エコシステム: オーバービュー

単一責務のツール群と outturn 規格からなるエコシステムの中央索引。北極星は、NixOS とは異なる「Nix 版 Arch / Gentoo」——最小のコアとユーザーによる組み立てを旨とし、優れた Handbook を持つディストリビューション。

NixOS がモジュールシステムという単一の大きな抽象で全体を統合するのに対し、このエコシステムは**単一責務のツールを outturn 規格のパイプで合成する**ことで同じ問題を解く。

## 全体構成

```
build(Nix eval/build)
  → layat apply  # config / env の配置
  → ユーザー / グループ照合ツール  # ユーザー/グループ差分調停
  → setuid ラッパー生成ツール  # setuid ラッパー生成
  → サービス差分適用ツール  # サービス差分 restart/reload
  → ブート世代マネージャ  # ブート世代エントリ同期
  ↑ オーケストレータ  # 上記を固定順で呼び、失敗時は逆順 rollback を指揮
```

## ツール一覧

| ツール | 責務 | 状態 | リポジトリ |
|--------|------|------|-----------|
| **layat** | store パスの任意パスへの配置（symlink / copy）。世代管理・rollback | **active** | [yasunori0418/layat](https://github.com/yasunori0418/layat) |
| **ブート世代マネージャ** | kernel / initrd の ESP 配置とブートエントリの世代同期 | planned | — |
| **setuid ラッパー生成ツール** | setuid / capabilities ラッパーの生成（store の権限モデル上 symlink では原理的に不可能な領分） | planned | — |
| **サービス差分適用ツール** | サービス差分の restart / reload / skip 適用。init 非依存の責務（初期実装は systemd バックエンド） | planned | — |
| **ユーザー / グループ照合ツール** | 宣言された users / groups と /etc/passwd・/etc/shadow の差分調停 | planned | — |
| **オーケストレータ** | ツール群の固定順パイプライン実行と失敗時の逆順 rollback 指揮 | planned | — |
| **outturn** | 上記全ツールが会話する JSON 規格（本リポジトリ） | **draft** | 本リポジトリ |

outturn の **draft → stable** は **2 ツール適合**（layat + 次ツール、M2 のブート世代マネージャを想定）で宣言する。宣言時に draft 表記を落とす。リリースは同一コミットに `v1.N.P`（規格スナップショット）と `go/v1.N.P`（Go module 用ミラー）を対で打ち、互換追加ごとに `N`、規格に影響しない修正で `P` を増やす（→ outturn ADR-0026）。

## 設計原則（要約）

1. **単一責務**: 各ツールは 1 つのことをする。layat のコンセプトに合わない責務は layat に入れず、独立したツールとして分離する
2. **standalone**: どのツールも単体で・どのディストロ上でも・JSON を手で与えて試せる（outturn spec §8 の適合条件）
3. **規格が契約**: ツールの実行結果は outturn 規格（結果エンベロープ）のみに依存して消費できる。入力 JSON の形は各ツールが公開・管理し、ツール固有の知識は `info` に隔離される（→ outturn ADR-0014）
4. **オーケストレータは愚直に**: 固定順パイプライン + 失敗時逆順のみ。依存グラフエンジンにしない

## ドキュメント

| 文書 | 内容 |
|------|------|
| [distro-plan.md](distro-plan.md) | ディストリビューション構想の全体プラン（各ツールの詳細責務・standalone 契約・マイルストーン） |
| [../concept.md](../concept.md) | outturn 規格のコンセプト |
| [../../spec/v1/spec.md](../../spec/v1/spec.md) | outturn 規格の規範仕様 |

layat 固有の計画（mkEnv・plan / verify 等の機能拡張）は [layat リポジトリ](https://github.com/yasunori0418/layat)の issue / ADR が正。本リポジトリはエコシステム横断の文書のみを管理する。

## 未確定事項

- **傘 / ディストロ名**: 未定。北極星に近づいてから確定する
- **計画中ツールの名前**: 未定。各ツールの着手時に決める。条件は、日本語由来の語を避ける（造語は可）、UNIX 哲学に沿った短いコマンド名にする、GitHub の name 完全一致・npm・crates.io・PyPI・Web（Reddit / HN を含む）で既存プロダクトと被らないことを調査する。「束ねる」役割の名前は傘名と関係しうるので、傘名を先に決めるかどうかを先に判断する
- 各 planned ツールの着手順序は distro-plan.md のマイルストーン（M0〜M4）を参照
