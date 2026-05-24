# architecture.md（必ず書く：最新版）

この文書は Nagare / 流 の実装設計の正本である。要件の背景は
`docs/nagare_requirements_v0_3.md`、機能仕様は `docs/spec.md`、
Agent Profile / Skill の詳細データ形式は `docs/agent_data_model.md`、Adapter 境界は
`docs/adapter_contract.md` を参照する。
この文書の本文は日本語で管理し、CLI 名、型名、ファイル名、設定 key などの
識別子は英語のまま記載する。

## 1. アーキテクチャ概要（構成要素と責務）

Nagare は、Agent 実行環境そのものではなく、Agent 作業を Work Item 単位
で整流するローカル優先の制御レイヤーである。

主要構成要素:

- CLI: `nagare` コマンド。ユーザー操作、スモークテスト、将来の自動化入口。
- Core: Work Item、Agent Run、Evidence、Verification、Handoff、Decision の不変条件と状態遷移を所有する。
- Agent Management: Runtime、Adapter、Agent Profile、Skill Set、Capability Probe、Project Rule、Policy を解決する。
- Adapter Kernel: Run Packet を各 Agent runtime の入力形式へ変換し、Run Event / Artifact / Evidence を回収する。
- Ledger: Work Item と実行証跡の永続化。MVP では JSON ledger、後続で SQLite を使う。
- Artifacts: log、Run Packet、Probe 出力、diff、screenshot、transcript などの大きい証跡。
- npm package: 配布経路。製品操作面は `nagare` コマンドに統一する。

### 1.1 レイヤー責務

Nagare の実装は、以下のレイヤーに分ける。依存方向は常に外側から内側へ向ける。
内側のレイヤーは外側の都合を知らない。

| レイヤー | 責務 | 置き場所の目安 | 禁止事項 |
| --- | --- | --- | --- |
| Presentation / CLI | 引数 parsing、人間向け出力、exit code、command routing | `crates/nagare-cli/src/*` | ledger JSON、TOML、process 実行、状態遷移を直接扱わない |
| UseCase | 1つのユーザー操作を成立させる orchestration。Domain と Port を組み合わせる | `crates/nagare-core/src/usecases/*` | CLI 表示文言、具体的な filesystem/process 実装を持たない |
| Domain | Work Item、Agent Run、Evidence、Verification、Handoff、Decision、Run Packet の型、不変条件、状態遷移 | `crates/nagare-core/src/domain/*` | filesystem、process spawn、TOML/JSON I/O、CLI 出力を持たない |
| Agent Management | Runtime、Adapter、Agent Profile、Skill Set、Project Rule、Policy、Capability Probe、Resolved Context の解決 | `crates/nagare-core/src/agent/*` | Work Item の完了判定や Human Decision を所有しない |
| Adapter Kernel | Resolved Run Packet を runtime 固有 protocol へ変換し、Run Event / output を正規化する | `crates/nagare-core/src/adapters/*` | Work Item を `done` にしない。Human Decision を作らない |
| Infrastructure | JSON ledger、artifact store、TOML config、process、clock、filesystem | `crates/nagare-core/src/infra/*` | Domain rule を判断しない。UseCase を呼ばない |
| Distribution | npm wrapper、binary staging、package metadata | `scripts/*`, `packages/*` | product behavior を実装しない |

### 1.2 主要コンポーネント責務

