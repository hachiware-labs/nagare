# delta-request

## Delta ID
- DR-20260526-agent-output-notes

## Delta Type
- FEATURE

## 目的
- Work / Review agent の最終 Output に、実施内容と次工程へのコメントを構造化して残せるようにする。
- 後続 agent、review、verification、approval が直前の意図を拾いやすくし、途中再開と handoff の安定性を上げる。

## 変更対象（In Scope）
- OutputContract:
  - `## Nagare Result` と `## Nagare Review` の必須構造に `completed` と `next_notes` を追加する。
- Snapshot / CLI:
  - `AgentOutputRecord.fields` に保存された `completed` / `next_notes` を `item show` で確認できるようにする。
- Static UI:
  - Work Item detail に最新 Agent Output Notes を表示する。
- Tests:
  - Work / Review output の `completed` / `next_notes` が parse されることを固定する。
  - Static UI detail に Agent Output Notes が出ることを固定する。
- Docs:
  - `docs/spec.md` と `docs/plan.md` に今回の仕様を同期する。

## 非対象（Out of Scope）
- Dispatch JSON schema の拡張。
- WorkflowSupervision output schema の拡張。
- completed / next_notes を承認可否の必須条件にすること。
- LLM による note 品質評価。
- ledger schema の大規模変更。

## Candidate Files/Artifacts
- `docs/delta/DR-20260526-agent-output-notes.md`
- `crates/nagare-core/src/output_contract.rs`
- `crates/nagare-cli/src/output.rs`
- `crates/nagare-core/src/ui.rs`
- `crates/nagare-core/tests/agent_output_notes.rs`
- `crates/nagare-core/tests/static_ui_export.rs`
- `docs/spec.md`
- `docs/plan.md`

## 差分仕様
- DS-01:
  - Given: Work agent が `## Nagare Result` に `completed` と `next_notes` を含める
  - When: AgentRun が保存される
  - Then: AgentOutputRecord.fields に `completed` と `next_notes` が保存され、snapshot から取得できる
- DS-02:
  - Given: Review agent が `## Nagare Review` に `completed` と `next_notes` を含める
  - When: AgentRun が保存される
  - Then: AgentOutputRecord.fields に `completed` と `next_notes` が保存され、review status 遷移は既存通り維持される
- DS-03:
  - Given: Work Item に notes 付き AgentOutputRecord が存在する
  - When: `nagare item show` または Static UI detail を確認する
  - Then: completed / next_notes が表示される

## 受入条件（Acceptance Criteria）
- AC-01: Work / Review output の `completed` と `next_notes` が parse される回帰テストが PASS する
- AC-02: `item show` の `agent_outputs` に completed / next_notes が表示される
- AC-03: Static UI detail に Agent Output Notes が表示される
- AC-04: 既存の next_action / questions / review verdict の状態遷移を壊さない
- AC-05: `cargo test -p nagare-core --test agent_output_notes` と `cargo test -p nagare-core --test static_ui_export` が PASS する

## Verify Profile
- static check: Required
- targeted unit: Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: OutputContract の仕様と実装を同じ差分で同期する。

## 制約
- `completed` と `next_notes` は補助判断材料であり、approval の必須条件にはしない。
- 既存の OutputContract block を parse する互換性を維持する。

## Review Gate
- required: Yes
- reason: Agent output contract と後続 workflow の判断材料に影響するため。

## Review Focus（REVIEW または review gate required の場合）
- checklist: `docs/delta/REVIEW_CHECKLIST.md`
- target area: output contract instruction、field parse、CLI/UI 表示、既存状態遷移

## 未確定事項
- Q-01: なし
