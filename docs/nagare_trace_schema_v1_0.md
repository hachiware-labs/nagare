# Nagare トレーススキーマ v1.0(NF-2)

| 項目 | 内容 |
| --- | --- |
| 版 | v1.0(2026-07-05) |
| 位置づけ | PRD NF-2「トレースのデータ契約」の実装仕様。記録原則は PRD 5.10.1「判断を記録し、通信は記録しない」に従う |
| 消費者 | ワーク詳細(実行の流れ・結果・レビュー)、分析・改善(品質記録・提案生成・効果測定) |
| 受け入れ基準 | プロトタイプ(`design-assets/prototype/index.html`)のハーネス表示が、本スキーマのフィールドだけで再現できること(§8 対応表) |

## 1. 原則(要約)

- 記録するのは**判断**: 誰が・何を根拠に・どの知識で・何を作り・どう評価し・人間が何を決めたか。
- 記録しないのは**通信**: 実行環境内部のAPIやりとり(全プロンプト・全レスポンス・トークンspan)。
- 深掘りは**診断ポインタ**(§6)で実行環境側のセッションログへ委譲する。

## 2. 保存場所と形式

```
.nagare/works/<work_id>/
  work.toml        … 現在状態(状態機械)。本仕様の対象外
  trace.jsonl      … 判断記録。本仕様の対象
```

- **JSONL・追記専用**。1行=1レコード。工程や判断が**確定した時点**で追記する(実行中の中間状態は書かない)。
- レコードは削除・書き換えしない(監査線)。訂正が必要な場合も新レコードの追記で表現する。
- 文字コードUTF-8。1ワークあたり想定サイズは数KB〜数十KB。

## 3. 共通封筒(全レコード)

```json
{
  "schema": "nagare.trace/1.0",
  "record": "worker_output",          // レコード種別(§4, §5)
  "work_id": "W-1024",
  "seq": 3,                            // ワーク内の追記順(1始まり・欠番なし)
  "at": "2026-07-05T10:12:34+09:00",  // 記録時刻
  "payload": { ... }                   // 種別ごとの本体
}
```

工程系レコード(§4)の payload には、さらに次の共通フィールドを含める。

| フィールド | 型 | 必須 | 内容 |
| --- | --- | --- | --- |
| `step_no` | int | ✓ | 工程番号(1始まり) |
| `step_kind` | enum | ✓ | `intake` / `create` / `review` |
| `agent` | obj | ✓ | `{ id, name, role: organizer\|worker\|reviewer, builtin: bool }` |
| `runtime` | obj | ✓ | `{ id, model }`(モデル切替非対応環境は `model: "runtime-default"`) |
| `duration_ms` | int | ✓ | 所要時間 |
| `status` | enum | ✓ | `completed` / `failed` |
| `knowledge_refs` | array | ✓ | 注入した知識 `[{ id, version }]`(空配列可) |
| `diagnostics` | obj | ✓ | 診断ポインタ(§6) |

## 4. 3つの原型(不変の核)

> オーガナイザーは「どんな根拠で・何に作業させる判断をしたか」。
> ワーカーは「どんな入力で・何を作り・何を回答したか」。
> レビュアーは「何を出力し・どのような評価を下したか」。
> この3型は**原型であり変わらない**(互換性ポリシーは§7)。

### 4.1 `organizer_decision` — 根拠と割り当ての判断

| フィールド | 型 | 必須 | 内容 |
| --- | --- | --- | --- |
| `interpreted_request` | str | ✓ | 依頼をどう解釈したか(1〜2文) |
| `domain_id` / `artifact_type_id` | str | ✓ | 割り当てたドメイン / 成果物。`artifact_type_id`は内部互換名 |
| `plan` | array | ✓ | 実行計画 `[{ step_no, step_kind, agent_id }]` |
| `assignments` | array | ✓ | `[{ step_no, agent_id, rationale }]` — **rationale は必須**(UIの「割り当ての根拠」) |
| `candidates_considered` | array | 任意 | 比較した候補 `[{ agent_id, reason_rejected }]` |

