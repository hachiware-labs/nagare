# delta-request

## Delta ID
- DR-20260526-ui-serve-answer

## Delta Type
- FEATURE

## 目的
- ローカルUIサーバで `needs_input` の Work Item に回答し、CLIを介さずブラウザ操作で状態を `ready` に戻せるようにする。

## 変更対象（In Scope）
- UI server: Work Item detail page を追加する。
- UI API: `POST /api/items/<work_id>/answer` で HumanFeedback を保存する。
- UI: `needs_input` の detail に回答フォームを表示し、送信後に状態を更新する。
- E2E: ブラウザから回答し、一覧/detail/CLIで状態変化を検証する。
- spec / plan: UIサーバ回答操作の仕様を記録する。

## 非対象（Out of Scope）
- agent run / review / approve をUIサーバから直接実行するAPI。
- 認証、永続HTTPサービス化、外部公開対応。
- Static UI export の削除。

## Candidate Files/Artifacts
- crates/nagare-cli/src/ui.rs
- tests/e2e/static-ui.spec.ts
- docs/spec.md
- docs/plan.md

## 差分仕様
- DS-01:
  - Given: Work Item が `needs_input` である。
  - When: `GET /items/<work_id>` を開く。
  - Then: Work Item status、最新質問、回答フォームが表示される。
- DS-02:
  - Given: DS-01 の detail page が表示されている。
  - When: 回答フォームから回答を送信する。
  - Then: `POST /api/items/<work_id>/answer` が HumanFeedback を保存し、Work Item status が `ready` になる。
- DS-03:
  - Given: DS-02 の送信後である。
  - When: detail page と一覧を再表示する。
  - Then: detail に `ready` と回答内容が表示され、一覧にも `ready` が反映される。

## 受入条件（Acceptance Criteria）
- AC-01: `GET /items/<work_id>` が Work Item detail を表示する。
- AC-02: `needs_input` detail に回答フォームと最新質問が表示される。
- AC-03: ブラウザから回答すると `HumanFeedback` が保存され、`nagare item show` または一覧で `ready` が確認できる。
- AC-04: `npm run test:e2e` と `cargo test --workspace` が成功する。

## Verify Profile
- static check: Required
- targeted unit: Not Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: UI server の回答操作仕様を `docs/spec.md` に直接反映するため。

## 制約
- 初期実装はローカルHTTPサーバ用途に限定する。
- APIは form-urlencoded でよい。

## Review Gate
- required: No
- reason: Work Item answer の既存usecaseをUIサーバから呼ぶ限定差分であり、workflow logic は変更しないため。

## 未確定事項
- Q-01: なし
