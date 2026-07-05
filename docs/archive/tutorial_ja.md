# Nagare チュートリアル

[English](tutorial.md)

このチュートリアルでは、最初に完成したユーザーシナリオを扱います。Codex CLI agent profile の失敗、Evidence 保存、Codex App Server agent profile への Handoff、成功する再実行、Review、Human approval、最終的な `done` までを確認します。

## 前提

- `nagare` コマンドが install 済みで、`PATH` から実行できること
- Codex CLI

## 1. CLI を確認する

```powershell
nagare version
```

## 2. Nagare project を初期化する

```powershell
nagare init
```

これにより、以下が作られます。

```text
.nagare/
  project.toml
  agents/
  artifacts/
  logs/
  state/
```

## 3. 環境を確認する

```powershell
nagare doctor
nagare locale show
```

このコマンドは、現在の project root、`.nagare/project.toml` の有無、主要なローカルツールが利用可能かを表示します。

## 4. Agent Profile を登録して Work Item を作成する

まず project-local な Agent Profile を登録します。

```powershell
nagare locale use --language ja-JP --timezone Asia/Tokyo
nagare agent add --id codex-impl-smoke --display-name "Codex CLI Smoke Implementer" --runtime codex-local --adapter process.codex-cli --working-dir . --description "実装とレビュー向け" --specialties implementation,review
nagare agent add --id codex-app-smoke --display-name "Codex App Server Smoke Implementer" --runtime codex-app-local --adapter stdio.codex-app-server --working-dir . --description "計画とレビュー向け" --specialties planning,review
nagare agent list
nagare agent use --work-agent codex-impl-smoke --review-agent codex-app-smoke --dispatch-agent codex-impl-smoke
nagare agent defaults
nagare agent doctor codex-impl-smoke
nagare agent probe codex-impl-smoke
```

```powershell
nagare item create --title "Repair failing agent run" --description "Demonstrate cross-agent evidence and handoff."
```

以下が表示されます。

```text
created work_0001 ready
```

## 5. 最初の Agent 実行を記録する

```powershell
nagare item preview work_0001 --command "echo dispatch preview && exit /B 0"
nagare item dispatch accept work_0001
```

Preview は DispatchPlan を `draft` として保存し、accept すると `item run` の既定実行先として使えるようになります。

```powershell
nagare item run work_0001 --command "echo codex run failed && exit /B 1"
```

失敗した run と evidence が記録されます。

```text
run run_0002 failed agent=codex-impl-smoke
```

## 6. Handoff を作成する

```powershell
nagare handoff create work_0001 --from-agent codex-impl-smoke --to-agent codex-app-smoke --reason "Codex agent profile produced a failing run" --summary "Retry with Codex App Server agent profile using the captured run log as evidence."
```

## 7. 別 agent profile で再実行する

```powershell
nagare item run work_0001 --agent codex-app-smoke --command "echo codex app server retry fixed the task && exit /B 0"
```

## 8. 検証する

```powershell
nagare item review work_0001 --command "type review-pass.md"
```

## 9. 承認する

```powershell
nagare decision approve work_0001 --rationale "Required review passed after cross-agent handoff."
```

作成された Work Item を確認します。

```powershell
nagare item show work_0001
```

snapshot には以下が含まれます。

```text
runs:
  run_0002  codex-impl-smoke     failed
  run_0006  codex-app-smoke  succeeded
evidence:
  Agent run failed with profile `codex-impl-smoke`
  Agent run succeeded with profile `codex-app-smoke`
  Review passed
handoffs:
  codex-impl-smoke -> codex-app-smoke
decisions:
  approve
```

## 10. 一時 root で実行する

```powershell
$tmp = Join-Path $env:TEMP "nagare-first"
$env:NAGARE_ROOT = $tmp
nagare init
nagare locale use --language ja-JP --timezone Asia/Tokyo
nagare agent add --id codex-impl-smoke --runtime codex-local --adapter process.codex-cli --working-dir . --description "実装とレビュー向け" --specialties implementation,review
nagare agent add --id codex-app-smoke --runtime codex-app-local --adapter stdio.codex-app-server --working-dir . --description "計画とレビュー向け" --specialties planning,review
nagare agent use --work-agent codex-impl-smoke --review-agent codex-app-smoke --dispatch-agent codex-impl-smoke
nagare agent probe codex-impl-smoke
nagare item create --title "Repair failing agent run"
nagare item preview work_0001 --command "echo dispatch preview && exit /B 0"
nagare item dispatch accept work_0001
nagare item run work_0001 --command "echo codex run failed && exit /B 1"
nagare handoff create work_0001 --from-agent codex-impl-smoke --to-agent codex-app-smoke --reason "Codex agent profile produced a failing run"
nagare item run work_0001 --agent codex-app-smoke --command "echo codex app server retry fixed the task && exit /B 0"
nagare item review work_0001 --command "type review-pass.md"
nagare decision approve work_0001
nagare item show work_0001
Remove-Item Env:\NAGARE_ROOT
```

## 実際の Codex App Server 実行

`stdio.codex-app-server` adapter の Agent Profile に対して `--prompt` を使うと、
Nagare は `codex app-server --listen stdio://` を起動し、thread を作成して turn を開始します。
app-server の transcript は AgentRun ExecutionRecord として保存されます。
