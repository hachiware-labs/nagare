# delta-request

## Delta ID
- DR-20260526-finish-first-handoff-resume

## Delta Type
- FEATURE

## 目的
- `finish_first` の Work Item が、作成済み HandoffPacket から dispatch / accept / run に復帰できることを workflow 回帰として固定する。
- HandoffPacket の作成自体は人間ゲートに残し、handoff 後の復帰導線だけを完走優先に寄せる。

## 変更対象（In Scope）
- Workflow Advance:
  - `NeedsHandoff` かつ HandoffPacket が存在する Work Item を `finish_first` で停止点まで進める。
  - handoff dispatch、dispatch accept、target agent run、review、verify、approval gate まで戻る動線を固定する。
- Tests:
  - 作成済み HandoffPacket から `advance --until-blocked` が approval gate まで進む回帰テストを追加する。
- Docs:
  - Workflow Advance 仕様と plan に finish-first handoff resume を追加する。

## 非対象（Out of Scope）
- HandoffPacket の自動作成。
- handoff 先 agent の完全自動判断。
- approval の自動化。
- NeedsInput の自動回答。
- UI の対話フォーム追加。

## Candidate Files/Artifacts
- `docs/delta/DR-20260526-finish-first-handoff-resume.md`
- `crates/nagare-core/src/workflow.rs`
- `crates/nagare-core/src/snapshot.rs`
- `crates/nagare-core/tests/workflow_handoff.rs`
- `docs/spec.md`
- `docs/plan.md`

## 差分仕様
- DS-01:
  - Given: `workflow_mode=finish_first` の Work Item が `NeedsHandoff` で、HandoffPacket が存在する
  - When: `nagare item advance <work_id> --until-blocked true` を実行する
  - Then: handoff dispatch、dispatch accept、target agent run、review、verify を経て approval gate で停止する
- DS-02:
  - Given: Work Item が `NeedsHandoff` だが HandoffPacket が存在しない
  - When: `nagare item advance <work_id> --until-blocked true` を実行する
  - Then: `CreateHandoff` で停止し、packet 作成は人間ゲートとして残る

## 受入条件（Acceptance Criteria）
- AC-01: 作成済み HandoffPacket から approval gate まで進む回帰テストが PASS する
- AC-02: HandoffPacket 未作成時は `CreateHandoff` で停止する既存挙動を維持する
- AC-03: 自動 approve は行われない
- AC-04: `cargo test -p nagare-core --test workflow_handoff` が PASS する

## Verify Profile
- static check: Required
- targeted unit: Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: finish-first workflow semantics と仕様を同じ差分で同期する。

## 制約
- handoff 作成は人間が行う。
- finish-first でも approval は停止点にする。

## Review Gate
- required: Yes
- reason: Workflow Advance の完遂挙動を変更/固定するため。

## Review Focus（REVIEW または review gate required の場合）
- checklist: `docs/delta/REVIEW_CHECKLIST.md`
- target area: handoff resume の停止条件と自動承認しない不変条件

## 未確定事項
- Q-01: なし
