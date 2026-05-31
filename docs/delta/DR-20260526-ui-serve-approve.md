# delta-request

## Delta ID
- DR-20260526-ui-serve-approve

## Delta Type
- FEATURE

## 目的
- ローカルUIサーバから approval gate ready の Work Item を承認し、ブラウザ操作で `done` まで到達できるようにする。

## 変更対象（In Scope）
- UI server detail: approval gate ready の Work Item に approve form を表示する。
- UI API: `POST /api/items/<work_id>/approve` で human approval を保存する。
- E2E: ブラウザから approve form を送信し、`done` と approval decision を検証する。
- spec / plan: UIサーバ approve 操作の仕様を記録する。

## 非対象（Out of Scope）
- reject / request_changes / pause / delegate / override のUI。
- recover のUIサーバAPI。
- 認証、権限管理、非同期ジョブ管理。

## Candidate Files/Artifacts
- crates/nagare-cli/src/ui.rs
- tests/e2e/static-ui.spec.ts
- docs/spec.md
- docs/plan.md

## 差分仕様
- DS-01:
  - Given: Work Item の approval gate が ready である。
  - When: `GET /items/<work_id>` を開く。
  - Then: approve form が表示される。
- DS-02:
  - Given: DS-01 の detail page が表示されている。
  - When: rationale を任意入力して approve form を送信する。
  - Then: `POST /api/items/<work_id>/approve` が HumanDecision を保存し、Work Item が `done` になる。
- DS-03:
  - Given: DS-02 の送信後である。
  - When: detail page を再表示する。
  - Then: status と next action が完了状態として表示される。

## 受入条件（Acceptance Criteria）
- AC-01: approval gate ready detail に approve form が表示される。
- AC-02: ブラウザから approve form を送信すると Work Item が `done` になる。
- AC-03: E2Eは create / answer / run / review / verify / approve をブラウザ操作で検証する。
- AC-04: `npm run test:e2e` と `cargo test --workspace` が成功する。

## Verify Profile
- static check: Required
- targeted unit: Not Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: UI server の approve 操作仕様を `docs/spec.md` に直接反映するため。

## 制約
- 既存CLIの approval semantics は変更しない。
- approval gate が ready でない場合の拒否は既存 usecase に従う。

## Review Gate
- required: No
- reason: 既存 approve usecase をUIサーバから呼ぶ限定差分であり、workflow logicは変更しないため。

## 未確定事項
- Q-01: なし
