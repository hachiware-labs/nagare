# delta-request

## Delta ID
- DR-20260526-workflow-mode

## Delta Type
- FEATURE

## 目的
- Work Item 作成時に `confirm_first` / `finish_first` の workflow mode を選べるようにする。
- `finish_first` の Work Item は、明示的な `--auto-recover true` がなくても applicable な recovery accept/apply を advance ループで継続する。

## 変更対象（In Scope）
- Work Item model:
  - `workflow_mode` を保存し、既存 ledger では `confirm_first` を既定値にする。
- CLI:
  - `nagare item create` で `--workflow-mode confirm_first|finish_first` を受け付ける。
  - `nagare item advance` で一時 override 用の `--workflow-mode confirm_first|finish_first` を受け付ける。
- Workflow Advance:
  - `finish_first` の Work Item は `--auto-recover true` 相当で recovery accept/apply を継続する。
  - `confirm_first` では既存通り recovery accept で停止する。
- Display / UI:
  - `item show` と static UI detail で workflow mode を確認できるようにする。
- Tests / Docs:
  - workflow mode による finish-first advance を回帰テストで固定し、CLI usage と仕様を同期する。

## 非対象（Out of Scope）
- approval の自動化。
- NeedsInput の自動回答。
- Handoff の自動作成/dispatch。
- 対話型 UI の入力フォーム実装。
- Project default workflow mode の追加。
- agent adapter / storage backend の刷新。

## Candidate Files/Artifacts
- `docs/delta/DR-20260526-workflow-mode.md`
- `crates/nagare-core/src/workflow_policy.rs`
- `crates/nagare-core/src/lib.rs`
- `crates/nagare-core/src/model.rs`
- `crates/nagare-core/src/result_types.rs`
- `crates/nagare-core/src/work_items.rs`
- `crates/nagare-core/src/workflow.rs`
- `crates/nagare-core/src/ui.rs`
- `crates/nagare-core/src/workflow_types.rs`
- `crates/nagare-cli/src/commands.rs`
- `crates/nagare-cli/src/output.rs`
- `crates/nagare-core/tests/workflow_advance.rs`
- `crates/nagare-core/tests/static_ui_export.rs`
- `docs/spec.md`
- `docs/architecture.md`
- `docs/plan.md`

## 差分仕様
- DS-01:
  - Given: Project が初期化済み
  - When: `nagare item create --title <title> --workflow-mode finish_first` を実行する
  - Then: Work Item に `workflow_mode=finish_first` が保存され、`item show` で確認できる
- DS-02:
  - Given: `workflow_mode=confirm_first` の Work Item に draft RecoveryPlan が存在する
  - When: `nagare item advance <work_id>` を実行する
  - Then: `accept_recovery_plan` の判断で停止し、人間確認を要求する
- DS-03:
  - Given: `workflow_mode=finish_first` の Work Item に applicable な draft RecoveryPlan が存在する
  - When: `nagare item advance <work_id> --until-blocked true` を実行する
  - Then: draft plan を accept し、accepted plan を apply して、approval / input / handoff などの人間ゲートまで継続する
- DS-04:
  - Given: Work Item が `workflow_mode=finish_first` であっても approval gate に到達する
  - When: `advance --until-blocked true` が approval 判断を作る
  - Then: Work Item は自動 approve されず `Approve` で停止する

## 受入条件（Acceptance Criteria）
- AC-01: Work Item は `workflow_mode` を永続化し、既存データは `confirm_first` として読める
- AC-02: CLI create / show / help で workflow mode を確認できる
- AC-03: `finish_first` の Work Item は `--auto-recover true` なしで recovery accept/apply を継続する
- AC-04: `confirm_first` または `--workflow-mode confirm_first` では recovery accept gate で停止する
- AC-05: static UI detail に workflow mode が表示される
- AC-06: `cargo test --workspace` が PASS する

## Verify Profile
- static check: Required
- targeted unit: Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: Work Item data model、CLI usage、workflow semantics が変わるため、仕様と計画を同一差分で同期する。

## 制約
- 既定値は `confirm_first` とする。
- `--auto-recover true` は互換性のため残し、`finish_first` と同じ recovery 自動継続を有効にする。
- `finish_first` でも approval / input / handoff は自動完了しない。

## Review Gate
- required: Yes
- reason: Work Item model、workflow service、CLI、UI export、docs、tests を横断するため。

## Review Focus（REVIEW または review gate required の場合）
- checklist: `docs/delta/REVIEW_CHECKLIST.md`
- target area: workflow mode の後方互換性、confirm-first の停止挙動、finish-first の自動 recovery 継続

## 未確定事項
- Q-01: なし
