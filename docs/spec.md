# spec.md（機能仕様：最新版）

この文書は Nagare / 流 の機能仕様の正本である。背景と上位要件は
`docs/nagare_requirements_v0_3.md`、実装設計は `docs/architecture.md`、
Agent Profile / Skill のデータ形式は `docs/agent_data_model.md` を参照する。

仕様 ID は機能ごとに `大分類.機能.仕様` の三階層連番で管理する。
例: `3.2.1` は「Agent Profile 管理」内の「表示」機能の 1 番目の仕様を示す。
四階層目は作らない。詳細なエラーやメッセージは `ERR-*` / `MSG-*` で別管理する。

実装状態:

- `実装済み`: 現在の CLI / core で動く。
- `進行中`: データモデルまたは設計は固定済みで、実装対象。
- `計画`: MVP 後続または設計保持。

## 1. Project / Locale

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 1.1.1 | Project を初期化すると、`.nagare` 配下に設定、台帳、Domain Group / Domain / Agent / artifact / log 保存先を作成する。 | project root が存在する | `nagare init` を実行する | `.nagare/project.toml`、`.nagare/domain-groups/general.toml`、`.nagare/domains/general.toml`、`.nagare/agents/`、`.nagare/state/ledger.json`、`.nagare/artifacts/`、`.nagare/logs/` が存在する | 実装済み |
| 1.1.2 | 既存の Nagare Project を再初期化しても、既存の設定と台帳を破壊しない。 | `.nagare` が存在する | `nagare init` を再実行する | 既存ファイルが維持され、CLI は成功する | 実装済み |
| 1.2.1 | Project locale を設定できる。 | Project が初期化済み | `nagare locale use --language <locale> --timezone <timezone>` を実行する | `.nagare/project.toml` の `[locale]` が更新される | 実装済み |
| 1.2.2 | Project locale を確認できる。 | Project が初期化済み | `nagare locale show` を実行する | language と timezone が表示される | 実装済み |
| 1.2.3 | Nagare が生成する記録には locale を保存する。 | Project locale が設定済み | Work Item、Run、Evidence、Review、Decision、Probe を作成する | 各 ledger record に `locale` が保存される | 実装済み |
| 1.2.4 | timezone は Project 設定として保存し、日時表示の locale 対応に利用できる設計にする。 | Project locale が設定済み | 時刻を表示または記録する | timezone を参照できる | 進行中 |
| 1.2.5 | Project 初期化時の locale は起動環境の locale を既定にする。 | `NAGARE_LOCALE` / `LC_*` / `LANG` またはOS localeが設定されている | `nagare init` を実行する | `.nagare/project.toml` の `[locale].language` が環境localeから初期化される。明示設定済みProjectではProject localeを優先する | 実装済み |
| 1.2.6 | UI文言、初期Domain/Agent説明、Agent prompt補助文は同じi18n層から切り替える。 | Project locale が `ja-JP` または `en-US` | UI表示、Project初期化、Agent実行prompt生成を行う | 固定UI文言、default domain context、default agent instruction、output contract instruction が locale に応じて日本語/英語で生成される | 実装済み |

## 2. Work Item

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 2.1.1 | Work Item を作成できる。 | Project が初期化済み | `nagare item create --title <title>` を実行する | `work_0001` 形式の ID を持つ Work Item が ledger に保存される | 実装済み |
| 2.1.2 | Work Item 作成時に description を保存できる。 | Project が初期化済み | `--description <text>` を付けて作成する | Work Item snapshot に description が残る | 実装済み |
| 2.1.3 | Work Item 作成時の初期 status は `ready` とする。 | Project が初期化済み | Work Item を作成する | status が `ready` になる | 実装済み |
| 2.1.4 | Work Item に acceptance criteria を保存できる。 | Project が初期化済み | `nagare item create --acceptance <csv>` を実行する | Work Item snapshot と Run Packet goal に criteria が反映される | 実装済み |
| 2.1.5 | Work Item に expected artifacts を保存できる。 | Project が初期化済み | `nagare item create --artifact <csv>` を実行する | Recovery と Review が期待成果物不足を判断できる | 実装済み |
| 2.1.6 | Work Item の検査観点は Review で扱う。 | Project が初期化済み | Work Item を作成する | 独立した検査ヒント は新規UI/CLI導線に出さず、CI/test/artifact check は Review の `completed` / `findings` / `criteria_results` に残る | 実装済み |
| 2.1.7 | Work Item に work folder を保存できる。 | Project が初期化済み | `nagare item create --work-folder <relative_path>` を実行する | Run Packet の path / work_folder に反映される | 実装済み |
| 2.1.8 | Work Item に constraints を保存できる。 | Project が初期化済み | `nagare item create --constraint <csv>` を実行する | Agent prompt の制約文脈に反映される | 実装済み |
| 2.1.9 | Work Item に workflow mode を保存できる。 | Project が初期化済み | `nagare item create --workflow-mode confirm_first\|finish_first` を実行する | Work Item snapshot に `workflow_mode` が保存され、未指定時は Domain override、Domain Group default、Project workflow default の順に解決される | 実装済み |
| 2.1.10 | Work Item に Domain Group、Domain、最終承認ポリシーを固定できる。 | Project が初期化済み | `nagare item create --domain-group <group_id> --domain <domain_id> --approval-policy manual_final_approval\|auto_complete_on_review_pass` または Settings の Work Item 作成フォームを使う | Work Item snapshot に `domain_group_id` / `domain_id` と解決済み `approval_policy` が保存され、Domain が Group に属する場合は矛盾する Group 指定を拒否する | 実装済み |
| 2.2.1 | Work Item 一覧を表示できる。 | Work Item が存在する | `nagare item list` を実行する | ID、status、title が確認できる | 実装済み |
| 2.2.2 | Work Item 詳細を表示できる。 | Work Item が存在する | `nagare item show <work_id>` を実行する | Work Item、runs、artifacts、evidence、review、handoffs、decisions が確認できる | 実装済み |
| 2.2.3 | 存在しない Work Item は拒否する。 | 指定 ID が ledger に存在しない | Work Item 対象 command を実行する | エラーになり、ledger は変更されない | 実装済み |

