# delta-request

## Delta ID
- DR-20260526-ui-next-action-panel

## Delta Type
- FEATURE

## 目的
- Static UI の Work Item Detail に、完遂に向けた次操作と判断材料を集約する Next Action Panel を追加する。
- ユーザーが `advance` / recovery / handoff / verify / approve のどれを実行すべきかを UI 上で迷いにくくする。

## 変更対象（In Scope）
- Static UI Detail:
  - Work Item Detail の概要付近に Next Action Panel を追加する。
  - completion、workflow decision、approval gate、active recovery、handoff、agent notes をもとに推奨コマンドと確認材料を表示する。
- Tests:
  - Static UI export の detail HTML に Next Action Panel、推奨コマンド、判断材料が出ることを固定する。
- Docs:
  - `docs/spec.md` と `docs/plan.md` に今回の仕様を同期する。

## 非対象（Out of Scope）
- ブラウザ上でコマンドを実行する対話型 UI。
- Web server / API / フロントエンド framework の導入。
- CLI の挙動変更。
- Workflow 判定ロジックの変更。

## Candidate Files/Artifacts
- `docs/delta/DR-20260526-ui-next-action-panel.md`
- `crates/nagare-core/src/ui.rs`
- `crates/nagare-core/tests/static_ui_export.rs`
- `docs/spec.md`
- `docs/plan.md`

## 差分仕様
- DS-01:
  - Given: Work Item Detail を Static UI export する
  - When: `<out>/items/<work_id>.html` を確認する
  - Then: Next Action Panel に state、next action、recommended command、workflow mode が表示される
- DS-02:
  - Given: Work Item に draft RecoveryPlan が存在する
  - When: Work Item Detail を確認する
  - Then: `nagare item recover accept <work_id>` が推奨コマンドとして表示され、recovery の failure_class と action が判断材料として表示される
- DS-03:
  - Given: approval gate、handoff、agent notes が存在する
  - When: Work Item Detail を確認する
  - Then: Next Action Panel から approval / handoff / notes の要点を確認できる

## 受入条件（Acceptance Criteria）
- AC-01: Detail HTML に `Next Action Panel` が表示される
- AC-02: draft RecoveryPlan がある場合、推奨コマンドとして `nagare item recover accept <work_id>` が表示される
- AC-03: workflow mode、completion next_action、active recovery、agent notes が同じ panel で確認できる
- AC-04: `cargo test -p nagare-core --test static_ui_export` が PASS する

## Verify Profile
- static check: Required
- targeted unit: Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: UI 表示仕様と実装を同じ差分で同期する。

## 制約
- Static UI はコマンド実行せず、実行すべき CLI command を表示するだけにする。
- 既存の Board / Detail / Inspector 表示を壊さない。

## Review Gate
- required: Yes
- reason: UI の主要導線と完遂率改善に影響するため。

## Review Focus（REVIEW または review gate required の場合）
- checklist: `docs/delta/REVIEW_CHECKLIST.md`
- target area: next action の優先順位、表示情報の過不足、既存 detail layout との整合

## 未確定事項
- Q-01: なし
