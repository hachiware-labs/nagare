# delta-request

## Delta ID
- DR-20260526-ui-visibility-input

## Delta Type
- FEATURE

## 目的
- Static UI で確認が必要な Work Item を一覧から視認しやすくし、Detail で追加指示または回答の CLI command を少ない手順で作れるようにする。

## 変更対象（In Scope）
- Board UI: 要確認 Work Item の優先表示と行の視覚強調。
- Detail UI: Next Action Panel 近傍に人間入力用の支援パネルを追加する。
- Static UI test / spec / plan: 追加表示と入力支援を検証・記録する。

## 非対象（Out of Scope）
- ブラウザから CLI command を直接実行する機能。
- サーバ/API/SPA 化。
- Work Item 状態遷移、workflow decision、recovery logic の変更。
- CLI command 体系の変更。

## Candidate Files/Artifacts
- crates/nagare-core/src/ui.rs
- crates/nagare-core/tests/static_ui_export.rs
- docs/spec.md
- docs/plan.md

## 差分仕様
- DS-01:
  - Given: Work Item が `done` / `agent_running` 以外の停止または確認状態である。
  - When: Static UI Board を export する。
  - Then: `確認キュー` に対象 Work Item、理由、次アクション、Detail link が表示され、一覧行も `attention` として識別できる。
- DS-02:
  - Given: Work Item が `needs_input` または追加指示を受け付ける状態である。
  - When: Static UI Detail を export する。
  - Then: `Human Input Panel` に最新質問または追加指示対象、textarea、生成される CLI command、copy button が表示される。
- DS-03:
  - Given: ユーザーが Human Input Panel の textarea に入力する。
  - When: 生成 script が動作するブラウザで入力値が変わる。
  - Then: `nagare item answer <work_id> --answer "<text>"` または `nagare item advance <work_id> --until-blocked true --prompt "<text>"` の command が更新される。

## 受入条件（Acceptance Criteria）
- AC-01: Board HTML に `確認キュー` が表示され、要確認 Work Item の title、blocking reason、next action、Detail link が含まれる。
- AC-02: Board の要確認行に `attention-row` class が付与される。
- AC-03: Detail HTML に `Human Input Panel`、textarea、copy button、入力用 command template が表示される。
- AC-04: `needs_input` の Work Item では `nagare item answer <work_id> --answer` が表示される。
- AC-05: `cargo test -p nagare-core --test static_ui_export` が成功する。

## Verify Profile
- static check: Required
- targeted unit: Required
- targeted integration / E2E: Not Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: Static UI の表示仕様を `docs/spec.md` に直接反映するため。

## 制約
- Static HTML と inline JavaScript のみで完結し、外部ビルドツールを追加しない。
- 既存の Next Action Panel と CLI command を壊さない。

## Review Gate
- required: No
- reason: Static UI export の限定差分であり、workflow logic は変更しないため。

## 未確定事項
- Q-01: なし