| コンポーネント | 責務 | 主な入力 | 主な出力 |
| --- | --- | --- | --- |
| Project Layout | `.nagare` 配下の標準 path を決める | project root | config / ledger / artifacts / logs path |
| Config Repository | `.nagare/project.toml` と `.nagare/agents/*.toml` を読む/書く | TOML files | Locale、Nagare Agent defaults、Agent Profile、Runtime、Project Rule |
| Ledger Repository | ledger-owned entity を load/save する | `ledger.json` | WorkItem、AgentRun、Evidence、Verification、Handoff、Decision、Resolved records |
| Artifact Store | 大きい証跡を file として保存する | bytes / structured record | artifact URI |
| Work Item Service | Work Item の作成、表示 snapshot、状態遷移を扱う | Work Item command | WorkItem / WorkItemSnapshot |
| Agent Registry Service | Agent Profile の登録、一覧、詳細、health/probe を扱う | Agent Profile command | AgentProfile / AgentDoctorReport / CapabilityProbe |
| Rule Resolver | path から Project Rule、Agent Profile、Skill Set、Policy、Verification を解決する | path、optional agent override | RuleResolution |
| Run Resolver | RuleResolution と Probe から ResolvedSkillContext / ResolvedRunPacket を作る | Work Item、Agent Profile、RuleResolution、Probe | ResolvedSkillContext / ResolvedRunPacket |
| Probe Resolver | Run / Preview 前に CapabilityProbe を再利用または更新する | Agent Profile、Runtime、既存 Probe | CapabilityProbe |
| Skill Set Resolver | Agent capability と Skill Set required capability を照合し、applied / skipped を決める | Skill Set、CapabilityProbe | SkillSetResolution |
| Run Orchestrator | Agent Run を開始し、Artifact / Evidence / AgentRun を記録する | ResolvedRunPacket、prompt、adapter | RunWorkItemResult |
| Dispatch Planner | dispatch preview の Agent 出力を実行前確認用の DispatchPlan として保存する | AgentRun、ResolvedRunPacket、Artifact | DispatchPlan |
| Verification Service | command / verifier の結果を VerificationResult と Evidence にする | Work Item、verifier | VerifyResult |
| Handoff Service | Handoff Packet を作り、次工程の文脈を残す | Work Item、from/to Agent | HandoffPacket |
| Decision Service | Human Decision を保存し、`done` 遷移を成立させる | Work Item、rationale | HumanDecision |
| Adapter Implementations | runtime 固有の実行方法を隠蔽する | Prepared run | AdapterRunOutput / RunEvent |

### 1.3 単一責務とファイルサイズ制約

Nagare は機能追加よりも、責務境界を維持することを優先する。ファイル分割の基準は以下とする。

- 1ファイルは原則 1000 行未満に保つ。
- 800 行を超えたら、次の変更で分割候補として扱う。
- 1000 行を超えるファイルは、機能追加前に分割計画を作る。
- 1ファイルは1つの主要責務だけを持つ。例: CLI parsing、ledger repository、rule resolution、adapter 実装を同じ file に混ぜない。
- public API を集約する `lib.rs` や `main.rs` は薄い module 宣言と re-export に寄せる。実装本体を置き続けない。
- テストは対象 module の近くに置く。ただし scenario / CLI smoke のような横断テストは専用 module に分ける。
- 機能追加で既存ファイルが 1000 行を超える場合、同じ差分内で分割するか、先に分割 commit を作る。

現在の実装では `crates/nagare-core/src/lib.rs` と `crates/nagare-cli/src/main.rs`
を薄い入口にし、主要責務を module へ分割済みである。後続でさらに責務が増えた場合は、
以下の詳細構造へ段階的に分割する。

```text
crates/nagare-core/src/
  lib.rs                 public API と module export のみ
  layout.rs              ProjectLayout / root resolution
  error.rs               NagareError
  domain/
    mod.rs
    work_item.rs
    run.rs
    evidence.rs
    verification.rs
    handoff.rs
    decision.rs
  config/
    mod.rs
    project.rs
    agent_profile.rs
  ledger/
    mod.rs
    json_store.rs
    snapshot.rs
  agent/
    mod.rs
    registry.rs
    probe.rs
    rule_resolution.rs
    run_resolution.rs
  adapters/
    mod.rs
    process_codex_cli.rs
    stdio_codex_app_server.rs
  usecases/
    mod.rs
    item.rs
    agent.rs
    locale.rs
    verify.rs
    handoff.rs
    decision.rs
  scenario.rs

crates/nagare-cli/src/
  main.rs                command dispatch のみ
  args.rs                ParsedArgs / root option
  output.rs              print helpers
  commands/
    mod.rs
    init.rs
    agent.rs
    rule.rs
    item.rs
    locale.rs
    verify.rs
    handoff.rs
    decision.rs
```

