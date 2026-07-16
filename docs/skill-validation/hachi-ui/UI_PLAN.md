# UI Plan: Nagare — エージェントの Skill 割当を確認する

## Product Thesis

Nagare の管理者が、担当AIごとの有効Skillと継承元を一画面で照合し、意図しないグローバル継承を安全に見つけ、次に修正する割当を根拠付きで決められるようにする。

## Users And Roles

- ワークスペース管理者: 割当を確認し、個別Skillを追加・除外・継承解除する。
- 担当AI: 割当の対象。Skillや継承設定を直接入力しない。
- システム: グローバル既定、適用範囲、最終確認日時、影響する作業を auto-fill する。

## Core Objects

- 担当AI: 名前、担当、割当状態、次の対応優先度を持つ。
- Skill: Skill名、適用元（個別 / グローバル継承）、有効状態を持つ。
- 割当: 担当AIとSkillの関係。継承、除外、個別追加、要確認の状態を持つ。
- グローバル既定: 全担当AIへ継承されうるSkillと、その影響範囲。
- 修正候補: 警告、影響する作業、推奨操作、確認条件を持つ。

## User Decisions

1. どの担当AIに、どのSkillが現在有効か。
2. そのSkillは個別に許可されたか、グローバル設定から継承されたか。
3. 意図しない継承のうち、どれを先に修正すべきか。
4. 修正前に、影響範囲と安全な復帰方法を確認できるか。

## Screen Inventory

| Screen | Type | User Question | Primary Action | State | Pattern |
|---|---|---|---|---|---|
| Skill割当の確認 | desktop settings | 次に直すべき割当はどれか | 選択した担当AIの割当を修正 | グローバル継承の警告あり | Settings Dependency Map + Permission / Role Matrix |

## Flow Causality

| From | Trigger | To | Why This Follows |
|---|---|---|---|
| Skill割当の確認 | 「割当を修正」をクリック | 割当編集（次画面） | 選択した警告のある担当AIだけを安全に変更するため |
| Skill割当の確認 | 「グローバル設定を見る」をクリック | グローバル既定（次画面） | 継承元と影響範囲を比較してから方針を決めるため |
| 割当編集（次画面） | 保存時に競合を検出 | 競合の確認（回復） | 個別設定がポリシーと矛盾する場合に、保存前に復帰または理由入力を選ぶため |

## Screen Information Plan

### Skill割当の確認

- **Dominant information unit:** 担当AI別のSkill割当表。`user_intent: compare`、`shape_pattern: permission_scope_input`、`unit_pattern: permission_scope_row`、`area_need: large`。管理者は横方向のSkill列と適用元を比較し、警告がある行を選ぶ。
- **Priority recommendation:** 次に修正する割当。`user_intent: verify`、`shape_pattern: diagnostic_validation_view`、`unit_pattern: validation_state_block`、`area_need: medium`。選択中の担当AI、不要な継承、影響する作業、推奨修正を一つの判断面にまとめる。
- **Global inheritance warning:** `user_intent: view`、`shape_pattern: status_readiness_view`、`unit_pattern: validation_state_block`、`area_need: medium`。件数、継承元、影響人数、確認導線を上部に置く。
- **Selected assignment provenance:** `user_intent: verify`、`shape_pattern: evidence_provenance_view`、`unit_pattern: evidence_strip`、`area_need: small`。最終確認日時と適用元を表示し、詳細は折りたたむ。
- **Recovery state:** ポリシーにより変更不可の場合は「編集できません」と理由、管理者への依頼、戻る操作を表示する。空状態では「割当はありません」とグローバル設定確認への導線を表示する。

## Field Responsibility Matrix

| Screen | User Input | User Selection | Auto-filled | Review Only | Exception-only Input | Evidence / Display |
|---|---|---|---|---|---|---|
| Skill割当の確認 | なし | 担当AIの行、表示範囲 | 担当AI名、担当、Skill状態、継承元、影響作業、最終確認日時 | 有効Skill、優先度、推奨修正 | ポリシー例外を申請する理由（「例外を申請」後のみ） | グローバル既定、継承経路、警告根拠、変更履歴 |

Responsibility keywords: `user_selection`, `auto_filled`, `review_only`, `exception_only_input`, `evidence_display`.

## Input Friction Audit

| Information | Source | Handling | Why |
|---|---|---|---|
| 担当AI名と担当 | 既知の設定 | auto_filled の表行 | 管理者に再入力させない |
| 有効Skill | 解決済みの割当 | review-only のマトリクス | 個別と継承を比較して判断する情報 |
| 継承元 | グローバル既定 | 「グローバルから継承」表示と確認リンク | どこを直すかを判断できる |
| 修正対象 | 警告ルール | 優先度順に自動表示 | ユーザーは候補を選ぶだけ |
| 例外理由 | 例外申請時だけ必要 | 通常画面には出さず、次画面で入力 | ふだんの確認作業を重くしない |

## States And Recovery

- **Current state:** 2件のグローバル継承警告。資料調査AIが対象外の `browser` を継承している。
- **Ready:** 警告なしの担当AIは「問題なし」で表示し、主操作を要求しない。
- **Blocked:** ポリシーで固定されたSkillは編集不可。理由と「管理者に依頼」を表示する。
- **Conflict:** 個別除外がグローバル必須設定と競合した場合、保存を止め、グローバル設定確認または例外申請へ誘導する。
- **Recovery:** 編集を取り消す、グローバル設定を見る、または例外を申請する。

## Evidence And Provenance

- 各Skillセルは「個別」「グローバルから継承」「無効」を文字と線種で区別し、色だけに依存しない。
- 選択中の修正候補には、継承元、影響する作業、最終確認日時、推奨操作を表示する。
- 詳細な変更履歴やポリシー本文は初期表示せず、「詳細を開く」の後に置く。

## Area Budget

- 58%: 担当AI別のSkill割当表（比較と選択）。
- 24%: 選択中の修正候補（次の行動と根拠）。
- 10%: グローバル継承警告（状況把握）。
- 8%: ナビゲーション、フィルタ、補助操作。

## Gaze Route

1. 上部の「グローバル継承の警告 2件」で今のリスクを把握する。
2. 優先度順の割当表で警告のある担当AIを選ぶ。
3. 右側の「次に修正する割当」で影響と推奨操作を確認し、「割当を修正」を実行する。

## Pattern Selection And Audit

- **Selected:** Settings Dependency Map を主パターン、Permission / Role Matrix を比較単位として採用。設定の継承・影響・保存前の確認が中心であり、単なる一覧では判断できないため。
- **Quality score:** 91/100。ユーザーの判断、担当AI/Skill/割当のオブジェクト、継承/競合/固定の状態、長い設定ページを避ける反パターン、Nagare固有の「次に修正する割当」が明確。

## Rejected Patterns

- **Generic dashboard:** 警告件数だけでは、どの担当AIのどのSkillを直すか判断できない。
- **Bare toggle settings:** 有効/無効だけでは継承元・影響範囲・ポリシー競合を失う。
- **Two equal panes:** 表と詳細を同じ強さにすると、まず比較すべき割当表の優先度がぼやける。
