# Fable-5 向けプロダクト文脈

> **追記(2026-07-05): UI再作成は完了した。** 現在の正は
> `design-assets/prototype/index.html` と `nagare_prd_v1_0.md`。
> 本文中の参照文書の多くは `archive/` へ移動済み。この文書は経緯として残す。

この文書は、Fable-5 で Nagare のUIを作り直すための入口である。
引き継ぎ内容は機能概要を中心にし、デザイン指定はロゴと Hachiware Labs の
テイストに絞る。

## Nagare とは何か

Nagare は、AI支援の作業をローカル優先で進めるための制御画面である。

ユーザーの依頼を Work Item にし、適切な Agent に割り当て、作業を実行し、
結果をレビュー・検証し、人間の判断が必要な時だけ質問や承認を出し、
成果物・根拠・履歴を後から確認できるようにする。

UI が最優先で答えるべき問いは次の1つである。

> この作業は今どうなっていて、次に自分は何をすればよいか。

## 主な機能領域

### 1. 初回セットアップ

- 最初のプロジェクトまたは作業フォルダを作成・選択する。
- 検出済みのAI実行環境を選択する。
- 最初の作業を作る前に接続確認を行う。
- 既定Agentの配線は、ユーザーが設定を開くまで見せすぎない。

### 2. Work Home

- 自然文の依頼入力から新しい作業を作る。
- 既存作業を、人間にとって意味のある状態ごとに表示する。
- 対応が必要な作業を見つけやすくする。
- 行の操作から1つの Work Item 詳細を開けるようにする。

### 3. Work Detail / Run Trace

- 現在の状態、現在の工程、担当Agent、次の操作を表示する。
- 進行状況は raw log ではなく、読める履歴として見せる。
- 診断情報、内部ID、runtime詳細、完全な provenance は、明示的に開く詳細表示へ寄せる。

### 4. 人間の判断点

- Agent からの質問に回答する。
- 回復案を選ぶ、または適用する。
- 作成された結果を確認する。
- 結果を承認・採用する、またはコメント付きで差し戻す。

### 5. 結果と履歴

- 生成された成果物または変更ファイルを表示する。
- レビュー判定、確認条件、根拠、懸念点を表示する。
- 完了した作業は、最終サマリーを短く表示する。

### 6. 補助的な管理画面

これらは主作業の流れを支える画面であり、初回利用時の主役にしない。

- Project
- Knowledge Domain / Knowledge Artifact
- Agent
- Runtime
- Skill
- MCP connection

## ユーザー向け主要オブジェクト

| Object | ユーザー向け表記 | 役割 |
| --- | --- | --- |
| Work Item | 依頼 / 作業 | アプリの中心になる作業単位 |
| Project | プロジェクト | 作業フォルダ、参加Agent、プロジェクト知識 |
| Knowledge Domain | 知識 / ドメイン | 共有知識と生成物定義 |
| Knowledge Artifact | 生成物 | 出力種別と評価ルーブリック |
| Agent | エージェント / 担当AI | Organizer、worker、reviewer |
| Runtime | 実行環境 | セットアップや問題対応が必要な時だけ見せる |
| Artifact | 変更案 / 成果物 | 具体的な結果または変更ファイル |
| Evidence | 根拠 | 補助的な証拠、レビュー材料 |
| Diagnostics | 詳細 / 診断 | 開発者向け詳細。通常は閉じる |

## 作業状態

状態は、人間が対応すべきかどうかが分かる言葉にする。

- `処理中`
- `要対応・質問`
- `要対応・確認`
- `要対応・回復`
- `完了`

## デザイン引き継ぎ

- ブランド参照: `logo.png`。
- Hachiware Labs のテイストを残す。静かで実用的、白い面、slate系テキスト、
  indigo の primary action、細い境界線、コンパクトな密度、7-8px程度の角丸。
- 色は意味に使う。primary / selection、success、warning、danger、disabled。
- 派手なAIダッシュボード、マーケティングhero、装飾的なグラデーション、
  飾りのカード群にはしない。

Fable-5 再設計以前の wireframe は削除済み。機能フローと情報階層の参照は
`docs/design-assets/prototype/` と PRD に限定する。

## Fable-5 が読むアクティブ参照

- `docs/fable-5-ui-context.md`: この機能概要。
- `docs/design-assets/prototype/index.html`: 現在のクリック遷移プロトタイプ。
- `docs/design-assets/prototype/README.md`: 画面一覧・トークン・設計方針。
- `docs/nagare_prd_v1_0.md`: 機能IDと画面の対応。
- `docs/nagare_data_storage_spec_v1.md`: データルート、プロジェクト選択、テスト隔離、既存データ移行の仕様。
- `docs/design-assets/prototype/logo.png`: ブランドマーク参照。

## データ参照のみ

画面が正確なオブジェクト境界やフィールド名を必要とする時だけ参照する。

- `docs/agent_data_model.md`
- `docs/architecture.md`

## 初回UI再作成では archive 扱い

非UI、実装詳細、過去要件、詳細なビジュアルデザイン文脈は
`docs/archive/fable-5-non-ui-context.md` に整理する。