Nagare が所有するもの:

- Work Item
- Project Rule
- Agent Profile
- Run Request
- Run Packet
- Dispatch Plan
- Agent Run
- Artifact
- Evidence
- Verification Result
- Handoff Packet
- Human Decision

Agent runtime が所有するもの:

- モデル選択
- runtime 固有のセッション状態
- runtime 内部の計画
- tool 実行の詳細
- runtime 固有の権限確認
- provider 固有の記憶領域

## 2. concept のレイヤー構造との対応表

`concept.md` は未作成のため、現時点では要件書の「Kanban + Execution
Ledger + Evidence Board + Agent Registry」を概念ソースとする。

| 概念レイヤー | 実装レイヤー | 責務 |
| --- | --- | --- |
| Kanban / Board | UI / CLI / API | Work Item の作成、確認、承認、異常対応 |
| Execution Ledger | Core / Ledger | 状態遷移、実行履歴、証跡、Decision の保存 |
| Evidence Board | Core / Artifact Store | Artifact、Evidence、Verification、Review の関連付け |
| Agent Registry | Agent Management | Runtime、Adapter、Agent Profile、Skill Set、Project Rule の管理 |
| Adapter-first execution | Adapter Kernel / Infrastructure | Agent runtime ごとの差異を Run Packet / Run Event に正規化 |

依存方向は外から内へ固定する。

```text
CLI / UI / API
  -> UseCase
    -> Domain

Infrastructure / Adapter implementations
  -> UseCase ports
    -> Domain
```

UseCase は SQL、HTTP、process spawn、filesystem の詳細を直接所有しない。
それらは port の実装として Infrastructure / Adapter 側に置く。

## 3. インターフェース設計

### UI / APP 境界

MVP では UI は CLI である。将来の Web UI / REST API / automation endpoint も同じ
UseCase を呼ぶ。

UseCase 境界:

- `InitProject(root) -> InitResult`
- `CreateWorkItem(root, title, description) -> WorkItem`
- `ListWorkItems(root) -> Vec<WorkItem>`
- `ShowWorkItem(root, work_item_id) -> WorkItemSnapshot`
- `ResolveProjectRule(root, path, agent?) -> RuleResolution`
- `PreviewWorkItemRun(root, work_item_id, path?, agent?) -> RunPreview`
- `ReviewWorkItem(root, work_item_id, agent?) -> AgentRun`
- `RunWorkItem(root, work_item_id, agent_profile_id, command | resolved_run_packet) -> AgentRun`
- `VerifyWorkItem(root, work_item_id, command | verifier_id) -> VerificationResult`
- `CreateHandoff(root, work_item_id, from_agent, to_agent, reason, summary) -> HandoffPacket`
- `ApproveWorkItem(root, work_item_id, rationale) -> HumanDecision`

### 外部 I/F

Agent runtime adapter の境界:

```text
healthcheck() -> AdapterHealth
probe(agent_profile, project_context) -> CapabilityProbe
prepare(run_packet, workspace_policy, permission_policy) -> PreparedRun
start(prepared_run) -> ExternalRunId
stream(external_run_id) -> Vec<RunEvent>
collect(external_run_id) -> CollectedOutput
cancel(external_run_id) -> CancelResult
```

対応 Agent adapter:

- `process.codex-cli`
- `stdio.codex-app-server`

Agent adapter ではないもの:

- `nagare verify --command` が使う shell command
- smoke test 用の local command
- 別言語の SDK wrapper

