# docs/archive

Fable-5 によるUIプロトタイプ再設計(2026-07)の設計判断を**反映していない**文書の置き場。
2026-07-05 に `docs/` 直下および `docs/design-assets/` から物理的に移動した。

## 現在アクティブな文書(docs/ 直下)

| 文書 | 役割 |
| --- | --- |
| `../design-assets/prototype/` | UIの正(クリック遷移プロトタイプ+カタログ) |
| `../nagare_prd_v1_0.md` | PRD+機能一覧。旧要件(v0.3/v0.4)のUX部分を置き換える |
| `../nagare_app_analysis_fable5.md` | 分析(使いやすさ/困りごと/差別化/評点) |
| `../nagare-product-notes-fable5.md` | プロダクト所感・発展性 |
| `../nagare_trace_schema_v1_0.md` | NF-2トレーススキーマ(判断記録の契約) |
| `../fable-5-ui-context.md` | Fable-5再設計の入口文書(経緯) |

## この archive の内訳

- **旧要件・旧仕様**: `nagare_requirements_v0_3.md`、`prototype_requirements_v0_4.md`、
  `prototype_screen_spec_v0_1.md`、`spec.md`、`plan.md`
  — UX要件はPRD v1.0が置き換え。バックエンドの実装状況表として参照する場合は
  PRDとの差分に注意(UI関連の記述は旧UI前提)。
- **旧設計・旧UI検討**: `design.md`、`design-implementation-audit.md`、
  `create-new-item-*.md`、`workflow-state-design.html`、`domain-rubric-agent-design.html`、
  `project-aware-control-plane-usecases.md`
- **実装アーキテクチャ(Fable-5入力外)**: `architecture.md`、`adapter_contract.md`、
  `agent_data_model.md`、`agent_management.md`、`agent-management-usecases.md`
  — バックエンド実装時の参照は可。UI階層の入力にはしない。
- **決定記録**: `delta/`(DR-*)
- **配布・チュートリアル(旧UI前提)**: `tutorial.md`、`tutorial_ja.md`
- **旧デザイン資産**: 削除済み。古い画面資産は現在のUI判断と矛盾しやすいため、
  archiveにも残さない。
- **Fable-5準備時の文脈境界メモ**: `fable-5-non-ui-context.md`(当時は「移動しない」方針
  だったが、本整理で物理移動に切り替えた)