## 3. Agent Profile 管理

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 3.1.1 | Project-local Agent Profile を登録できる。 | Project が初期化済み | `nagare agent add --id <id> --runtime <runtime> --adapter <adapter>` を実行する | `.nagare/agents/<id>.toml` が作成される | 実装済み |
| 3.1.2 | Agent Profile には表示名、role、runtime、adapter、working_dir、description、specialties、担当 Domain Group / Domain を保存できる。 | Project が初期化済み | `nagare agent add` に各 option を渡す、または Settings の Agent form を送信する | 保存された TOML に値が残り、dispatch prompt の候補 summary に反映される | 実装済み |
| 3.1.3 | Agent Profile の `working_dir` は Project 内の相対 path に限定する。 | Project が初期化済み | workspace 外または絶対 path を指定する | 登録または実行が拒否される | 実装済み |
| 3.1.4 | Agent Profile の dispatch routing hint を後から更新できる。 | Agent Profile が存在する | `nagare agent update <id> --description ... --specialties ...` を実行する | `.nagare/agents/<id>.toml` が更新され、Agent Profile 詳細に反映される | 実装済み |
| 3.1.5 | Project-local Domain Profile を登録できる。 | Project が初期化済み | Settings の Domain 作成フォームを送信する | `.nagare/domains/<id>.toml` が作成され、description、artifact_types、rubric、dispatch_hints、workflow override が保存される | 実装済み |
| 3.1.6 | Domain Profile を後から更新できる。 | Domain Profile が存在する | Settings の Domain 編集フォームを送信する | `.nagare/domains/<id>.toml` が更新され、Settings の Domains 一覧に反映される | 実装済み |
| 3.1.7 | Domain Group は大きな業務領域を束ねる第一級概念として扱う。 | 複数Domainが同じ業務領域に属する | Settings の Domain Group 作成フォームを送信する | `.nagare/domain-groups/<id>.toml` に共通知識、共通Rubric、dispatch_hints、workflow default が保存され、Domain は `group_id` で所属を持てる | 実装済み |
| 3.1.8 | Project-local Agent Profile を削除できる。 | Settings から作成した Agent Profile が存在する | Agent 編集画面の Delete Agent を実行する | `.nagare/agents/<id>.toml` が削除され、Project config 由来の既定Agentは削除不可になる | 実装済み |
| 3.2.1 | 登録済み Agent Profile を一覧表示できる。 | Agent Profile が存在する | `nagare agent list` を実行する | profile ID、adapter、runtime、role が確認できる | 実装済み |
| 3.2.2 | 登録済み Agent Profile の詳細を表示できる。 | Agent Profile が存在する | `nagare agent show <agent_profile_id>` を実行する | display_name、adapter、runtime、working_dir が確認できる | 実装済み |
| 3.2.3 | 未知の Agent Profile を実行対象に指定した場合は拒否する。 | 指定 ID が存在しない | `item run --agent <id>` を実行する | エラーになり、Agent Run は作成されない | 実装済み |
| 3.2.4 | Settings の Agent 一覧を Domain Group / Domain で絞り込める。 | Domain Group / Domain と Agent Profile が存在する | Settings の Agents で Domain Group または Domain の checkbox を選ぶ | 選択した Domain Group または Domain に属する Agent Profile だけが表示され、Clear filters で全件表示に戻る | 実装済み |
| 3.2.5 | Agent Profile は tool kind を持ち、Codex / Codex CLI / OpenClaw を横断して管理できる。 | Agent Profile を作成または読み込む | `runtime` / `adapter` が指定される | `tool_kind = codex\|codex_cli\|openclaw` が保存される。既存Profileで未指定の場合は runtime / adapter から推定され、矛盾する組み合わせは拒否される | 実装済み |
| 3.2.6 | Agent Profile は tool内の model provider / model / base_url を保存できる。 | Agent Profile を作成または更新する | `--model-provider`、`--model`、`--base-url` を指定する、または Settings Agent form を送信する | Codex / Codex CLI は OpenAI系 provider のみ許可し、OpenClaw の Ollama / LM Studio provider は base_url 未指定を拒否する | 実装済み |
| 3.2.7 | Agent Profile は個別の skill set を持てる。 | Project config に `[skill_sets.<id>]` が定義されている | `nagare agent add/update --skills <ids>` または Settings Agent form で skill を選択する | `.nagare/agents/<id>.toml` に `skill_set_ids` が保存され、未定義の skill set id は拒否される | 実装済み |
| 3.2.8 | Agent Profile は Prompt instructions を保存でき、既存 description と互換性を持つ。 | Agent Profile を作成または読み込む | Profile に `prompt.instructions` がある、または既存 `description` だけがある | 実行時は `prompt.instructions` を優先し、空の場合だけ `description` を instructions として使う。`description` は表示用 summary / dispatch hint として残る | 実装済み |
| 3.2.9 | Settings の Agent form から installed skill set を検索・選択できる。 | Project config に skill set が定義されている | Agent 作成または編集画面の Skills section で skill id / path / capability を検索し、checkbox で選択して保存する | 選択済み skill が chip で表示され、Agent Profile に `skill_set_ids` が保存され、Agent 一覧の Skills 列にも chip として表示される | 実装済み |
| 3.2.10 | Settings の Agent 一覧は Agent を中心に Tool / Model / Skills / Scope / Actions を確認できる。 | Agent Profile が存在する | Settings の Agents を開く | Agent 名、role、working_dir、tool kind、model、source、skills、domain scope が横断的に確認でき、モバイルでは card-like row として読める | 実装済み |
| 3.2.11 | Settings の Agent 管理 UI は基本的なアクセシビリティ状態を持つ。 | Settings を開く | keyboard / screen reader で Settings tabs と Agent form を操作する | tablist は `aria-controls` / `tabpanel` / `aria-selected` を持ち、focus-visible、hover、status live region、ロゴ寸法属性が設定されている | 実装済み |
| 3.2.12 | Skill package を source provenance 付きでProjectに登録できる。 | Project が初期化済み | `nagare skill add --from local\|git\|clawhub\|vercel\|skill-creator ...` を実行する | `.nagare/project.toml` に `[skill_packages.<id>]` と `[skill_sets.<id>]` が追加され、source_kind / source / ref / checksum / provided_skill_sets が記録される | 実装済み |
| 3.2.13 | skill-creator 形式の skill folder から skill id を推定できる。 | `SKILL.md` frontmatter に `name` がある | `nagare skill add --from skill-creator --path <skill-folder>` を実行する | `name` を package id / skill set id として使い、required / optional capability と path を登録できる | 実装済み |
| 3.2.14 | Settings UI から Skill package 登録画面へ遷移できる。 | Settings の Agent 作成/編集画面を開いている | Skills section の Add Skill を押し、source / path / ref / checksum / capability を入力して保存する | `/api/skills` が skill package と skill set を登録し、Agent 作成画面に戻る | 実装済み |

## 3A. Workflow Policy

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 3A.1.1 | Project 既定の進行方針を設定できる。 | Project が初期化済み | Settings の Workflow で `default_progress_mode` を保存する | `.nagare/project.toml` の `[workflow].default_progress_mode` が更新され、新規 Work Item の既定値になる | 実装済み |
| 3A.1.2 | Project 既定の最終承認ポリシーを設定できる。 | Project が初期化済み | Settings の Workflow で `approval_policy` を保存する | `manual_final_approval` では人間の approve で止まり、`auto_complete_on_review_pass` では review pass 後の advance で done まで進む | 実装済み |
| 3A.1.3 | Domain は Project workflow を上書きできる。 | Domain Profile が存在する | Domain 編集フォームで workflow override を保存する | Work Item 作成時に domain を選ぶと、Domain override が Project default より優先される | 実装済み |