初期対象外:

- OpenCode adapter
- Codex Cloud adapter
- HTTP worker adapter
- OpenCode HTTP server adapter
- Claude Code adapter
- Codex MCP Server

### 内部 I/F

Core の現在の実装入口:

- `init_project(root) -> InitResult`
- `doctor(root) -> DoctorReport`
- `get_locale_settings(root) -> LocaleSettings`
- `set_locale_settings(root, settings) -> LocaleSettings`
- `get_nagare_agent_settings(root) -> NagareAgentSettings`
- `set_nagare_agent_settings(root, settings) -> NagareAgentSettings`
- `resolve_rule_for_path(root, path?, agent?) -> RuleResolution`
- `create_work_item(root, title, description) -> CreateItemResult`
- `list_work_items(root) -> Vec<WorkItem>`
- `get_work_item_snapshot(root, work_item_id) -> WorkItemSnapshot`
- `run_work_item_with_input(root, work_item_id, RunWorkItemInput) -> RunWorkItemResult`
- `run_work_item(root, work_item_id, agent_profile_id, command) -> RunWorkItemResult` は smoke 互換用
- `verify_work_item(root, work_item_id, command) -> VerifyResult`
- `create_handoff(root, work_item_id, from_agent_profile, to_agent_profile, reason, summary) -> HandoffResult`
- `approve_work_item(root, work_item_id, rationale) -> DecisionResult`

現在の Agent 実行入口は `run_work_item_with_input` である。CLI では `item run`
の `--agent` が省略された場合、`nagare_agents.work_agent` を Work Item の
既定実行先として使う。`item preview` は `nagare_agents.dispatch_agent` を使い、
Agent Run の `purpose` を `dispatch_preview` として記録する。`item review` は
`nagare_agents.review_agent` を使い、`purpose` を `review` として記録する。
preview / review は Artifact と Evidence を残すが、Work Item status を実行結果で進めない。
dispatch preview はさらに DispatchPlan を保存し、AgentRun、ResolvedRunPacket、
実行ログ Artifact と紐づける。
`--path` が指定された場合は Project Rule を解決し、Agent Profile、Skill Set、
Permission Policy、Workspace Policy、Verification を表示する。`item run --path` で
`--agent` が省略された場合は、Project Rule の `default_agent` を実行先に使う。
Run / Preview 前には CapabilityProbe を確認し、未取得、古い、runtime / adapter /
runtime_version 不一致の場合は自動で再 probe する。MVP では stale 判定 TTL は
24 時間の内部既定値とする。
Skill Set は Agent の CapabilityProbe と照合し、
required capability を満たすものを `applied_skill_set_ids`、満たさないものを
`skipped_skill_set_ids` に記録する。skip 理由は Run Packet の constraints に残す。
Agent Profile は
`working_dir` を持ち、実行時の cwd は project root からの相対 path として解決する。
`--prompt` は `process.codex-cli` adapter 経由で
`codex exec --cd <working_dir> <prompt>` を実行し、
`--command` は smoke 用 fallback として adapter I/F の内側で実行する。
`stdio.codex-app-server` adapter は `codex app-server --listen stdio://` を起動し、
JSON-RPC over stdio で `initialize`、`thread/start`、`turn/start`、`turn/completed`
を扱う。app-server の通知 transcript は AgentRun artifact に保存する。

### 型定義

主要 Domain 型:

