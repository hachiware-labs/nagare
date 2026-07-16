# Work Detail: 実行構成と禁止タスクレビュー UI 計画

## 対象の判断

依頼者が Work の詳細画面で、次の二点を確認して承認または差し戻しを判断できること。

1. 実行したエージェントに、必要最小限の Skill と MCP だけが有効だったか。
2. 明示した禁止タスクを Reviewer が確認し、未確認または違反の場合に通過していないか。

## 画面パターン

- 画面: `agent_execution_trace`（既存の Work Detail を拡張）
- 主情報単位: `object_header`、`permission_scope_row`、`evidence_detail_panel`、`validation_state_block`、`activity_timeline_step`
- 視線順: Work 状態と次アクション → 実行構成 → 結果とレビュー → 実行の流れと診断記録

## 実行構成ブロック

各実行を 1 行の監査可能な `permission_scope_row` として表示する。

| 情報 | 表示 | 役割 |
| --- | --- | --- |
| 工程 / 実行エージェント | 工程名と agent profile ID | 実行主体の確認 |
| Effective Skills | 有効な Skill ID のチップ。未設定は「割当なし」 | 意図しない全体 Skill の混入検出 |
| Effective MCP | agent profile に割り当てた接続 ID。未設定は「割当なし」 | 外部接続の確認 |
| Skill 境界 | Codex の許可数 / 無効化数とスコープ診断 | 実行時の厳格な allowlist の確認 |
| Skill 指示参照 | 折りたたみ詳細の SKILL.md パス | 必要時だけ証跡を追跡 |

生のログや絶対パスは最初の判断面を占有させず、折りたたみ詳細に置く。

## 禁止タスクのレビューゲート

- Work 作成時に任意の「禁止・制約」を 1 行 1 件で設定する。
- `禁止`、`[禁止]`、`forbidden:`、`prohibited:` を含む制約を禁止タスクルールと扱う。
- ルールがある Work の Reviewer には、ルールごとの確認と `禁止タスク` criterion の出力を要求する。
- `禁止タスク` criterion がない、または失敗なら、Review は自動的に `changes_requested` となり承認可能状態にしない。
- 詳細画面にはルール、確認済み / 要差し戻し / 未確認の状態、根拠を `validation_state_block` として表示する。

## 状態

- ルール未設定: 実行構成のみを表示。
- レビュー前: 「Reviewer 確認待ち」。
- 確認済み: 「禁止タスクを確認済み」。
- 未確認 / 違反: 「レビューを差し戻し」。承認操作は出さない。

## 受入条件

1. Work Detail の API が実行エージェント、Effective Skills、Effective MCP、Skill 境界を返す。
2. Work Detail が各実行の実効能力を表示する。
3. 禁止ルールがある Review は、ポリシー criterion を欠くと必ず `ChangesRequested` になる。
4. ポリシー criterion が失敗なら、`Pass` verdict を返しても承認可能にならない。
