# delta-request

## Delta ID
- DR-20260526-ui-serve-info-architecture

## Delta Type
- UX

## 目的
- ローカルUIサーバの Work Item detail を、現在状態と次アクション中心の情報設計へ整理し、ユーザーが少ない視線移動で次操作を判断できるようにする。

## 変更対象（In Scope）
- UI server detail: status / next / workflow mode / execution policy をサマリに集約する。
- UI server detail: 現在必要な操作フォームだけを primary action として表示する。
- UI server detail: UI操作は単発実行で、自動継続しないことを状態情報として表示する。
- E2E: 既存の create / answer / run / review / verify / approve / recover 導線が維持されることを確認する。

## 非対象（Out of Scope）
- 実行中 child process の非同期キャンセル。
- 非同期job queue、SSE、progress log streaming。
- 確認ダイアログの追加。

## 受入条件（Acceptance Criteria）
- AC-01: detail の主導線は現在必要な action form だけを表示する。
- AC-02: detail のサマリに single step / manual continuation が表示される。
- AC-03: `npm run test:e2e` が成功する。

## Verify Profile
- static check: Required
- targeted integration / E2E: Required

## Canonical Sync Mode
- mode: direct canonical update
- reason: UI server detail の情報設計仕様を `docs/spec.md` に反映するため。
