# Nagare / 流

[English README](README.md) | [仕様書](docs/spec.md) | [設計書](docs/architecture.md) | [Tutorial](docs/tutorial.md) | [チュートリアル](docs/tutorial_ja.md)

Nagare は、コーディング Agent のための adapter-first な Execution Ledger です。

目的は、Agent 基盤が入れ替わっても、Work Item、Run Packet、Agent Run、Artifact、Evidence、Verification Result、Handoff、Human Decision を local-first な制御レイヤーに残すことです。

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
nagare item run work_0001 --command "echo codex attempt failed && exit /B 1"
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
nagare rule check README.md
nagare item preview work_0001
nagare item review work_0001
nagare handoff dispatch work_0001
nagare item list
nagare item show work_0001
```

npm package は install / distribution の経路に限定し、製品としての操作面は `nagare` コマンドに寄せます。

## ドキュメント言語ポリシー

ユーザー向け README とチュートリアルは、英語版と日本語版をペアで管理します。

- `README.md` / `README_ja.md`
- `docs/tutorial.md` / `docs/tutorial_ja.md`

実装設計の正本は日本語で管理します。

- `docs/architecture.md`
- `docs/spec.md`
