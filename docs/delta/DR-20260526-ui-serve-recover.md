# delta-request

## Delta ID
- DR-20260526-ui-serve-recover

## Delta Type
- FEATURE

## 目的
- ローカルUIサーバから失敗・停止中の Work Item に RecoveryPlan を作成、承認、適用し、ブラウザ操作で完遂ルートへ戻せるようにする。

## 変更対象（In Scope）
- UI server detail: recover が次アクション、または active RecoveryPlan がある Work Item に recovery 操作を表示する。
- UI API:
  - `POST /api/items/<work_id>/recover`
  - `POST /api/items/<work_id>/recover/accept`
  - `POST /api/items/<work_id>/recover/apply`
- E2E: failed verification から recovery plan を作成、承認、適用し、Work Item が `ready_for_review` に戻ることを検証する。
- spec / plan: UIサーバ recover 操作の仕様を記録する。

## 非対象（Out of Scope）
- rerun系以外のRecoveryActionを自動適用すること。
- 複数RecoveryPlan候補の選択UI。
- 認証、権限管理、非同期ジョブ管理。

## Candidate Files/Artifacts
- crates/nagare-cli/src/ui.rs
- tests/e2e/static-ui.spec.ts
- docs/spec.md
- docs/plan.md

## 差分仕様
- DS-01:
  - Given: Work Item が recover を次アクションに持つ。
  - When: `GET /items/<work_id>` を開く。
  - Then: recovery plan 作成フォームが表示される。
- DS-02:
  - Given: draft RecoveryPlan が存在する。
  - When: accept form を送信する。
  - Then: latest draft plan が accepted になる。
- DS-03:
  - Given: accepted RecoveryPlan が rerun系actionである。
  - When: prompt または command を指定して apply form を送信する。
  - Then: RecoveryPlan が Work agent rerun として適用され、Work Item が通常の後続状態へ戻る。

## 受入条件（Acceptance Criteria）
- AC-01: recovery plan の作成、承認、適用をUIサーバから実行できる。
- AC-02: failed verification の Work Item がUI操作で `ready_for_review` に戻る。
- AC-03: E2Eは外部runtimeに依存しないテスト用 command で recovery apply を実行する。
- AC-04: `npm run test:e2e` と `cargo test --workspace` が成功する。

## Verify Profile
- static check: Required
- targeted unit: Not Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: UI server の recover 操作仕様を `docs/spec.md` に直接反映するため。

## 制約
- `recover/apply` は既存usecaseに従い、rerun系RecoveryActionのみを適用対象にする。
- 複数候補選択は後続sliceに回し、latest draft / latest accepted を既定対象にする。

## Review Gate
- required: No
- reason: 既存 recover usecase をUIサーバから呼ぶ限定差分であり、workflow logicは変更しないため。

## 未確定事項
- Q-01: なし
