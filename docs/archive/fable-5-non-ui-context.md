# Fable-5 非UI文脈 archive

この archive は、Nagare プロジェクトには有用だが、Fable-5 による初回の
UI再作成では主入力にしない情報を分けるための目録である。

既存リンクを壊さないため、元ファイルは移動しない。この文書を
Fable-5 向けの文脈選択境界として扱う。

## 初期状態では archive 扱い

### 詳細なビジュアルデザイン文脈

Fable-5 再設計以前の画面資産は削除済み。見た目や画面構造の根拠には使わない。

- `docs/design.md`
- `docs/create-new-item-ui-evaluation.md`

### 実装アーキテクチャ

backend連携、adapter挙動、正確なAPI契約を実装する時だけ読む。

- `docs/adapter_contract.md`
- `docs/agent_management.md`
- `docs/spec.md`
- `docs/plan.md`
- `docs/delta/*`

### 過去要件と prototype

プロダクト上の経緯を確認する時、または捨てた案を復元したい時だけ読む。

- `docs/nagare_requirements_v0_3.md`
- `docs/prototype_requirements_v0_4.md`
- `docs/prototype_screen_spec_v0_1.md`
- `docs/design-implementation-audit.md`
- `docs/workflow-state-design.html`
- `docs/domain-rubric-agent-design.html`

### 深い Agent / Skill 管理

Agent、Skill、MCP、runtime の設定画面を設計する時だけ読む。

- `docs/agent-management-usecases.md`
- `docs/agent_data_model.md` の Agent Profile / Skill 関連セクション
- `docs/spec.md` の Agent Management 関連セクション

### 配布と tutorial

package配布、CLI onboarding、公開ドキュメントを扱う時だけ読む。

- `README.md`
- `README_ja.md`
- `docs/tutorial.md`
- `docs/tutorial_ja.md`
- `packages/nagare/README.md`
- `packages/nagare/README_ja.md`

## archive しないもの

Fable-5 の初回UI再作成では、次の文書だけをアクティブ文脈として扱う。

- `docs/fable-5-ui-context.md`
- `docs/design-assets/prototype/index.html`
- `docs/design-assets/prototype/README.md`
- `docs/nagare_prd_v1_0.md`
- `docs/design-assets/prototype/logo.png`

## 限定的なデータ参照

新しいUIがオブジェクト境界やフィールド名を必要とする時だけ参照する。

- `docs/agent_data_model.md`
- `docs/architecture.md`

これらを視覚階層の入力にしない。UI階層は
`docs/design-assets/prototype/` と `docs/nagare_prd_v1_0.md` を優先する。
