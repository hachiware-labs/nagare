# delta-request

## Delta ID
- DR-20260526-auto-recovery-advance

## Delta Type
- FEATURE

## 目的
- `item advance --until-blocked` が recovery plan 作成で止まりすぎる問題を減らし、明示オプション付きで recovery accept/apply まで自動継続できるようにする。
- 人間承認や入力待ちは引き続き停止点として残す。

## 変更対象（In Scope）
- Workflow Advance:
  - draft RecoveryPlan がある場合の次アクションを明示する。
  - accepted RecoveryPlan は既存通り `ApplyRecoveryPlan` で適用する。
- CLI:
  - `nagare item advance` に recovery 自動承認/継続用オプションを追加する。
- Tests:
  - review changes / verification failure から recovery を経由して approval gate へ戻る自動 advance 回帰テストを追加する。
- Docs:
  - CLI usage、仕様、計画を今回の差分に同期する。

## 非対象（Out of Scope）
- 人間 approval の自動化。
- `NeedsInput` の自動回答。
- Handoff の自動作成/自動 dispatch。
- 対話型 UI の実装。
- agent adapter / storage 設計の刷新。

## Candidate Files/Artifacts
- `docs/delta/DR-20260526-auto-recovery-advance.md`
- `crates/nagare-core/src/workflow.rs`
- `crates/nagare-cli/src/commands.rs`
- `crates/nagare-cli/src/output.rs`
- `crates/nagare-core/tests/workflow_advance.rs`
- `docs/spec.md`
- `docs/architecture.md`
- `docs/plan.md`

## 差分仕様
- DS-01:
  - Given: draft RecoveryPlan が存在する
  - When: `nagare item advance <work_id>` を実行する
  - Then: WorkflowDecision は recovery accept が必要な状態を返し、command hint に `nagare item recover accept <work_id>` を出す
- DS-02:
  - Given: draft RecoveryPlan が存在する
  - When: `nagare item advance <work_id> --auto-recover true --until-blocked true` を実行する
  - Then: draft plan を accept し、次ステップで `ApplyRecoveryPlan` を実行して workflow を継続する
- DS-03:
  - Given: review changes または verification failure が発生する
  - When: `--auto-recover true --until-blocked true` で進める
  - Then: recovery 作成/承認/適用後、review/verify を再実行し、最終的に approval gate で停止する

## 受入条件（Acceptance Criteria）
- AC-01: 既定では recovery accept は人間ゲートとして残る
- AC-02: `--auto-recover true` の時だけ draft RecoveryPlan を自動 accept する
- AC-03: accepted RecoveryPlan は通常の AgentRun として適用される
- AC-04: `ReadyForReview + passing verification` では自動 approve せず停止する
- AC-05: `cargo test --workspace` が PASS する

## Verify Profile
- static check: Required
- targeted unit: Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: CLI usage と workflow 仕様が変わるため、`spec.md` / `architecture.md` / `plan.md` を同一差分で同期する。

## 制約
- `--auto-recover` の既定値は `false` とする。
- 自動 recovery は accepted/applicable な agent rerun plan に限定し、人間 approval は自動化しない。

## Review Gate
- required: Yes
- reason: workflow service、CLI、docs、回帰テストを横断するため。

## Review Focus（REVIEW または review gate required の場合）
- checklist: `docs/delta/REVIEW_CHECKLIST.md`
- target area: Workflow Advance recovery lifecycle と CLI option の後方互換性

## 未確定事項
- Q-01: なし