- `WorkItem`: id、title、description、locale、status、created_at、updated_at
- `AgentRun`: id、work_item_id、agent_profile_id、adapter、purpose、status、exit_code、artifact_id、locale
- `AgentRunPurpose`: work、dispatch_preview、review
- `ResolvedSkillContext`: id、work_item_id、agent_profile_id、capability_probe_id、project_rule_ids、declared_skill_set_ids、applied_skill_set_ids、skipped_skill_set_ids、capabilities_in_force、instruction_sources、artifact_uri、content_hash、locale
- `ResolvedRunPacket`: id、work_item_id、agent_profile_id、adapter_id、purpose、working_dir、goal、path、permission_policy_id、workspace_policy_id、resolved_skill_context_id、project_rule_ids、verification、constraints、artifact_uri、content_hash、locale
- `DispatchPlan`: id、work_item_id、status、agent_run_id、dispatch_agent_profile_id、target_agent_profile_id、resolved_run_packet_id、raw_output_artifact_id、path、summary、risks、missing_information、locale
- `Artifact`: id、work_item_id、agent_run_id、artifact_type、uri、title、locale
- `Evidence`: id、work_item_id、claim、basis、artifact_id、produced_by、locale
- `VerificationResult`: id、work_item_id、result、artifact_id、locale
- `HandoffPacket`: id、work_item_id、from_agent_profile、to_agent_profile、reason、summary、locale
- `HumanDecision`: id、work_item_id、decision_type、rationale、locale
- `RuleResolution`: path、matched_rule_id、agent_profile_id、review_agent_profile_id、skill_set_ids、permission_policy_id、workspace_policy_id、verification、warnings

Agent Management 型:

- `RuntimeDeclaration`
- `AdapterDeclaration`
- `AgentProfileDeclaration`: runtime、adapter、role、working_dir、description、specialties を持つ
- `DeclaredSkillSet`
- `CapabilityProbe`
- `ResolvedSkillContext`
- `ProjectRule`
- `PermissionPolicy`
- `WorkspacePolicy`
- `ResolvedRunPacket`

## 4. 主要フロー設計（成功 / 失敗）

### 初期化

1. `nagare init`
2. `.nagare/project.toml` を作成する。
3. `.nagare/state/ledger.json` を作成する。
4. `.nagare/artifacts/` と `.nagare/logs/` を作成する。

既存ファイルがある場合は上書きせず、そのまま維持する。

### Work Item 実行成功

1. `nagare item create` で Work Item を `ready` として作成する。
2. `nagare item preview` で Agent Profile、Project Rule、Skill Set、Policy、Verification、Run Packet を確認する。
3. dispatch_agent は最大 5 件の compact な Agent Profile 候補から target Agent Profile を選び、DispatchPlan に記録する。
4. `nagare item dispatch accept` で DispatchPlan を accepted にする。
5. `nagare item run` で accepted DispatchPlan の target Agent Profile を使い Agent Run を開始する。
6. Adapter が stdout/stderr、成果物、exit code を回収する。
7. 成功なら Work Item を `ready_for_review` にする。
8. Evidence と実行ログ Artifact を保存する。
9. `nagare verify` が通る。
10. `nagare decision approve` で `done` にする。

### Agent Run 失敗

1. Agent Run が non-zero exit、timeout、protocol violation、cancel のいずれかで終了する。
2. Work Item を `failed_verification` または `needs_human` にする。
3. 失敗ログを Artifact として保存する。
4. Evidence に失敗 claim と basis を保存する。
5. 必要なら `nagare handoff create` で別 Agent Profile へ引き継ぐ。

### Handoff

1. 失敗またはレビュー結果から Handoff Packet を作成する。
2. `from_agent_profile`、`to_agent_profile`、reason、summary を保存する。
3. `handoff dispatch` は `item preview` と同じ dispatch_agent / candidate context / DispatchPlan lifecycle を使う。
4. 採用済み DispatchPlan があれば、次の `item run` はその target Agent Profile を使う。

### Human Approval

1. Work Item は `ready_for_review` である必要がある。
2. passing verification が少なくとも 1 件必要。
3. Human Decision を保存する。
4. Work Item を `done` にする。

## 5. データ設計（永続化・整合性・マイグレーション）

### 保存領域の分離