```json
{ "schema": "nagare.trace/1.0", "record": "organizer_decision", "work_id": "W-1024", "seq": 2,
  "at": "2026-07-05T10:01:10+09:00",
  "payload": {
    "step_no": 1, "step_kind": "intake",
    "agent": { "id": "builtin-organizer", "name": "オーガナイザー", "role": "organizer", "builtin": true },
    "runtime": { "id": "claude-code", "model": "runtime-default" },
    "duration_ms": 8200, "status": "completed",
    "knowledge_refs": [ { "id": "kn-product-common", "version": 3 }, { "id": "kn-relnote-template", "version": 2 } ],
    "diagnostics": { "runtime": "claude-code", "session_ref": "s_8f2c31" },
    "interpreted_request": "依頼を「文書作成」と判定。ドメイン「プロダクト文書」/ 成果物「リリースノート」。",
    "domain_id": "dom-product-docs", "artifact_type_id": "at-release-note",
    "plan": [ { "step_no": 2, "step_kind": "create", "agent_id": "ag-writer" },
              { "step_no": 3, "step_kind": "review", "agent_id": "ag-reviewer" } ],
    "assignments": [ { "step_no": 2, "agent_id": "ag-writer",
                       "rationale": "得意分野「ドキュメント作成」がドメイン「プロダクト文書」に一致" } ]
  } }
```

### 4.2 `worker_output` — 入力・成果・回答

| フィールド | 型 | 必須 | 内容 |
| --- | --- | --- | --- |
| `inputs` | obj | ✓ | `{ summary, refs: [...] }` — 何を受け取ったかの要約と参照(構造化依頼・知識・先行成果物・ユーザー回答) |
| `actions_summary` | str | ✓ | 何をしたかの要約(1〜3文。UIの「実行内容」) |
| `artifacts` | array | ✓ | `[{ path, change: new\|modified\|deleted, lines }]` — **具体的なファイル名**(空配列可) |
| `answer` | str | ✓ | 回答文(UIの「回答」。成果がファイルのみでも1文書く) |
| `question` | obj | 任意 | 停止した質問 `{ id, text, options: [...] }`。回答は `human_decision` が持つ |

```json
{ "schema": "nagare.trace/1.0", "record": "worker_output", "work_id": "W-1024", "seq": 3,
  "at": "2026-07-05T10:09:02+09:00",
  "payload": {
    "step_no": 2, "step_kind": "create",
    "agent": { "id": "ag-writer", "name": "Writer", "role": "worker", "builtin": false },
    "runtime": { "id": "claude-code", "model": "fable-5" },
    "duration_ms": 412000, "status": "completed",
    "knowledge_refs": [ { "id": "kn-relnote-template", "version": 2 }, { "id": "kn-glossary", "version": 5 } ],
    "diagnostics": { "runtime": "claude-code", "session_ref": "s_8f2c31" },
    "inputs": { "summary": "構造化依頼、知識2件、コミット履歴(GitHub MCP)",
                "refs": [ "organizer_decision#seq2" ] },
    "actions_summary": "テンプレート構成に沿ってリリースノート下書きを作成。コミット12件を変更点に対応付け。",
    "artifacts": [ { "path": "docs/release-notes/v0.4.md", "change": "new", "lines": 214 } ],
    "answer": "リリースノート v0.4 の下書きを作成しました。新機能5件・修正7件・破壊的変更1件を記載し、テンプレートの構成に合わせています。内部用語が2箇所残っているため、確認をお願いします。"
  } }
```

### 4.3 `reviewer_verdict` — 出力と評価

| フィールド | 型 | 必須 | 内容 |
| --- | --- | --- | --- |
| `rubric_ref` | obj | ✓ | `{ id, version }` — どの基準のどの版で評価したか |
| `target_artifacts` | array | ✓ | 評価対象(worker の artifacts への参照) |
| `item_verdicts` | array | ✓ | `[{ item, max_points, points, verdict: pass\|concern, evidence, concern_note? }]` — **全項目**を記録(合格項目にも evidence 必須。UIの全項目判定表) |
| `total_score` / `max_score` | int | ✓ | 評価項目の達成数。`item_verdicts` から再計算可能であること。総合得点には使用しない |
| `overall_score` / `overall_max_score` | int | ✓ | レビュアーが付けた総合得点。`overall_score` は0-100、`overall_max_score` は100 |
| `recommendation` | enum | ✓ | `approve` / `revise` |
| `summary` | str | ✓ | 判定の要約1文(UIの「承認を推奨 — 懸念1件」) |

