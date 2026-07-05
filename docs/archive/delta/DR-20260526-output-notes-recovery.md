# delta-request

## Delta ID
- DR-20260526-output-notes-recovery

## Delta Type
- FEATURE

## 目的
- Work / Review output の `completed` / `next_notes` 欠落を警告として記録し、RecoveryPlan の専用分類で修復できるようにする。
- notes を承認必須条件にはせず、後続判断の品質低下を早く検知できる状態にする。

## 変更対象（In Scope）
- OutputContract Parser:
  - Work / Review の parsed output で `completed` または `next_notes` が空の場合、`AgentOutputRecord.warnings` に欠落警告を残す。
- Recovery:
  - 最新 Work / Review output に notes 欠落警告がある場合、`failure_class=output_notes_missing` の RecoveryPlan を作る。
- Tests:
  - notes 欠落警告と専用 RecoveryPlan が作られることを固定する。
- Docs:
  - `docs/spec.md` と `docs/plan.md` に今回の仕様を同期する。

## 非対象（Out of Scope）
- `completed` / `next_notes` を approval gate の必須条件にすること。
- notes の内容品質を LLM や rule で採点すること。
- Dispatch / WorkflowSupervision output の notes 必須化。
- 既存 AgentOutputRecord schema の大規模変更。

## Candidate Files/Artifacts
- `docs/delta/DR-20260526-output-notes-recovery.md`
- `crates/nagare-core/src/output_contract.rs`
- `crates/nagare-core/src/recovery.rs`
- `crates/nagare-core/tests/agent_output_notes.rs`
- `docs/spec.md`
- `docs/plan.md`

## 差分仕様
- DS-01:
  - Given: Work output の `## Nagare Result` に `completed` または `next_notes` がない
  - When: AgentRun を保存する
  - Then: AgentOutputRecord は parsed のまま、欠落した field ごとに warning を持つ
- DS-02:
  - Given: Review output の `## Nagare Review` に `completed` または `next_notes` がない
  - When: AgentRun を保存する
  - Then: ReviewResult の状態遷移は既存通り維持され、AgentOutputRecord は notes 欠落 warning を持つ
- DS-03:
  - Given: 最新 Work / Review output に notes 欠落 warning がある
  - When: `create_recovery_plan` を実行する
  - Then: `failure_class=output_notes_missing` の RecoveryPlan が作られ、対象 agent に contract block の再出力を促す

## 受入条件（Acceptance Criteria）
- AC-01: notes 欠落時に `missing_completed` / `missing_next_notes` warning が保存される
- AC-02: notes 欠落時に `output_notes_missing` RecoveryPlan が作られる
- AC-03: notes 欠落は approval / review / verification の既存状態遷移を壊さない
- AC-04: `cargo test -p nagare-core --test agent_output_notes` が PASS する

## Verify Profile
- static check: Required
- targeted unit: Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: OutputContract warning / recovery behavior と仕様を同じ差分で同期する。

## 制約
- notes 欠落は補助情報の不足として扱い、自動 approve を止める条件にはしない。
- required contract block の未parse警告とは分けて扱う。

## Review Gate
- required: Yes
- reason: OutputContract warning と recovery classification に影響するため。

## Review Focus（REVIEW または review gate required の場合）
- checklist: `docs/delta/REVIEW_CHECKLIST.md`
- target area: notes 欠落 warning、RecoveryPlan failure_class、既存状態遷移の維持

## 未確定事項
- Q-01: なし