```text
.nagare/project.toml
  project 単位の宣言

.nagare/agents/*.toml
  project-local な Agent Profile 宣言

.nagare/rules/*.toml
  path / glob 単位の任意 Project Rule

.nagare/state/ledger.json
  MVP の ledger

.nagare/artifacts/
  Run Packet、Probe 出力、diff、screenshot、transcript

.nagare/logs/
  command log と adapter log
```

Config-owned（宣言として保存するもの）:

- RuntimeDeclaration
- AdapterDeclaration
- AgentProfileDeclaration
- DeclaredSkillSet
- ProjectRule
- PermissionPolicy
- WorkspacePolicy

Ledger-owned（観測事実として保存するもの）:

- WorkItem
- CapabilityProbe
- ResolvedSkillContext
- ResolvedRunPacket
- DispatchPlan
- RunRequest
- AgentRun
- Artifact
- Evidence
- VerificationResult
- HandoffPacket
- HumanDecision

### 整合性

- Work Item の `done` は Human Decision だけで成立する。
- Agent Run は Work Item を直接 `done` にできない。
- Evidence は claim と basis を必ず持つ。
- Verification Result は log Artifact と紐づく。
- Handoff は from/to の Agent Profile を必ず持つ。
- Run Packet は解決済み Project Rule、Skill Context、Policy、実行目的、作業ディレクトリ、実行目標の hash / id または値を持つ。
- Skill Set は required capability を満たす場合だけ applied として記録し、満たさない場合は skipped と constraints に記録する。
- Run / Preview は fresh な CapabilityProbe を ResolvedSkillContext に紐づける。
- Preview / Run は ResolvedSkillContext と ResolvedRunPacket を ledger と artifact に保存する。
- Preview / Handoff Dispatch は DispatchPlan を ledger に保存し、raw output artifact を参照する。
- Dispatch target の最終判断は Nagare の `dispatch_agent` が行う。
- Dispatch prompt の Agent context は最大 5 件の Agent Profile summary に制限し、巨大な instruction source 本文は渡さない。
- Dispatch output の JSON `target_agent_profile_id` は、登録済み Agent Profile に一致する場合だけ DispatchPlan に採用する。
- DispatchPlan は `draft` / `accepted` / `superseded` の lifecycle を持つ。
- `item run` の agent 解決順は、明示 `--agent`、明示 accepted `--dispatch-plan`、最新 accepted DispatchPlan、Project Rule、`work_agent` とする。

### マイグレーション

MVP では `ledger.json` を正本とする。SQLite へ移行する際も entity 名と
境界は維持する。

移行方針:

1. `ledger.json` に schema version を追加する。
2. SQLite table を ledger-owned entity 単位で作る。
3. JSON import を冪等にする。
4. import 後も artifact URI は変えない。

## 6. 設定：場所 / キー / 既定値

初期設定ファイル:

```text
.nagare/project.toml
```

主要 key:

- `[project]`
- `[storage]`
- `[locale]`
- `[nagare_agents]`
- `[runtimes.*]`
- `[adapters.*]`
- `[agent_profiles.*]`
- `[skill_sets.*]`
- `[permission_policies.*]`
- `[workspace_policies.*]`
- `[[project_rules]]`

既定 Agent Profile:

- `codex-cli`: implementer / `process.codex-cli`
- `codex-app-server`: implementer / `stdio.codex-app-server`

Nagare 本体が使う既定 Agent:

- `work_agent`: `item run` の既定実行先。`--agent` があれば明示指定を優先する
- `review_agent`: 後続のレビュー/検証支援用の既定Agent
- `dispatch_agent`: 後続の dispatch 案作成や Work Item routing 用の既定Agent

Locale:

- `language`: `ja-JP` / `en-US` など。Nagare が生成する Evidence / Verification / Decision の文言と記録に保存する
- `timezone`: `Asia/Tokyo` など。MVPでは設定値として保存し、後続で日時表示に使う

CLI root 解決:

1. `--root <path>`
2. `NAGARE_ROOT`
3. current working directory（現在の作業ディレクトリ）

## 7. 依存と拡張点

拡張点:

- Runtime: process、stdio、ci、cloud
- Adapter: runtime ごとの protocol translation
- Agent Profile: role、limits、policies、skills の束
- Skill Set: instruction、schema、playbook、rubric、script の束
- Project Rule: path / glob による agent、skill、policy、verification の選択
- Verifier: command、CI、LLM judge、rubric review
- Storage: JSON ledger、SQLite、将来 PostgreSQL

Codex のサーバー型入口は `codex app-server --listen stdio://` を優先する。
これは HTTP control plane ではなく、stdio JSONL の双方向 protocol として
扱える。Codex 固有の thread、turn、approval、stream event を扱えるため、
Nagare の Agent Run / Evidence / Review への写像に向いている。

Codex MCP Server は Nagare の Agent adapter として使わない。
Nagare は言語ごとの SDK adapter を持たない。Rust core の adapter 境界は
`stdio.codex-app-server` に置く。
`codex remote-control`、`codex exec-server` は現行 CLI では experimental
なので、初期 adapter には入れず観測対象に留める。

OpenCode adapter、HTTP worker adapter、OpenCode HTTP server adapter、Claude Code
adapter は初期対象外とする。Kernel は HTTP control plane、言語別 SDK、Claude
Code semantics に依存しない。

## 7.5 依存関係（DI）

UseCase は port を要求する。

```text
LedgerPort
  load() -> Ledger
  save(ledger) -> ()

ArtifactStorePort
  write(path, bytes) -> ArtifactUri

ClockPort
  now() -> Timestamp

ProcessPort
  run(command, cwd?, env?) -> CommandRunOutput

AdapterPort
  healthcheck/probe/prepare/start/stream/collect/cancel
```

MVP では一部が直接関数として実装されている。Adapter Kernel 実装時に port
境界へ切り出す。

## 8. エラーハンドリング設計

主要エラー:

- 入力不備: 必須 option 不足、unknown command、unknown Work Item
- 整合性違反: passing verification なしの approval、Project Rule 競合
- 依存障害: runtime 不在、adapter healthcheck failure、filesystem write failure
- 実行失敗: non-zero exit、timeout、canceled、protocol violation
- 解決失敗: required skill 非対応、permission approval 不足

ユーザー向け出力:

- CLI は `error: <message>` を stderr に出す。
- exit code は成功 `0`、失敗 `1` を基本とする。
- 後続では機械可読 output 用に `--json` を追加する。

## 9. セキュリティ設計

Permission Policy は Skill Set と分離する。

Policy が制御するもの:

- 許可 action
- 禁止 action
- 承認が必要な action
- workspace 書き込み境界
- network / dependency install / secrets access

初期 deny:

- main push
- production access
- secrets read
- workspace 外書き込み

Run Packet は選択された Permission Policy を含む。Adapter は実行前に policy
を検査し、違反や未承認 action を `needs_human` に正規化する。

## 10. 観測性

保存するイベント:

- work_item.created
- run.previewed
- run.requested
- run.started
- run.stdout
- run.stderr
- run.artifact.produced
- run.evidence.produced
- run.completed
- run.failed
- verification.completed
- handoff.created
- decision.recorded

各 event には最低限 `who`、`what`、`target`、`when`、`result`、`basis` を
持たせる。Run Packet / Skill Context / Capability Probe の id/hash を保存し、
「何を渡して Agent が動いたか」を後から追跡できるようにする。

## 11. テスト設計

Domain / Core:

- Work Item の status transition
- approval の precondition
- handoff creation
- evidence と artifact の linkage
- ledger load/save

UseCase:

- first scenario が `done` に到達する
- failed run が Evidence を作る
- verification failure が approval をブロックする
- unknown Work Item が NotFound を返す

Adapter:

- healthcheck missing dependency
- process success / failure
- stdout/stderr capture
- artifact collection
- cancel unsupported behavior

CLI:

- help output にはユーザー向け command だけを載せる
- `item run` smoke flow
- npm-installed `nagare` wrapper smoke

## 12. 配布・実行形態

Rust workspace:

- `crates/nagare-core`
- `crates/nagare-cli`

npm package:

- `@hachiware-labs/nagare`
- bin: `nagare`
- prepack: platform binary を build / stage する

配布方針:

- npm は install / distribution path。
- 操作面は常に `nagare` command。
- tutorial も README も `nagare` command で完結させる。
- dev-only helper は docs/help に出さない。

## 13. CLI：コマンド体系 / 引数 / 出力 / exit code

### 現在実装済み

```text
nagare init [--root <path>]
nagare doctor [--root <path>]
nagare locale show [--root <path>]
nagare locale use [--language <locale>] [--timezone <timezone>] [--root <path>]
nagare agent add --id <agent_profile_id> --runtime <runtime_id> --adapter <adapter_id> [--display-name <text>] [--role <role>] [--working-dir <relative_path>] [--description <text>] [--specialties <csv>] [--root <path>]
nagare agent list [--root <path>]
nagare agent show <agent_profile_id> [--root <path>]
nagare agent defaults [--root <path>]
nagare agent use [--work-agent <agent_profile_id>] [--review-agent <agent_profile_id>] [--dispatch-agent <agent_profile_id>] [--root <path>]
nagare agent doctor <agent_profile_id> [--root <path>]
nagare agent probe <agent_profile_id> [--root <path>]
nagare rule check <path> [--agent <agent_profile_id>] [--root <path>]
nagare item create --title <title> [--description <text>] [--root <path>]
nagare item list [--root <path>]
nagare item show <work_id> [--root <path>]
nagare item preview <work_id> [--path <path>] [--agent <agent_profile_id>] [--prompt <text> | --command <command>] [--root <path>]
nagare item dispatch accept <work_id> [--dispatch-plan <dispatch_plan_id>] [--root <path>]
nagare item run <work_id> [--path <path>] [--agent <agent_profile_id>] [--dispatch-plan <dispatch_plan_id>] [--prompt <text> | --command <command>] [--root <path>]
nagare item review <work_id> [--agent <agent_profile_id>] [--prompt <text> | --command <command>] [--root <path>]
nagare verify <work_id> --command <command> [--root <path>]
nagare handoff create <work_id> --from-agent <agent_profile_id> --to-agent <agent_profile_id> --reason <text> [--summary <text>] [--root <path>]
nagare handoff dispatch <work_id> [--agent <agent_profile_id>] [--prompt <text> | --command <command>] [--root <path>]
nagare decision approve <work_id> [--rationale <text>] [--root <path>]
nagare status [--root <path>]
nagare version
nagare help
```

### Agent Management Kernel で追加する

```text
nagare runtime list
nagare runtime doctor
nagare runtime add process --id <runtime_id> --command <command>

nagare adapter list

nagare skill list
nagare skill add --id <skill_set_id> --path <path>
nagare skill show <skill_set_id>

nagare rule list
nagare rule add --id <rule_id> --match <glob> --skill <skill_set_id> --agent <agent_profile_id>
nagare rule check <path>

nagare item preview <work_id> --path <path> [--agent <agent_profile_id>]
nagare item run <work_id> --path <path> [--agent <agent_profile_id>]
```

### 出力方針

- default は人間向けの短い text。
- `--json` は後続で追加し、automation / CI 用に固定 schema を返す。
- 成功時は作成/更新された id と status を必ず出す。
- 失敗時は原因と次の action が分かる文言にする。

### exit code

- `0`: success
- `1`: user input / state / dependency / runtime error
- 後続で必要になった場合のみ、機械可読な error code を JSON に追加する。