## 4. Agent Selection / Dispatch 準備

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 4.1.1 | Agent Profile、Domain Group、Domain Profile は選択材料を持つ。 | Agent / Domain Group / Domain Profile を登録する | Agent の role、specialties、description、working_dir、domain scope と、Group / Domain の rubric、dispatch_hints を設定する | dispatch が候補比較に使える情報が `.nagare/agents/<id>.toml`、`.nagare/domain-groups/<id>.toml`、`.nagare/domains/<id>.toml` に残る | 実装済み |
| 4.1.1a | Domain Group はdispatchの選択材料として使う。 | Domain Group が定義されている | dispatch context を組み立てる | Group の共通知識、共通Rubric、dispatch_hints、workflow default が Domain Profile と合わせて候補比較に使える | 実装済み |
| 4.1.2 | Work / Review の担当は固定Defaultではなく dispatch が選ぶ。 | 複数の Agent Profile が存在する | Work Item を進める | dispatch が purpose、role、specialties、description、working_dir、履歴を見て target Agent Profile を選ぶ | 実装済み |
| 4.1.3 | fallback Agent は内部設定として扱い、Settings UI の主機能にしない。 | dispatch が選べない、または明示Agentが省略される | `item run` / `item review` / `item preview` を実行する | 既存の fallback 設定を使えるが、ユーザー向け Settings には Agent Defaults form を出さない | 実装済み |
| 4.1.4 | Nagare のワークフロー判断に使う supervisor_agent は高度なfallback設定として扱う。 | Agent Profile が存在する | `nagare agent use --supervisor-agent <id>` を実行する | `.nagare/project.toml` の `[nagare_agents].supervisor_agent` が更新される | 実装済み |
| 4.1.5 | Domain Group / Domain / Rubric / Agent の関係をHTML設計文書で確認できる。 | 設計文書を読む | `docs/domain-rubric-agent-design.html` を開く | Domain Group、Domain Profile、Rubric、Agent Profile、Workflow Policy、Dispatch / Review の関係図と責務表が確認できる | 実装済み |
| 4.2.1 | dispatch_agent は Work Item の実行前確認に使う。 | dispatch_agent が設定済み | `nagare item preview <work_id>` を実行する | dispatch_agent の AgentRun が `dispatch_preview` として記録される | 実装済み |
| 4.2.2 | dispatch は Work Item の実作業を進めない。 | Work Item が存在する | Preview を実行する | AgentRun と Evidence は残るが、Work Item status は実行結果で進まない | 実装済み |
| 4.2.3 | review_agent は実行後の評価に使う。 | review_agent が設定済み | `nagare item review <work_id>` を実行する | review_agent の AgentRun が `review` として記録される | 実装済み |
| 4.2.4 | dispatch preview の結果は Dispatch Plan として保存する。 | dispatch preview が成功する | Preview または Handoff Dispatch を実行する | DispatchPlan が AgentRun、ResolvedRunPacket、Artifact と紐づいて ledger に保存される | 実装済み |
| 4.2.5 | dispatch_agent には小さな候補 Agent Profile リストだけを渡す。 | dispatch preview を開始する | `nagare item preview` を実行する | 依頼時の work_folder、Agent Profile の role / specialties / description / working_dir から最大 5 件の候補 summary が prompt に含まれる | 実装済み |
| 4.2.6 | dispatch_agent は候補リストから target Agent Profile を選べる。 | dispatch_agent が JSON を返す | `target_agent_profile_id` を含む dispatch output を保存する | 存在する Agent Profile なら DispatchPlan.target_agent_profile_id に採用され、不正 ID は fallback target になる | 実装済み |
| 4.2.7 | DispatchPlan は `draft`、`accepted`、`superseded` の lifecycle を持つ。 | DispatchPlan が存在する | Preview または accept を実行する | 新しい preview は古い draft を superseded にし、accept は選択 plan を accepted にする | 実装済み |
| 4.2.8 | DispatchPlan を実行前に採用できる。 | draft DispatchPlan が存在する | `nagare item dispatch accept <work_id>` を実行する | 対象 plan が accepted になり、同じ Work Item の他 plan は superseded になる | 実装済み |
| 4.2.9 | dispatch output contract 違反は fallback として記録する。 | dispatch_agent が JSON なし、target 未指定、または未知 target を返す | dispatch preview を保存する | fallback target が使われ、DispatchPlan.selection_warnings に理由が残る | 実装済み |
| 4.2.10 | Agent Profile は domain group × domain × role で分化できる。 | `code-planner`、`docs-worker`、`ui-reviewer` のような Agent Profile が登録されている | dispatch preview を実行する | 候補 summary に `role`、`specialties`、`description`、`domain_group_ids`、`domain_ids` が含まれ、dispatch_agent が Planner / Worker / Reviewer を domain に応じて選べる | 実装済み |
| 4.2.11 | 初期 Agent Profile は汎用 Domain に紐づく。 | Project が初期化済み | `nagare agent list` または Settings の Agent 一覧を見る | `worker`、`reviewer`、`dispatcher`、`supervisor` が `domain_group_ids = ["general"]`、`domain_ids = ["general"]` を持つ | 実装済み |
| 4.2.12 | Domain 指定 Work Item は、Domain Agent が見つからない場合の扱いを作成時に3択で固定できる。 | Work Item 作成時に `--domain-agent-policy auto_general_fallback\|confirm_general_fallback\|require_domain_agent` を指定する | dispatch preview を実行する | `auto_general_fallback` は `general` scope の fallback Agent で確認なしに進む。`confirm_general_fallback` は general で進めるか確認し、Work Item を `needs_input` にする。`require_domain_agent` は対応 Agent 追加または policy 変更を求め、Work Item を `needs_input` にする | 実装済み |
| 4.2.13 | CLI と UI から Agent Profile の role を設定できる。 | Agent Profile を作成または編集する | `nagare agent add/update --role <role>` または Settings の Role select を送信する | `.nagare/agents/<id>.toml` に role が保存され、候補表示と dispatch prompt に反映される。UI は planner / worker / reviewer / dispatcher / supervisor / implementer を選択肢として表示する | 実装済み |

