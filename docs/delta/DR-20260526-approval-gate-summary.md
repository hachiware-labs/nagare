# delta-request

## Delta ID
- DR-20260526-approval-gate-summary

## Delta Type
- FEATURE

## 目的
- approval gate で人間が確認すべき判断材料を WorkItemSnapshot、CLI、Static UI に明示する。
- `finish_first` / `confirm_first` のどちらでも、完走後の最終確認が薄くならないようにする。

## 変更対象（In Scope）
- Snapshot:
  - Work Item の approval gate summary を追加する。
  - 最新 Work run 後の verification / review / acceptance criteria 充足を判定材料として持つ。
- CLI:
  - `item show` の出力に approval gate summary を表示する。
- Static UI:
  - Work Item detail に approval gate summary を表示する。
- Tests:
  - approval gate 到達時の summary が ready になることを回帰テストで固定する。
  - 最新 Work run 後に verification が古くなった場合は approval gate が blocked になることを固定する。

## 非対象（Out of Scope）
- approval の自動実行。
- UI の対話式 approval ボタン。
- 外部サービスとの approval 連携。
- Review agent の判定品質そのものの変更。

## Candidate Files/Artifacts
- `docs/delta/DR-20260526-approval-gate-summary.md`
- `crates/nagare-core/src/snapshot.rs`
- `crates/nagare-core/src/lib.rs`
- `crates/nagare-core/src/usecases.rs`
- `crates/nagare-cli/src/output.rs`
- `crates/nagare-core/src/ui.rs`
- `crates/nagare-core/tests/approval_gate.rs`
- `crates/nagare-core/tests/static_ui_export.rs`
- `docs/spec.md`
- `docs/plan.md`

## 差分仕様
- DS-01:
  - Given: Work Item が最新 Work run 後の review / verification / criteria を満たしている
  - When: snapshot または `nagare item show <work_id>` を確認する
  - Then: approval gate summary が `ready=true` と承認コマンド hint を示す
- DS-02:
  - Given: Work Item の verification が最新 Work run より古い
  - When: snapshot を確認する
  - Then: approval gate summary が `ready=false` かつ `verification_not_passed` blocker を示す
- DS-03:
  - Given: Static UI export を実行する
  - When: Work Item detail を開く
  - Then: Approval Gate セクションに criteria、review、verification、blockers が表示される

## 受入条件（Acceptance Criteria）
- AC-01: approval gate summary の ready / blocked 状態を検証する回帰テストが PASS する
- AC-02: CLI `item show` に approval gate summary が出る
- AC-03: Static UI detail に Approval Gate セクションが出る
- AC-04: 自動 approve は追加されない
- AC-05: `cargo test -p nagare-core --test approval_gate` が PASS する

## Verify Profile
- static check: Required
- targeted unit: Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: approval gate の観測面と仕様を同じ差分で同期する。

## 制約
- approval は人間ゲートのまま維持する。
- 古い verification を ready 判定に含めない。

## Review Gate
- required: Yes
- reason: 承認判断に使う情報を変更するため。

## Review Focus（REVIEW または review gate required の場合）
- checklist: `docs/delta/REVIEW_CHECKLIST.md`
- target area: latest-work-after 判定、criteria summary、UI/CLI 表示の過不足

## 未確定事項
- Q-01: なし
