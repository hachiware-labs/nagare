# delta-request

## Delta ID
- DR-20260526-ui-serve-run

## Delta Type
- FEATURE

## 目的
- ローカルUIサーバから `ready` の Work Item を agent run へ進め、ブラウザ操作で次の review 導線へ到達できるようにする。

## 変更対象（In Scope）
- UI server detail: `ready` / `run_agent` の Work Item に agent run form を表示する。
- UI API: `POST /api/items/<work_id>/run` で Work agent を実行する。
- E2E: ブラウザから run form を送信し、`ready_for_review` と実行履歴表示を検証する。
- spec / plan: UIサーバ agent run 操作の仕様を記録する。

## 非対象（Out of Scope）
- review / verify / approve のUIサーバAPI。
- 外部agent runtimeを必須にするE2E。
- 長期運用向けサーバ設計、認証、非同期ジョブ管理。

## Candidate Files/Artifacts
- crates/nagare-cli/src/ui.rs
- tests/e2e/static-ui.spec.ts
- docs/spec.md
- docs/plan.md

## 差分仕様
- DS-01:
  - Given: Work Item が `ready` である。
  - When: `GET /items/<work_id>` を開く。
  - Then: agent run form が表示される。
- DS-02:
  - Given: DS-01 の detail page が表示されている。
  - When: run form を送信する。
  - Then: `POST /api/items/<work_id>/run` が Work agent run を保存し、Work Item は `ready_for_review` へ進む。
- DS-03:
  - Given: DS-02 の送信後である。
  - When: detail page を再表示する。
  - Then: status、next action、timeline/evidence が更新される。

## 受入条件（Acceptance Criteria）
- AC-01: `ready` detail に run form が表示される。
- AC-02: ブラウザから run form を送信すると Work Item が `ready_for_review` になる。
- AC-03: E2Eは外部runtimeに依存しないテスト用 command で run を実行する。
- AC-04: `npm run test:e2e` と `cargo test --workspace` が成功する。

## Verify Profile
- static check: Required
- targeted unit: Not Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: UI server の agent run 操作仕様を `docs/spec.md` に直接反映するため。

## 制約
- E2Eは一時project内の fixture command で実行する。
- 既存CLIの run semantics は変更しない。

## Review Gate
- required: No
- reason: 既存 run usecase をUIサーバから呼ぶ限定差分であり、workflow logicは変更しないため。

## 未確定事項
- Q-01: なし