## 5. Agent Health / Capability Probe

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 5.1.1 | Agent Profile の runtime が利用可能か確認できる。 | Agent Profile が存在する | `nagare agent doctor <agent_profile_id>` を実行する | runtime command の存在と healthcheck 結果が表示される | 実装済み |
| 5.1.2 | Agent Profile の capability snapshot を取得できる。 | Agent Profile が存在する | `nagare agent probe <agent_profile_id>` を実行する | CapabilityProbe が ledger に保存される | 実装済み |
| 5.1.3 | CapabilityProbe は runtime、adapter、利用可否、発見 capability、instruction source、locale を保存する。 | Probe が実行される | Probe が完了する | 後続の解決処理で参照できる snapshot が残る | 実装済み |
| 5.1.4 | Run / Preview 前に CapabilityProbe を自動更新する。 | Agent Run を開始する | Probe が未取得、古い、runtime / adapter / runtime_version が一致しない | 新しい CapabilityProbe が ledger に保存され、その probe が ResolvedSkillContext に紐づく | 実装済み |
| 5.2.1 | Agent候補の適合性は Agent Profile の属性、working_dir、Probe 結果、adapter capability を使って判断する。 | Agent Profile と CapabilityProbe が存在する | Run Packet を解決する | work_folder との一致、capability、instruction source が候補 summary と constraints に残る | 実装済み |

## 6. Agent Scope / Run Packet

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 6.1.1 | 依頼時に work_folder を指定できる。 | Work Item が存在する | `nagare item preview <work_id> --work-folder <path>` を実行する | work_folder が preview / dispatch context に含まれる | 予定 |
| 6.1.2 | Agent Profile は working_dir を必須属性として扱う。 | Agent Profile を登録または更新する | `nagare agent add/update --working-dir <path>` を実行する | working_dir が project root 相対 path として保存される | 実装済み |
| 6.2.1 | ScopeResolution は work_folder と Agent Profile working_dir の関係を示す。 | work_folder と Agent Profile が存在する | Preview / Run Packet を解決する | exact / parent / child / mismatch などの scope match が表示される | 予定 |
| 6.2.2 | `item preview` は dispatch_agent で実行前確認を記録する。 | Work Item と dispatch_agent が存在する | `nagare item preview <work_id>` を実行する | `dispatch_preview` 目的の AgentRun、ExecutionRecord、Evidence、DispatchPlan が保存される | 実装済み |
| 6.2.3 | `item preview` は work_folder、Agent Profile、Policy、Review観点を解決して表示する。 | Work Item と Agent Profile が存在する | `nagare item preview <work_id> --work-folder <path>` を実行する | Agent Profile、working_dir、scope match、Policy、Review観点が表示され、dispatch prompt に含まれる | 予定 |
| 6.2.4 | dispatch prompt は Agent instruction source の全文を含めない。 | Agent Profile と Probe が存在する | dispatch preview prompt を生成する | 候補 context は profile summary に限定され、大きな AGENTS.md / SOUL.md などは直接展開しない | 実装済み |
| 6.3.1 | Resolved Skill Context は実行時に使った Capability、Instruction source を固定する。 | Preview または Run が実行される | AgentRun を作成する | `ResolvedSkillContext` が ledger と artifact に保存される | 実装済み |
| 6.3.1a | 実行時 skill set は Project Rule 由来と Agent Profile 由来を合成する。 | Project Rule と Agent Profile の両方に skill set が設定されている | Preview または Run を実行する | `project_rule.skill_sets + agent_profile.skill_set_ids` が重複排除され、CapabilityProbe により applied / skipped として `ResolvedSkillContext` に保存される | 実装済み |
| 6.3.2 | Resolved Run Packet は実行時に使った Work Item、Agent Profile、実行目的、working_dir、work_folder、goal、DispatchPlan、Policy、Review観点、Resolved Skill Context を固定する。 | Preview または Run が実行される | AgentRun を作成する | `ResolvedRunPacket` が ledger と artifact に保存され、Adapter 実行入力として使われる | 予定 |
| 6.3.3 | Work Item 詳細で解決済み Skill Context と Run Packet を確認できる。 | 解決済み記録が存在する | `nagare item show <work_id>` を実行する | resolved_skill_contexts と resolved_run_packets が表示される | 実装済み |
| 6.3.4 | Context Budget は初期MVPでは固定上限とする。 | dispatch prompt を生成する | 候補 Agent Profile を選ぶ | 候補数は最大5件に固定され、設定化はしない | 実装済み |