```json
{ "schema": "nagare.trace/1.0", "record": "reviewer_verdict", "work_id": "W-1024", "seq": 4,
  "at": "2026-07-05T10:11:40+09:00",
  "payload": {
    "step_no": 3, "step_kind": "review",
    "agent": { "id": "ag-reviewer", "name": "Reviewer", "role": "reviewer", "builtin": false },
    "runtime": { "id": "claude-code", "model": "fable-5" },
    "duration_ms": 96000, "status": "completed",
    "knowledge_refs": [ { "id": "kn-relnote-template", "version": 2 } ],
    "diagnostics": { "runtime": "claude-code", "session_ref": "s_8f2c31" },
    "rubric_ref": { "id": "rb-relnote-quality", "version": 2 },
    "target_artifacts": [ "docs/release-notes/v0.4.md" ],
    "item_verdicts": [
      { "item": "変更点の網羅性", "max_points": 20, "points": 20, "verdict": "pass",
        "evidence": "v0.3以降のコミット12件すべてに対応する記載を確認" },
      { "item": "読者向けの言い換え", "max_points": 20, "points": 8, "verdict": "concern",
        "evidence": "内部用語が2箇所(「dispatch hint」「AgentRun」)",
        "concern_note": "共通知識「プロダクト用語集」に定訳があるため置き換えを推奨" }
    ],
    "total_score": 5, "max_score": 6,
    "overall_score": 88, "overall_max_score": 100,
    "recommendation": "approve",
    "summary": "承認を推奨 — 懸念1件(読者向けの言い換え)"
  } }
```

## 5. 補助レコード

### 5.1 `work_header`(seq=1・必ず先頭)

`{ request_text, project_id, confirmation_policy, created_at }`

### 5.2 `human_decision` — 人間の判断

| フィールド | 型 | 必須 | 内容 |
| --- | --- | --- | --- |
| `kind` | enum | ✓ | `answer` / `approve` / `reject` / `recovery_choice` |
| `refs` | array | ✓ | 対象への参照(`worker_output#seq3` の question 等) |
| `content` | obj | ✓ | kind別: answer=`{ selected, note }` / approve=`{}` / reject=`{ comment, cited_concerns: [...] }` / recovery_choice=`{ option }` |

差し戻し(`reject`)の `comment` と `cited_concerns` は次の実行のワーカー入力(`inputs.refs`)として参照される。
これが「判断の資産化」(PRD AN-12)の一次データになる。

### 5.3 `recovery_event` — 失敗と引き継ぎ

`{ step_no, cause, impact, handoff: { completed: [...], pending: [...] }, diagnostics }`
回復方法の選択は `human_decision(kind=recovery_choice)` が持つ。再開後の工程は新しい工程レコードとして追記。

## 6. 診断ポインタ

```json
"diagnostics": { "runtime": "claude-code", "session_ref": "s_8f2c31", "hint": "claude --resume s_8f2c31" }
```

- `session_ref` は実行環境側のセッション識別子。**生ログ本体はNagareに保存しない**。
- `hint` は任意(ログを開く手段の提示)。保持期間・保護は実行環境側の責務。

## 7. 互換性ポリシー(原型は不変)

- §4の3レコード種別と、その必須フィールドは**削除・改名しない**。変更は任意フィールドの追加のみ
  (additive-only)。追加時はマイナー版を上げる(`nagare.trace/1.1`)。
- 読み手は未知のフィールド・未知のレコード種別を無視できなければならない(forward compatible)。
- 破壊的変更が避けられない場合のみメジャー版を上げ、旧版の読み込みを維持する。

## 8. UI対応表(受け入れ基準)

| ハーネスUIの表示 | スキーマ上の出所 |
| --- | --- |
| 工程行「担当 / 状態」 | 共通封筒 `agent` / `status` |
| 「割り当ての根拠」 | `organizer_decision.assignments[].rationale` |
| 「使用した知識」チップ | `knowledge_refs`(id→名前解決はナレッジ側) |
| 「入力 / 実行中(実行内容) / 出力」 | `worker_output.inputs.summary` / `actions_summary` / `artifacts` |
| 結果セクションの「回答」 | `worker_output.answer` |
| 「できたもの」(ファイル名・行数) | `worker_output.artifacts[]` |
| レビューの全項目判定表(✓/!・根拠・得点) | `reviewer_verdict.item_verdicts[]` |
| 「88 / 100 · 承認を推奨 — 懸念1件」 | `overall_score` / `recommendation` / `summary` |
| 質問画面の質問・選択肢 | `worker_output.question` |
| 回答・承認・差し戻し・回復選択の記録 | `human_decision` |
| 回復画面の「原因 / 影響 / 引き継ぎ」 | `recovery_event` |
| 「実行の詳細ログを開く(診断)」 | `diagnostics.session_ref` |
| 分析・改善のKPI / 失点マトリクス / 品質記録 | 本ファイル群からの**派生**(§9) |

## 9. 品質記録との関係

分析・改善が使う品質記録(懸念の反復、項目獲得率、差し戻し率、効果測定)は、
すべて `trace.jsonl` 群からの**派生集計**であり、独立した一次データを持たない。
集計キャッシュを持つ場合も、trace から常に再構築可能であること。
