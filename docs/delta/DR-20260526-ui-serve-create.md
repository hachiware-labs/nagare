# delta-request

## Delta ID
- DR-20260526-ui-serve-create

## Delta Type
- FEATURE

## 目的
- CLIを意識せず、ブラウザUIから Work Item を作成できる最小のローカルUIサーバを追加する。

## 変更対象（In Scope）
- CLI: `nagare ui serve` を追加し、ローカルHTTPサーバを起動する。
- UI server: `GET /` で Work Item 作成フォームと一覧を表示する。
- UI API: `POST /api/items` で Work Item を作成し、JSONを返す。
- E2E: Playwrightでブラウザからフォーム入力・作成・一覧反映を検証する。
- spec / plan: UI server の最小仕様を記録する。

## 非対象（Out of Scope）
- 回答、agent run、review、approve をUIボタンから直接実行するAPI。
- 認証、常駐サービス、外部公開向けサーバ。
- SPA framework や外部Web framework の導入。
- 既存 Static UI export の削除。

## Candidate Files/Artifacts
- crates/nagare-cli/src/ui.rs
- crates/nagare-cli/src/output.rs
- tests/e2e/static-ui.spec.ts
- docs/spec.md
- docs/plan.md

## 差分仕様
- DS-01:
  - Given: Nagare project が初期化済みである。
  - When: `nagare ui serve --host 127.0.0.1 --port <port> --open false` を実行する。
  - Then: ローカルHTTPサーバが起動し、`GET /` で作成フォームとWork Item一覧を返す。
- DS-02:
  - Given: UI server が起動している。
  - When: ブラウザから title / description / acceptance / workflow_mode を入力して作成する。
  - Then: `POST /api/items` が Work Item を作成し、ブラウザ上の一覧に新規項目が表示される。
- DS-03:
  - Given: DS-02 の作成後である。
  - When: CLIまたはAPIから状態を確認する。
  - Then: 作成された Work Item が ledger に保存され、workflow mode と acceptance criteria が保持される。

## 受入条件（Acceptance Criteria）
- AC-01: `nagare ui serve --open false --port <port>` が起動し、`GET /` が200を返す。
- AC-02: UIフォームから Work Item を作成でき、一覧に追加される。
- AC-03: E2Eで作成後の `nagare item list` に新規 title が含まれる。
- AC-04: `npm run test:e2e` と `cargo test --workspace` が成功する。

## Verify Profile
- static check: Required
- targeted unit: Not Required
- targeted integration / E2E: Required
- delta-project-validator: code-size-only

## Canonical Sync Mode
- mode: direct canonical update
- reason: UI server の仕様を `docs/spec.md` に直接反映するため。

## 制約
- 初期実装は `127.0.0.1` 既定でローカル確認用途に限定する。
- APIはテストしやすい単純な form-urlencoded / JSON レスポンスでよい。

## Review Gate
- required: No
- reason: 最小ローカルHTTPサーバとWork Item作成APIに限定し、既存workflow logicは変更しないため。

## 未確定事項
- Q-01: なし