## 7. Work Item Run / Adapter

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 7.1.1 | Work Item を Agent Profile で実行できる。 | Work Item と Agent Profile が存在する | `nagare item run <work_id> --agent <agent_profile_id>` を実行する | AgentRun、AgentOutputRecord、ExecutionRecord、Evidence が保存され、依頼された成果物がある場合だけ Artifact が保存される | 実装済み |
| 7.1.2 | `--command` は smoke / dev fallback として実行できる。 | Work Item と Agent Profile が存在する | `item run --command <command>` を実行する | command log が ExecutionRecord として保存される | 実装済み |
| 7.1.3 | `--prompt` は `process.codex-cli` adapter 経由で `codex exec` に渡す。 | adapter が `process.codex-cli` で Codex CLI が利用可能 | `item run --prompt <text>` を実行する | `codex exec --cd <working_dir> <prompt>` の結果が AgentRun に保存される | 実装済み |
| 7.1.4 | Agent Run の cwd は Agent Profile の `working_dir` を使う。 | Agent Profile に working_dir がある | Run を開始する | process cwd または Codex `--cd` が working_dir になる | 実装済み |
| 7.1.5 | `item run --work-folder` は Agent Profile の working_dir と照合して実行前提を記録する。 | work_folder が指定され、`--agent` が省略されている | `nagare item run <work_id> --work-folder <path>` を実行する | accepted DispatchPlan または work_agent で AgentRun が作成され、scope match が Run Packet に残る | 予定 |
| 7.1.6 | `item run` は採用済み DispatchPlan の target Agent Profile を使える。 | accepted DispatchPlan が存在し、`--agent` が省略されている | `nagare item run <work_id>` または `--dispatch-plan <id>` を実行する | AgentRun の agent_profile_id が DispatchPlan.target_agent_profile_id になる | 実装済み |
| 7.1.7 | Agent Profile は purpose別の OutputContract を持てる。 | Agent Profile が存在する | `nagare agent update <id> --output-purpose work --output-contract nagare.result.v1 --instruction-pack nagare-result-writer.v1` を実行する | Profile の `output_contracts.work` が更新される | 実装済み |
| 7.1.8 | Run Packet は使用した OutputContract を固定する。 | Agent Run を開始する | Run Packet を解決する | `ResolvedRunPacket.output_contract` に contract、instruction_pack、required、injection が保存される | 実装済み |
| 7.1.9 | OutputContract は prompt suffix としてAgentに注入される。 | Agent Run に prompt がある | Adapter 実行入力を作る | work/review/dispatch の purpose に応じた Nagare instruction pack が prompt に追加される | 実装済み |
| 7.1.10 | Work / Review の最終出力は AgentOutputRecord として保存する。 | AgentRun が終了する | `## Nagare Result` または `## Nagare Review` をparseする | parse_status、fields、questions、next_action、warnings が ledger に保存される | 実装済み |
| 7.1.11 | Agent出力に質問が含まれる場合、Work Item は人の入力待ちになる。 | AgentOutputRecord.questions が空ではない | Run を保存する | Work Item status が `needs_input` になる | 実装済み |
| 7.1.12 | required OutputContract がparseできない場合はwarningにする。 | required contract block が出力に存在しない | AgentOutputRecord を作る | `parse_status=unparsed` と `output_contract_unparsed` warning が残り、raw output は ExecutionRecord として保存される | 実装済み |
| 7.1.13 | 人はAgentからの質問に回答できる。 | Work Item が `needs_input` である | `nagare item answer <work_id> --answer <text>` を実行する | HumanFeedback が保存され、Work Item status が `ready` になる | 実装済み |
| 7.1.14 | 人の回答は次のAgent実行に渡される。 | HumanFeedback が存在する | 次の `item run` を実行する | prompt に `## Nagare Human Feedback` が追加され、Run Packet constraints に `human_feedback_context_applied` が残る | 実装済み |
| 7.1.15 | Work Item Snapshot は実行履歴を `WorkItemHistoryStep` として正規化する。 | Work Item に run、artifact、evidence、question、human feedback、review、handoff、decision が存在する | `nagare item show <work_id>` を実行する | request から approval までの主要 step が `history_steps` に同一形で表示され、互換用 `timeline` も生成される | 実装済み |
| 7.1.16 | Agentからの質問と人の回答は同じTimeline上で追える。 | AgentOutputRecord.questions と HumanFeedback が存在する | Snapshot を取得する | `question` と `human_feedback` event が関連 id と summary を持って表示される | 実装済み |
| 7.1.17 | Work Item Snapshot は完遂に向けた次アクションを算出する。 | Work Item の ledger record が存在する | `nagare item show <work_id>` を実行する | `completion.state`、`blocking_reason`、`next_action`、`next_command_hint` が表示される | 実装済み |
| 7.1.18 | Review出力はReviewResultとして保存し、状態遷移に接続する。 | review purpose のAgentRunが終了する | `## Nagare Review` をparseする | `ReviewResult` が保存され、pass は `ready_for_review` のまま承認待ちに進む。request_changes は `changes_requested`、blocked/questions は `needs_input` になる | 実装済み |
| 7.1.19 | Review は acceptance criteria ごとの評価を保存する。 | Work Item に acceptance criteria が存在する | `## Nagare Review` の `criteria` をparseする | `ReviewResult.criteria_results` に criterion / status / note が保存される | 実装済み |
| 7.1.20 | Acceptance criteria 未充足の Review pass は承認可能状態へ進めない。 | Criteria が未評価または failed である | review または approval を実行する | Work Item は `changes_requested` になり、approval は拒否される | 実装済み |
| 7.1.21 | Work Item Snapshot は approval gate の判断材料を表示する。 | Work Item に review、artifact、recovery が存在する | `nagare item show <work_id>` を実行する | `approval_gate` に ready/state、最新 review、criteria 充足数、blockers、approval command hint が表示される。独立した検査 blocker は使わない | 実装済み |
| 7.1.22 | Work / Review output は生成結果と次工程コメントを構造化して残せる。 | `## Nagare Result` または `## Nagare Review` に `completed` / `summary` / `next_notes` が含まれる | AgentRun を保存し、`nagare item show <work_id>` または Static UI detail を確認する | ユーザーへの結果は AgentOutputRecord.fields に保存され、`next_notes` は次の dispatch / agent へのヒントとして表示される | 実装済み |
| 7.1.23 | Work / Review output の notes 欠落は警告として残る。 | `## Nagare Result` または `## Nagare Review` が parsed だが `completed` / `next_notes` がない | AgentRun を保存する | AgentOutputRecord は parsed のまま `missing_completed` / `missing_next_notes` warnings を持ち、既存状態遷移は維持される | 実装済み |
| 7.1.24 | Review output の契約違反は review の再実行ループにしない。 | Work Item が `ready_for_review` で、review AgentRun が `## Nagare Review` を返さず `output_contract_unparsed` になる | Snapshot / WorkflowDecision を確認する | Work Item は `changes_requested` になり、completion.next_action は `recover`、WorkflowDecision は `create_recovery_plan` になる | 実装済み |
| 7.2.1 | `stdio.codex-app-server` は Agent Profile の runtime/adapter として登録・確認できる。 | Codex app-server runtime が設定済み | agent add/list/show/doctor/probe を実行する | Agent Profile と probe 結果が扱える | 実装済み |
| 7.2.2 | `stdio.codex-app-server` の実実行は stdio JSON-RPC adapter で扱う。 | Run Packet が存在する | app-server adapter で run を開始する | `initialize`、`thread/start`、`turn/start`、`turn/completed` の transcript が AgentRun ExecutionRecord に保存される | 実装済み |
| 7.3.1 | Codex MCP Server、Claude Code、HTTP adapter、SDK adapter は初期 Agent adapter に含めない。 | Adapter を登録または選定する | 初期 adapter scope を確認する | 対応予定は `process.codex-cli` と `stdio.codex-app-server` のみになる | 実装済み |

## 8. Artifact / Evidence

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 8.1.1 | Agent Run の stdout/stderr と終了状態を ExecutionRecord として保存する。 | Agent Run が開始される | Run が終了する | `.nagare/logs/` 配下に実行記録が残る | 実装済み |
| 8.1.2 | Artifact はユーザーが依頼した成果物または成果物作成に必要なファイルだけを指す。 | Artifact が作成される | Ledger に保存する | artifact_id、work_item_id、agent_run_id を辿れ、ログや raw output は artifact に含まれない | 実装済み |
| 8.2.1 | Agent Run の結果から Evidence を生成する。 | Run が終了する | Ledger を更新する | 成功または失敗 claim と basis が保存される | 実装済み |
| 8.2.2 | Evidence の自動生成文言は Project locale に合わせる。 | Project locale が設定済み | Evidence を生成する | 日本語または英語の claim / basis が保存される | 実装済み |
| 8.2.3 | Evidence は Artifact または ExecutionRecord を根拠として参照する。 | Artifact または ExecutionRecord が存在する | Evidence を作成する | evidence.artifact_id または evidence.execution_record_id から根拠を参照できる | 実装済み |
| 8.2.4 | Git workspace の変更ファイル一覧をExecutionRecord化する。 | Agent Run がGit work tree内で終了する | run 後に workspace を確認する | `.nagare/logs/` に `changed_files` ExecutionRecord が保存される | 実装済み |
| 8.2.5 | Git diff をExecutionRecord化する。 | tracked file に差分がある | run 後に `git diff --binary` を取得する | `diff_patch` ExecutionRecord が保存され、成果物 artifact には数えない | 実装済み |

