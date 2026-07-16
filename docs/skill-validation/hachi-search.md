# hachi-search の Skill 限定適用の検証

## 結論

この作業はローカル根拠の収集であり、適用 Skill は **hachi-search のみ**とする。`hachi-readable-writing` は文書の推敲・作成用、`hachi-ui` は UI 設計用であり、いずれも今回の検索作業の対象外である。Web 検索は実行していない。

## 根拠

1. **ローカル / `skills-lock.json:4-8`** — プロジェクトでロックされている Skill は `hachi-search` で、パスは `skills/hachi-search/SKILL.md` と記録されている。今回の検索対象に直接対応する唯一のロック済み Skill である。
2. **ローカル / `docs/nagare_agent_capability_scoping_design.md:105-111`** — 能力スコープの例は、エージェント割当の `skills` を `hachi-search` 一件（`source: project`, `assignment: agent`）としている。Skill を明示的に限定して実効能力を記録する設計を裏付ける。
3. **ローカル / `tests/e2e/desktop-ui.spec.ts:2332-2337`** — E2E テストは `hachiware-labs/hachi-search` パッケージの `provided_skill_sets` を `["hachi-search"]` として検証している。パッケージが hachi-search だけを提供する根拠である。

## 検索プロバイダー

- 使用: `local`（ワークスペース `C:\Users\naruhide\workspace\nagare`）。hachi-search の `local` プリセットで検索した。
- スキップ: `wiki-garden`（ローカル検索モードのため）、`tavily`、`exa`、`firecrawl`（ユーザー指示により Web 検索を禁止）。URL エンリッチメントも実行していない。

## 適用範囲

- 適用 Skill: `hachi-search` のみ。
- 対象外: `hachi-readable-writing`、`hachi-ui`。この作業にはローカル根拠の検索・出典記録だけが必要で、文章作成・UI 設計は含まれない。
