# delta-request

## Delta ID
- DR-20260526-ui-serve-verify

## Delta Type
- FEATURE

## 目的
- ローカルUIサーバから `ready_for_verification` の Work Item を検証し、ブラウザ操作で approval gate まで到達できるようにする。

## 変更対象（In Scope）
- UI server detail: `verify` が次アクションの Work Item に verify form を表示する。
- UI API: `POST /api/items/<work_id>/verify` で verification command を実行する。
- E2E: ブラウザから verify form を送信し、passing verification と approval next action を検証する。
- spec / plan: UIサーバ verify 操作の仕様を記録する。

## 非対象（Out of Scope）
- approve / recover のUIサーバAPI。
- 非同期ジョブ管理や認証。
- 外部agent runtimeを必須にするE2E。

## Candidate Files/Artifacts
- crates/nagare-cli/src/ui.rs
- tests/e2e/static-ui.spec.ts
- docs/spec.md
- docs/plan.md

## 差分仕様
- DS-01:
  - Given: Work Item が `ready_for_verification` である。
  - When: `GET /items/<work_id>` を開く。
  - Then: verify form が表示される。
- DS-02:
  - Given: DS-01 の detail page が表示されている。
  - When: verification command を送信する。
  - Then: `POST /api/items/<work_id>/verify` が VerificationResult を保存し、成功時は approval に進める状態になる。
- DS-03:
  - Given: DS-02 の送信後である。
  - When: detail page を再表示する。
  - Then: status、next action、approval gate が更新される。

## 受入条件（Acceptance Criteria）
- AC-01: `ready_for_verification` detail に verify form が表示される。
- AC-02: ブラウザから verify form を送信すると passing verification が保存される。
- AC-03: E2Eは外部runtimeに依存しないテスト用 command で verify を実行する。
- AC-04: `npm run test:e2e` と `cargo test --workspace` が成功する。

## Verify Profile
- static check: Required
- targeted unit: Not Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: UI server の verify 操作仕様を `docs/spec.md` に直接反映するため。

## 制約
- E2Eは一時project内の fixture command で実行する。
- 既存CLIの verification semantics は変更しない。

## Review Gate
- required: No
- reason: 既存 verify usecase をUIサーバから呼ぶ限定差分であり、workflow logicは変更しないため。

## 未確定事項
- Q-01: なし