## 9. Review Checks

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 9.1.1 | Review は必要な CI / test / schema check / artifact check を含めて実行する。 | Work Item が review 待ちである | Review Agent を実行する | 実行した検査、結果、根拠が `ReviewResult.completed`、`findings`、`criteria_results`、`referenced_artifacts`、ExecutionRecord に残る | 実装済み |
| 9.1.2 | Review で問題がなければ approval 可能にする。 | Review が pass で criteria も充足している | ReviewResult を保存する | WorkflowDecision は `approve` を選ぶ | 実装済み |
| 9.1.3 | Review で問題があれば指摘点として扱う。 | Review が request_changes、criteria failed、または check failure を返す | ReviewResult を保存する | Work Item は `changes_requested` になり、指摘点をもとに work 再実行または recovery へ進む | 実装済み |

## 10. Handoff

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 10.1.1 | Work Item の Handoff Packet を作成できる。 | Work Item と from/to Agent Profile が存在する | `nagare handoff create <work_id> --from-agent <id> --to-agent <id> --reason <text>` を実行する | HandoffPacket が ledger に保存される | 実装済み |
| 10.1.2 | Handoff Packet は reason と summary を保存する。 | Handoff を作成する | summary を指定または省略する | reason、summary、from_agent_profile、to_agent_profile が保存される | 実装済み |
| 10.1.3 | Handoff Packet には locale を保存する。 | Project locale が設定済み | Handoff を作成する | HandoffPacket.locale が保存される | 実装済み |
| 10.2.1 | Handoff から別 Agent Profile で再実行できる。 | HandoffPacket が存在する | `item run --agent <to_agent>` を実行する | 新しい AgentRun が同じ Work Item に追加される | 実装済み |
| 10.2.2 | Handoff 後に dispatch_agent で再確認できる。 | HandoffPacket が存在する | `nagare handoff dispatch <work_id>` を実行する | `dispatch_preview` 目的の AgentRun、ExecutionRecord、Evidence が保存される | 実装済み |
| 10.2.3 | 将来の Handoff は current state、artifact、evidence、open questions、requested output、review findings を含む。 | Work Item に実行履歴がある | Handoff を作成する | 次 Agent が判断できる構造化 packet が残る | 計画 |
| 10.2.4 | Handoff Packet は現在状態、未解決質問、artifact、diff、review、next request を保存する。 | Work Item に履歴がある | `nagare handoff create` を実行する | 後続 Agent Run の prompt / Run Packet constraints に handoff context が反映される | 実装済み |

## 11. Human Decision

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 11.1.1 | Review pass 済み Work Item を人間が approve できる。 | Work Item に passing review がある | `nagare decision approve <work_id>` を実行する | HumanDecision が保存され、Work Item が `done` になる | 実装済み |
| 11.1.2 | passing review がない Work Item の approve は拒否する。 | passing review が存在しない | `decision approve` を実行する | エラーになり、Work Item は `done` にならない | 実装済み |
| 11.1.3 | rationale が省略された場合は Project locale に合わせた既定理由を保存する。 | Project locale が設定済み | rationale なしで approve する | 日本語または英語の rationale が保存される | 実装済み |
| 11.1.4 | approval OK はワンアクションで完了できる。 | approval gate が ready である | UI の `Approve and finish` または `decision approve` を実行する | 追加入力なしで HumanDecision が保存され、Work Item が `done` になる | 実装済み |
| 11.1.5 | approval NG は理由付きで差し戻し、次は dispatch から再判断する。 | approval gate が ready である | `nagare decision reject <work_id> --rationale <text>` または UI の reject を実行する | `decision_type=reject` と理由が保存され、既存 dispatch/recovery は supersede され、Work Item は `ready` / `next_action=dispatch` になる | 実装済み |
| 11.2.1 | approve / reject 以外の request_changes、pause、delegate、override を保存できる設計にする。 | Review または Human action が発生する | Decision を記録する | decision_type と rationale が ledger に残る | 計画 |

## 12. CLI / Distribution

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 12.1.1 | ユーザー操作は `nagare` command に統一する。 | npm または local build で CLI が利用可能 | ユーザーが機能を操作する | README / tutorial の操作例が `nagare` command で完結する | 実装済み |
| 12.1.2 | npm は install / distribution path として扱う。 | package が存在する | `npm run pack:npm` または package install を行う | `nagare` bin が配布される | 実装済み |
| 12.1.3 | テスト用のまとめ command はユーザー向け help に出さない。 | CLI help を表示する | `nagare help` を実行する | dev-only scenario command が表示されない | 実装済み |
| 12.2.1 | README とチュートリアルは英語版・日本語版をペアで管理する。 | ユーザー向け文書を更新する | README または tutorial を変更する | `README.md` / `README_ja.md`、`docs/tutorial.md` / `docs/tutorial_ja.md` の対応が維持される | 実装済み |
| 12.2.2 | 設計書と仕様書の正本は日本語で管理する。 | 設計または仕様を更新する | docs を変更する | `docs/architecture.md` と `docs/spec.md` が日本語で更新される | 実装済み |

## 13. Error / Message ID

| ID | 原因 | 検出条件 | ユーザーアクション | 再試行 | 関連仕様 |
| --- | --- | --- | --- | --- | --- |
| ERR-PRJ-0001 | Project 未初期化 | `.nagare/project.toml` または ledger がない | `nagare init` を実行する | 可 | 1.1.1 |
| ERR-WORK-0001 | Work Item 不明 | 指定 ID が ledger にない | `nagare item list` で ID を確認する | 可 | 2.2.3 |
| ERR-AGENT-0001 | Agent Profile 不明 | 指定 ID が config にない | `nagare agent list` で ID を確認する | 可 | 3.2.3 |
| ERR-AGENT-0002 | working_dir 不正 | 絶対 path または Project 外 path | Project 内の相対 path を指定する | 可 | 3.1.3 |
| ERR-REVIEW-0001 | 承認前Review不足 | passing review が存在しない | Review を実行する | 可 | 11.1.2 |
| ERR-RUN-0001 | Agent 実行失敗 | process exit code が non-zero | Artifact と Evidence を確認し、handoff または再実行する | 可 | 7.1.1 |

| ID | 文面テンプレ | 出力先 | 発生条件 | 関連仕様 / ERR |
| --- | --- | --- | --- | --- |
| MSG-PRJ-0001 | Project initialized at `<root>` | stdout | `nagare init` 成功 | 1.1.1 |
| MSG-LOC-0001 | locale updated | stdout | `nagare locale use` 成功 | 1.2.1 |
| MSG-WORK-0001 | created `<work_id>` | stdout | Work Item 作成成功 | 2.1.1 |
| MSG-AGENT-0001 | added agent profile `<id>` | stdout | Agent Profile 登録成功 | 3.1.1 |
| MSG-RUN-0001 | run `<run_id>` completed | stdout | Agent Run 完了 | 7.1.1 |
| MSG-REVIEW-0001 | review `<id>` passed | stdout | Review成功 | 9.1.2 |
| MSG-DEC-0001 | work item approved | stdout | Human approval 成功 | 11.1.1 |

