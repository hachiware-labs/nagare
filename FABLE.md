# Fable-5 への指示

> **追記(2026-07-05): このUI再作成は完了した。**
> 現在の正は `docs/design-assets/prototype/index.html`(プロトタイプ)と
> `docs/nagare_prd_v1_0.md`(PRD+機能一覧)。
> 以下は初回作成時の指示であり、本文中の旧文書の多くは `docs/archive/` へ移動済み。

Nagare のUIを Fable-5 で作り直す。最初の作業では、既存UIの見た目をなぞらず、
機能概要から新しい画面構成を作る。

## 最初に読むもの

1. `docs/fable-5-ui-context.md`
   - Nagare の機能概要、主要画面、ユーザー向けオブジェクト、作業状態を理解する。
2. `docs/design-assets/nagare_screen_flow_summary.md`
   - 初回セットアップから作業完了までの流れを理解する。
3. `logo.png`
   - ブランドマークとして参照する。

## 初回では読まないもの

次の文書や資産は、初回UI再作成の主入力にしない。

- `docs/design.md`
- `docs/design-assets/DESIGN.md`
- `docs/design-assets/UI_PLAN.md`
- `docs/design-assets/` 配下の既存 wireframe
- `docs/spec.md`
- `docs/architecture.md`
- `docs/agent_data_model.md`
- `docs/agent-management-usecases.md`

これらは詳細確認が必要になった時だけ読む。特にデータ構造で詰まった場合のみ
`docs/agent_data_model.md` を参照する。

## 作るUIの目的

UIが最優先で答えるべき問いはこれである。

> この作業は今どうなっていて、次に自分は何をすればよいか。

Nagare は Agent 管理画面ではなく、作業を前に進めるための制御画面である。
主役は常に `Work Item`、ユーザー向けには `依頼 / 作業` として扱う。

## 必ず含める主な機能

- 初回セットアップ
  - プロジェクトまたは作業フォルダの作成・選択
  - 検出済みAI実行環境の選択
  - 作業作成前の接続確認
- 作業ホーム
  - 自然文での作業依頼入力
  - 既存作業の状態別一覧
  - 対応が必要な作業の強調
- 作業詳細 / 実行トレース
  - 現在状態、現在工程、担当エージェント、次の操作
  - raw log ではなく読める履歴
  - 診断情報は明示的に開く詳細へ分離
- 人間の判断点
  - 質問への回答
  - 回復案の選択または適用
  - 結果の確認
  - 承認・採用、またはコメント付き差し戻し
- 結果と履歴
  - 生成された成果物または変更ファイル
  - レビュー判定、確認条件、根拠、懸念点
  - 完了作業の短い最終サマリー

## 作業状態の表記

状態は、人間が対応すべきかどうかが分かる言葉にする。

- `処理中`
- `要対応・質問`
- `要対応・確認`
- `要対応・回復`
- `完了`

## デザインで残すもの

- `logo.png` をブランド参照として使う。
- Hachiware Labs のテイストを残す。
  - 静かで実用的
  - 白い面
  - slate系テキスト
  - indigo の primary action
  - 細い境界線
  - コンパクトな密度
  - 7-8px程度の角丸
- 色は意味に使う。
  - primary / selection
  - success
  - warning
  - danger
  - disabled

## 避けること

- 既存 wireframe の画面構図をそのまま再現する。
- 派手なAIダッシュボードにする。
- マーケティングheroを主役にする。
- 装飾的なグラデーションや飾りカードを増やす。
- raw log、内部ID、runtime詳細、完全な provenance を標準画面に出す。
- Agent、Skill、MCP、Runtime の管理を初回利用の主役にする。

## 成果物の期待

まずは、機能概要から次の画面群を再設計する。

1. 初回セットアップ
2. 作業ホーム
3. 作業依頼入力
4. 作業詳細 / 実行トレース
5. 質問・確認・回復の人間判断状態
6. 結果レビュー
7. 完了サマリー

Project、Knowledge、Agent、Runtime、Skill、MCP の管理画面は補助画面として扱い、
主作業フローが分かるようになってから設計する。
