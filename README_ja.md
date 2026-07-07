# Nagare / 流

![Nagare ロゴ](logo.png)

[English README](README.md) | [PRD](docs/nagare_prd_v1_0.md) | [UIプロトタイプ](docs/design-assets/prototype/) | [トレーススキーマ (NF-2)](docs/nagare_trace_schema_v1_0.md) | [過去文書](docs/archive/)

Nagare は、コーディング Agent のための adapter-first な Execution Ledger です。

目的は、Agent 基盤が入れ替わっても、Work Item、Run Packet、Agent Run、Artifact、Evidence、Verification Result、Handoff、Human Decision を local-first な制御レイヤーに残すことです。

## エージェントスキルとして使う

このリポジトリは、Claude Code や Codex CLI などのコーディングエージェントに
`nagare` 台帳の使い方を教えるスキル(`skills/nagare/SKILL.md`)を同梱しています。

**このスキルの目的** — エージェントの作業を「監査可能」にすることです。
すべてのタスクについて、誰が・何を入力に・何を作り・どうレビューされ・
人間が何を承認したかが、構造化されたローカル記録として残ります。
Nagare が記録するのは判断であり、通信ではありません(生のAPIやりとりは保存しません)。

**インストール**

```bash
# 1. スキルをエージェント環境に追加
npx skills add hachiware-labs/nagare

# 2. スキルが操作する CLI を導入
npm install -g @hachiware-labs/nagare
nagare doctor
```

**使い方** — インストール後、エージェントに次のように頼みます。

- 「このリファクタリングを Nagare で記録しながら進めて、完了前にレビューを通して」
- 「この作業を Work Item にして実行し、失敗したら別のエージェントに引き継いで」
- 「work_0001 の台帳の状態を見せて」

エージェントは台帳の初期化(`nagare init`)、Agent Profile の登録、Work Item の
作成、実行と失敗の Evidence 記録、エージェント間 Handoff、レビュー実行までを行い、
`done` にする前には必ずあなたの明示的な承認を求めて停止します。

**どういうことに使うとよいか**

- **証跡が必要な作業**: 納品物、リリース作業など、後から「なぜこうなったか」を
  説明する必要があるタスク。
- **複数エージェントの併用**: Claude Code / Codex などを切り替えながら、
  失敗の引き継ぎ文脈も含めて1本の履歴を保ちたいとき。
- **レビューゲート付きの作業**: 結果を黙って受け入れず、レビューと人間の
  明示的な承認を通してから完了にしたいとき。

## 現在のスライス

このリポジトリでは、最初の end-to-end ユーザーシナリオまで実行できます。

- ローカル Nagare ledger を初期化する
- `.nagare/agents/*.toml` に project-local な Agent Profile を登録する
- Work Item を作成する
- `codex-cli` agent profile の失敗実行を記録する
- 失敗を Evidence として保存する
- `codex-app-server` への Handoff を作成する
- 成功する再実行を記録する
- Verification を通す
- Human Decision として approve する
- `done` に到達する

## ローカル開発

```powershell
npm test
npm run build
nagare doctor
nagare init
```

`npm run build` は release CLI binary をビルドし、local npm package に
stage してから workspace package を global link します。これにより
`nagare` は現在の開発版を実行します。

## 最初のシナリオ

通常のユーザー向けコマンド列としてシナリオを実行します。

```powershell
$env:NAGARE_ROOT = "$env:TEMP\nagare-first"
nagare init
nagare locale use --language ja-JP --timezone Asia/Tokyo
nagare agent add --id codex-impl-smoke --display-name "Codex CLI Smoke Implementer" --runtime codex-local --adapter process.codex-cli --role implementer --working-dir . --description "実装と検証向け" --specialties implementation,verification
nagare agent add --id codex-app-smoke --display-name "Codex App Server Smoke Implementer" --runtime codex-app-local --adapter stdio.codex-app-server --role implementer --working-dir . --description "計画とレビュー向け" --specialties planning,review
nagare agent list
nagare agent use --work-agent codex-impl-smoke --review-agent codex-app-smoke --dispatch-agent codex-impl-smoke
nagare agent defaults
nagare agent doctor codex-impl-smoke
nagare agent probe codex-impl-smoke
nagare item create --title "Repair failing agent run" --description "Demonstrate cross-agent evidence and handoff."
nagare item preview work_0001 --command "echo dispatch preview && exit /B 0"
nagare item dispatch accept work_0001
nagare item run work_0001 --command "echo codex run failed && exit /B 1"
nagare handoff create work_0001 --from-agent codex-impl-smoke --to-agent codex-app-smoke --reason "Codex agent profile produced a failing run" --summary "Retry with Codex App Server agent profile using the captured run log as evidence."
nagare item run work_0001 --agent codex-app-smoke --command "echo codex app server retry fixed the task && exit /B 0"
nagare verify work_0001 --command "echo verification passed && exit /B 0"
nagare decision approve work_0001 --rationale "Required verification passed after cross-agent handoff."
nagare item show work_0001
Remove-Item Env:\NAGARE_ROOT
```

snapshot header が以下になれば成功です。

```text
work_0001	done	Repair failing agent run
```

このシナリオは登録した agent profile ID を使いますが、実行コマンドはローカルの demo command です。これにより、最初の workflow を決定的に保ちながら、adapter-first な製品形状を確認できます。未知の agent profile ID は拒否されます。

## `nagare` コマンド

インストール後のユーザー向け操作はすべて `nagare` コマンドから実行できます。

```powershell
nagare doctor
nagare init
nagare locale show
nagare agent list
nagare agent show codex-cli
nagare agent defaults
nagare agent doctor codex-cli
nagare agent probe codex-cli
nagare item preview work_0001
nagare item dispatch accept work_0001
nagare item review work_0001
nagare handoff dispatch work_0001
nagare item list
nagare item show work_0001
```

npm package は install / distribution の経路に限定し、製品としての操作面は `nagare` コマンドに寄せます。

## ドキュメント言語ポリシー

ユーザー向け README は、英語版と日本語版をペアで管理します。

- `README.md` / `README_ja.md`

プロダクト設計の正本は日本語で管理します。

- `docs/nagare_prd_v1_0.md`(PRD・機能一覧)
- `docs/nagare_trace_schema_v1_0.md`(NF-2 トレーススキーマ)
- `docs/design-assets/prototype/`(UIの正)

再設計以前の文書(spec、architecture、チュートリアル、旧wireframe)は
`docs/archive/` に保管しています。