## 14. Recovery / Completion

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 14.1.1 | Work Item の停止理由からRecoveryPlanを作成できる。 | Work Item が停止または要対応状態である | `nagare item recover <work_id>` を実行する | action、reason、summary、target_agent、command_hint を持つ draft RecoveryPlan が保存される | 実装済み |
| 14.1.2 | RecoveryPlan は lifecycle を持つ。 | draft RecoveryPlan が存在する | `nagare item recover accept <work_id>` を実行する | 選択 plan が accepted になり、同じ Work Item の他 draft は superseded になる | 実装済み |
| 14.1.3 | OutputContract欠落は定型出力再生成のRecoveryPlanになる。 | 最新 AgentOutputRecord が `unparsed` である | `nagare item recover <work_id>` を実行する | `rerun_with_contract_reminder` action と prompt_hint が保存される | 実装済み |
| 14.1.4 | 採用済みRecoveryPlanをAgent再実行に適用できる。 | accepted RecoveryPlan がある | `nagare item recover apply <work_id>` を実行する | target Agent Profile で再実行され、結果が通常の AgentRun / AgentOutputRecord として保存される | 実装済み |
| 14.1.5 | RecoveryPlan は failure class を保存する。 | RecoveryPlan を作成する | 停止理由を分類する | `contract_violation`、`review_changes`、`missing_artifact`、`no_diff` などが保存される | 実装済み |
| 14.1.6 | RecoveryPlan は追加候補を作成できる。 | 期待成果物不足や diff 不足が検出される | `nagare item recover <work_id>` を実行する | primary plan に加えて補助 draft plan が保存される | 実装済み |
| 14.1.7 | Output notes 欠落は専用RecoveryPlanになる。 | 最新 Work / Review AgentOutputRecord に `missing_completed` または `missing_next_notes` warning がある | `nagare item recover <work_id>` を実行する | `failure_class=output_notes_missing` の `rerun_with_contract_reminder` plan が保存される | 実装済み |

## 15. Workflow Advance

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 15.1.1 | WorkflowDecision を台帳に保存する。 | Work Item が存在する | `create_workflow_decision` または `item advance` を実行する | action、reason、requires_human、target_agent、confidence が保存され、Timeline に出る | 実装済み |
| 15.1.2 | Work Item を1ステップ進められる。 | Work Item が存在する | `nagare item advance <work_id>` を実行する | 現在状態に応じて dispatch、accept、run、review、recover のうち1つだけ実行する | 実装済み |
| 15.1.3 | Work Item を停止点まで進められる。 | Work Item が存在する | `nagare item advance <work_id> --until-blocked true` を実行する | max steps 以内で次アクションを繰り返し、人間入力、handoff、recovery、approval、done で停止する | 実装済み |
| 15.1.4 | 複雑な復旧ワークフローを回帰テストで固定する。 | review changes が発生する | advance、recover、rerun、review を実行する | recovery 後に approval gate へ戻れることをテストで確認する | 実装済み |
| 15.1.5 | supervisor_agent は WorkflowDecision を作成できる。 | supervisor_agent が設定済み | `nagare item advance <work_id> --supervisor true` を実行する | `workflow_supervision` AgentRun と `source=supervisor_agent` の WorkflowDecision が保存される | 実装済み |
| 15.1.6 | 採用済みRecoveryPlanをadvanceから適用できる。 | accepted RecoveryPlan が存在する | `nagare item advance <work_id>` を実行する | WorkflowDecision が `apply_recovery_plan` になり、accepted plan が同じWork ItemのAgentRunとして適用される | 実装済み |
| 15.1.7 | draft RecoveryPlanを明示オプションで自動承認してadvanceを継続できる。 | draft RecoveryPlan が存在する | `nagare item advance <work_id> --until-blocked true --auto-recover true` を実行する | draft plan は `accept_recovery_plan` として承認され、applicable な rerun plan は `apply_recovery_plan` へ進み、approval / input / handoff などの人間ゲートでは停止する | 実装済み |
| 15.1.8 | Work Item の finish_first mode は recovery 自動継続方針として扱われる。 | `workflow_mode=finish_first` の Work Item が review changes で停止する | `nagare item advance <work_id> --until-blocked true` を実行する | `--auto-recover true` なしで draft RecoveryPlan の accept/apply を進め、approval / input / handoff などの人間ゲートで停止する | 実装済み |
| 15.1.9 | 作成済み HandoffPacket は finish_first の継続導線として扱われる。 | `workflow_mode=finish_first` の Work Item が `needs_handoff` で HandoffPacket を持つ | `nagare item advance <work_id> --until-blocked true` を実行する | handoff dispatch、dispatch accept、target agent run、review を経て approval gate で停止する。HandoffPacket 未作成時は `create_handoff` で停止する | 実装済み |
| 15.1.10 | auto approval policy は passing review 後に完了できる。 | `approval_policy=auto_complete_on_review_pass` の Work Item が approval gate ready である | `nagare item advance <work_id>` を実行する | WorkflowDecision は `done` / `review_passed_auto_complete` を保存し、approval decision を記録して Work Item が `done` になる | 実装済み |

## 16. Static UI Export

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 16.1.1 | Work Item Board を静的HTMLとして出力できる。 | Project が初期化済み | `nagare ui export --out <dir>` を実行する | `<dir>/index.html` が生成され、Work Itemの状態、次アクション、Agent、DoD概要が表示される | 実装済み |
| 16.1.2 | Work Item Detail を静的HTMLとして出力できる。 | Work Item が存在する | `nagare ui export --out <dir>` を実行する | `<dir>/items/<work_id>.html` が生成され、Timeline、WorkflowDecision、Approval Gate、Agent Output Notes、Review criteria、Recovery、Handoffが表示される | 実装済み |
| 16.1.3 | 静的UIは設計文書の主導線に合わせる。 | Work Itemに履歴がある | 生成HTMLを開く | `次へ進める`、`判断を確認`、`回復案を見る` の判断材料が見える | 実装済み |
| 16.1.4 | 静的UIは外部ビルドツールなしでCLIから生成できる。 | `nagare` CLI が実行可能 | `nagare ui export` を実行する | Rust CLIのみでHTML/CSSが生成される | 実装済み |
| 16.1.5 | Work Item Detail は次操作を集約して表示する。 | Work Item に completion、workflow decision、approval gate、recovery、handoff、agent notes が存在する | `nagare ui export --out <dir>` を実行し detail を確認する | Next Action Panel に state、next action、recommended command、workflow mode、approval gate、active recovery、handoff、next notes が表示される | 実装済み |
| 16.1.6 | Static UI は確認対象の視認と追加指示入力を支援する。 | Work Item が確認待ち、recovery 待ち、または `needs_input` である | `nagare ui export --out <dir>` を実行し Board / Detail を確認する | Board の `確認キュー` と `attention-row` に対象が表示され、Detail の `Human Input Panel` で `item answer --answer` または `item advance --prompt` command を入力内容から作れる | 実装済み |
| 16.1.7 | Static UI を確認用コマンドで起動できる。 | Project が初期化済み | `nagare ui open --out <dir>` を実行する | UI export 後に `index.html` が既定ブラウザで開かれ、`--open false` では生成のみ確認できる | 実装済み |
| 16.1.8 | Static UI の入力導線はブラウザE2Eで検証する。 | `needs_input` の Work Item を含む project がある | `npm run test:e2e` を実行する | Playwright が生成HTMLを開き、確認キューのリンク遷移、Human Input Panel の textarea 入力、command 更新、表示 command の CLI 実行、再export後の `human_feedback` と次の `run_agent` 導線を検証する | 実装済み |
| 16.1.9 | Static UI E2E は回答後の agent run 段階まで検証する。 | `needs_input` への回答後に Work Item が `ready` になっている | `npm run test:e2e` を実行する | Human Input Panel が `item run --prompt` command を生成し、E2E が外部runtimeに依存しない `--command` に置換して実行し、再export後に agent run evidence と次の review 導線を確認する | 実装済み |
| 16.1.10 | ローカルUIサーバから Work Item を作成できる。 | Project が初期化済み | `nagare ui serve --host 127.0.0.1 --port <port>` を実行し、ブラウザフォームから title / description / acceptance / workflow mode を送信する | `POST /api/items` が Work Item を作成し、ブラウザ一覧と `nagare item list` に新規項目が表示される | 実装済み |
| 16.1.11 | ローカルUIサーバから人の回答を送信できる。 | Work Item が `needs_input` である | `nagare ui serve` の detail page で回答フォームを送信する | `POST /api/items/<work_id>/answer` が HumanFeedback を保存し、detail と一覧に `ready` と回答内容が表示される | 実装済み |
| 16.1.12 | ローカルUIサーバから Work agent を実行できる。 | Work Item が `ready` である | `nagare ui serve` の detail page で run form を送信する | `POST /api/items/<work_id>/run` が Work agent run を保存し、detail と `nagare item show` に `ready_for_review` と run evidence が表示される | 実装済み |
| 16.1.13 | ローカルUIサーバから Review agent を実行できる。 | Work Item が `ready_for_review` である | `nagare ui serve` の detail page で review form を送信する | `POST /api/items/<work_id>/review` が Review agent run を保存し、pass review では detail と `nagare item show` に approval ready と review evidence が表示される | 実装済み |
| 16.1.14 | ローカルUIサーバでは独立検査操作を出さない。 | Work Item detail を表示する | 次アクションを確認する | CI / test / artifact check は Review 内の記録として扱われ、独立検査 form/API は表示しない | 実装済み |
| 16.1.15 | ローカルUIサーバから Work Item を承認できる。 | Work Item の approval gate が ready である | `nagare ui serve` の detail page で approve form を送信する | `POST /api/items/<work_id>/approve` が HumanDecision を保存し、detail と `nagare item show` に `done` と approval decision が表示される | 実装済み |
| 16.1.16 | ローカルUIサーバから RecoveryPlan を作成・承認・適用できる。 | Work Item が失敗または停止し、`recover` が次アクションである | `nagare ui serve` の detail page で recovery form を送信する | `POST /api/items/<work_id>/recover`、`/recover/accept`、`/recover/apply` が RecoveryPlan lifecycle と rerun を保存し、review 待ちへ戻れる | 実装済み |
| 16.1.17 | ローカルUIサーバの detail は次アクション中心に情報を整理する。 | Work Item detail を表示する | 状態が変わる | サマリに status / next / workflow mode / single step / manual continuation が表示され、現在必要な操作フォームだけが primary action として表示される | 実装済み |
| 16.1.18 | ローカルUIサーバの Settings で Workflow、Agent Profile、Domain Profile を確認できる。 | Project が初期化済み | `nagare ui serve` の Settings を開く | Agent Defaults form は表示せず、Project workflow default、登録済み Agent Profile、Domain Profile の rubric / dispatch_hints / workflow override を確認できる | 実装済み |
| 16.1.19 | ローカルUIサーバは次Agentへの自動継続を行わない。 | Work Item が run / review / recovery apply の後続状態へ進む | UI操作を1回実行する | 次のAgentや次工程は自動実行されず、detail は `single step` / `manual continuation` と次に必要な操作フォームを表示して停止する | 実装済み |
| 16.1.20 | ローカルUIサーバから Agent Profile を追加・編集できる。 | Project が初期化済み | Settings の Agent Profile 追加/編集フォームを送信する | `POST /api/agents` または `POST /api/agents/<id>` が `.nagare/agents/<id>.toml` を作成・更新し、dispatch の選択材料になる | 実装済み |
| 16.1.20a | ローカルUIサーバから Domain Profile を追加・編集できる。 | Project が初期化済み | Settings の Domain Profile 追加/編集フォームを送信する | `POST /api/domains` または `POST /api/domains/<id>` が `.nagare/domains/<id>.toml` を作成・更新し、rubric、dispatch_hints、workflow override が保存される | 実装済み |
| 16.1.21 | ローカルUIサーバの Detail はCLI/static UIと同じ `WorkItemHistoryStep` 中心の履歴を表示する。 | Work Item に workflow decision、agent run、agent output、review、human feedback、approval、recovery、handoff が存在する | `nagare ui serve` の detail page を開く | Processing History は request / dispatch / work / review / input / handoff / recovery / approval を同じカード形で表示し、Artifact / Evidence / Agent Output / WorkflowDecision は各 step の facts / links / Details で参照する | 実装済み |
| 16.1.22 | ローカルUIサーバの Detail は各ステップの判断、証跡、コメントを確認できる。 | WorkflowDecision と AgentOutputRecord が保存済みである | Processing History のevent詳細を開く | WorkflowDecision の action/reason/source/target/requires_human/confidence/warnings と、Agent Output の summary/completed/next_notes/questions/next_action/output record が表示される | 実装済み |
| 16.1.23 | ローカルUIサーバの実行中表示は構造化された一時状態を使う。 | UI server が Work Item を background advance している | `.nagare/state/<work_id>-ui-running.txt` を読む | JSON の `kind`、`actor`、`label`、`message`、`related_action`、`started_at_epoch` から実行中カードを表示し、旧line形式も互換読み込みできる | 実装済み |
| 16.1.24 | ローカルUIサーバの Home は Work Queue を主画面として扱う。 | Project が初期化済み | `nagare ui serve` の home を開く | Work Queue が主領域いっぱいに表示され、右上の `Create New Item` から作成画面へ移動でき、状態ショートカットで Work Queue を絞り込める。`Selected Work` のサマリ列は出さない | 実装済み |
| 16.1.25 | ローカルUIサーバの Detail はパンくずで一覧との位置関係を示す。 | Work Item が存在する | `nagare ui serve` の detail page を開く | 左ナビに Detail 専用項目を出さず、本文上部に `Work Queue / Detail` のパンくずを表示する | 実装済み |
