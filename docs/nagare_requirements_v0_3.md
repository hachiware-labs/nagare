# Nagare / 流 要件定義 v0.3

**作成日:** 2026-05-24  
**目的:** Agent時代の「整流する現場看板」として、人間が仕事を置き、Agentが計画・実行・検証し、その証跡・レビュー・引き継ぎ・学習が残るセルフホスト可能なKanbanを定義する。  
**対象:** 初期対応は Codex CLI と Codex App Server に絞る。将来的には他のAgent実行環境を Agent Profile として追加できる構造にするが、Claude Code、HTTP adapter、SDK adapter は初期対象外とする。  
**位置づけ:** Company Agent OSではなく、Agent作業を整流する **Kanban + Execution Ledger + Evidence Board + Agent Registry**。

---

## 0. Executive Summary

このツールは、単なるLinear/Plane/Vibe Kanbanクローンではない。目指すものは次である。

> **Nagare / 流**  
> 人間が作業カードを置き、登録済みAgent Profileで実行し、Plan・Execution・Artifact・Evidence・Verification・Review・Handoff・Workflow Memoryをカードに紐づけて保存する、Agent作業を整流するセルフホスト可能な現場看板。

本ツールは、以下の4つを一体として提供する。

```text
1. UI
   人間が作業・異常・レビュー・承認を見るためのKanban / Timeline / Evidence画面。

2. API
   Agent、外部Bot、CLI、スマホ、SNS連携、別Orchestratorが操作するための機械インターフェース。

3. Skills
   AgentがこのKanbanをどう使うかを学ぶための手順・Rubric・Playbook・JSON Schema・API client群。

4. Core / Execution Ledger
   Work Item、Plan、Agent Run、Artifact、Evidence、Verification、Review、Decision、Memoryを保存する真実の層。
```

最重要の差別化は以下である。

```text
- Cross-runtime:
  初期対応は Codex CLI と Codex App Server に絞る。将来の runtime 追加に備えるが、言語別 SDK adapter は持たない。

- Agent Profile Registry + Work Item Run:
  Agent Profileを登録し、カード単位で「Codex CLIで実行」「Codex App Serverで再実行」のように実行できる。

- Evidence-first:
  Agentの自己申告ではなく、Artifact / Evidence / Verification / Review / Human DecisionでDoneを成立させる。

- Self-hosted / local-first:
  ローカルrepo、git worktree、CLI agent、raw logs、private artifactsを手元で扱える。

- Human Attention / Andon:
  人間は全ログではなく、Needs Human、Failed Verification、Stale Agent、Review Blockerなどの異常だけを見る。

- Workflow Memory → Skills:
  成功・失敗・Rubric・Findingを次回のSkill / Playbook / Rubricへ昇格する。
```

### 0.1 名称と思想: Nagare / 流

本ツールの名称は **Nagare / 流** とする。

ここでいう「流」は、単に作業を速く流すことではない。TPS（トヨタ生産システム）における **整流** の考え方をAgent作業に適用する。Agent作業は、複数Agentの並列実行、成果物と根拠の分離、レビュー待ち、検証失敗、handoff不備、人間のattention不足によって容易に乱流化する。Nagare / 流 は、その乱流を、Work Item、Task Contract、Plan、Execution、Artifact、Evidence、Verification、Review、Handoff、Decision、Memoryへ分解し、流れとして見える化・制御・改善する。

```text
Nagare / 流 が扱う「整流」:
  - 仕事を流す前に、Task ContractとPlanで流路を定める。
  - Agent Run中は、Claim / Lease / Heartbeatで滞留と異常を検知する。
  - 成果物はArtifactとして保存し、判断根拠はEvidenceとして分けて保存する。
  - Doneは自己申告ではなく、Verification / Review / Human Decisionで成立させる。
  - 詰まった仕事はHandoff Packetで次のAgentまたは人間へ流す。
  - 失敗・成功・判断はWorkflow Memoryとして残し、次回のSkill / Playbook / Rubricへ昇格する。
```

そのため、Nagare / 流 は「Agentをただ動かすツール」ではない。
**人とAgentの仕事を、止めるべきところで止め、流すべきところへ流し、証跡で次工程へ渡すための整流板**である。

---

## 1. 背景整理

### 1.1 大企業動向：Agent OS / Control Planeは大手が取りに来る

OpenAIはAgentKitで、Agent workflowの設計・デプロイ・最適化、connector、eval、agentic UIなどを統合する方向を打ち出している[^openai-agentkit]。またSymphonyでは、Linearのようなproject-management boardをCodex orchestrationのcontrol planeに変える実装方針を公開し、issue trackerをAgent実行の中心に置いた[^openai-symphony]。

MicrosoftはAgent 365を「AI agentsのcontrol plane」と位置づけ、Microsoft 365、Entra、Defender、Purviewと結びついたagent registry、governance、security、observabilityを提供している[^ms-agent365][^ms-agent365-blog]。

GoogleはADKとA2Aを通じて、multi-agent system開発、specialized agent間のtransfer、異なるenterprise platform上のAgent同士の通信・協調を支援している[^google-adk-a2a][^google-a2a]。

AnthropicはMCPで外部ツール・データ接続の標準化を進め、Agent Skillsで手順・スクリプト・テンプレート・参考資料をフォルダ単位でAgentに与える方向を示している[^anthropic-skills][^agent-skills-docs]。

このため、以下の領域で正面衝突するのは避けるべきである。

```text
- 汎用Agent builder
- 汎用workflow builder
- 大企業向けAgent governance dashboard
- 企業ID / 権限 / 監査基盤
- Connector marketplace
- 自社SaaS内Agent管理
- モデル提供そのもの
```

本ツールの狙いは、大企業のAgent OSと競争することではなく、**大企業・OSS・ローカルAgentをすべて利用する側の「現場看板」**になることである。

---

### 1.2 現場/Xトレンド：KanbanがAgent作業のUIになりつつある

X周辺の議論、HN、GitHub、公式ブログを見ると、Agent作業の現場は明確にKanban化している。

OpenAI Symphonyは、interactive coding sessionsを直接監督する運用が人間のattention bottleneckになるとし、issue trackerをcontrol planeに変えた。OpenAIはSymphonyについて「Linearのようなproject-management boardをcoding agentsのcontrol planeに変える」と説明している[^openai-symphony]。同記事では、Codex sessionやPRではなく、issue/task/ticketを作業単位にすること、各issueがisolated workspaceを持つこと、CIやreview packetまで扱うことが示されている[^openai-symphony]。

Vibe Kanbanは、Claude Code、Codex、Gemini CLI、OpenCodeなど複数のcoding agentをKanbanとworkspaceで扱う先行例である。READMEでは、今後のsoftware engineerはcoding agentsのplanningとreviewに多くの時間を使うため、planning/reviewを速くすることが重要だと説明している[^vibe-kanban]。一方で、Vibe Kanbanの会社bloopは2026年に終了し、プロジェクトはOSS/community maintainedへ移った。理由として、多くの利用はあったが商用モデルを見つけられなかったことが述べられている[^vibe-shutdown]。これは「単なるAgent Kanban」だけでは差別化・収益化が弱いことを示す。

Cline Kanbanは、Claude CodeやCodexなどを含むCLI-agnosticなmulti-agent orchestration appとして、Agentにlock-inしない「calm, organized layer」を掲げている[^cline-kanban]。Cline CLIの説明でも、terminalまたはKanban boardから複数Agentを並列に動かす方向が明示されている[^cline-cli]。

Hermes Kanbanは、Kanbanを単なるUIではなくdurable multi-agent boardとして設計している。dispatcher、heartbeat、reclaim、zombie detection、auto-block、per-task retriesなどを備え、worker agent profileが実行してもlifecycle truthはKanban kernelが所有する設計になっている[^hermes-release][^hermes-worker-lanes]。

VS Code Agent Kanbanは、taskをMarkdown + YAML frontmatterとして保存し、plan/todo/implement workflowをGit管理可能にすることで、context rot対策をしている[^vscode-agent-kanban][^hn-vscode-agent-kanban]。

これらから得られる現場知見は次である。

```text
- Kanbanは人間向けUIだけでなく、Agent-readable task kernelである。
- 人間のattentionがボトルネックになる。
- Agent非依存・CLI非依存が重要。
- per-card worktree / workspace分離が必要。
- Plan承認とExecutionを分けるべき。
- Heartbeat / reclaim / zombie detection / timeoutが必要。
- DoneはAgent自己申告ではなく、Verification + Review + Decisionで成立させるべき。
- Handoffは自然文コメントではなく、summary + structured metadataにすべき。
- raw logsはDBではなくappend-only file / object storeに逃がすべき。
```

---

### 1.3 論文知見：Agentを増やすより、計画・検証・証跡・記憶が重要

Multi-Agent LLM Systemsの失敗分析であるMASTは、multi-agent systemの失敗を大きく `(i) specification and system design failures`, `(ii) inter-agent misalignment`, `(iii) task verification and termination` に分類している[^mast]。これはNagare / 流 が、単なる進捗表ではなく、Task Contract、Role、Handoff、Verification、Terminationを保存する必要があることを示す。

VeriMAPは、multi-agent collaborationにおいて、plannerがタスク分解、subtask dependency、passing criteriaをverification functionとして持たせることで、single-agentおよびmulti-agent baselineを上回り、robustnessとinterpretabilityを高めたと報告している[^verimap]。これは、カードにPlan、Subtask Graph、expected artifact、verification functionを持たせる要件につながる。

Agent Workflow Memoryは、過去経験からreusable workflowを抽出し、将来タスクに再利用することでlong-horizon taskに役立てるフレームワークである[^awm]。これは、完了カードからSkill / Playbook / Rubric改善へつなぐ要件になる。

Trace-Based Assuranceの研究は、Agentic AIの実行をMessage-Action Traceとしてinstrumentし、step/trace contracts、deterministic replay、stress testing、runtime governanceを扱う枠組みを提案している[^trace-assurance]。これは、Nagare / 流 が「ログ保存」ではなく、contract、evidence、verification、policy decisionを含むExecution Ledgerを持つべき理由になる。

Overeager Coding Agentsは、Claude Code、OpenHands、Codex CLI、Gemini CLIなどのcoding agentが、良性タスクでも依頼範囲外の行動をする問題を測定している。許可範囲の明示を外すとovereager rateが上がることが示されており、scope / allowed actions / disallowed actions / approval gatesをカード側に持つ必要がある[^overeager]。

Single-Agent with Skillsの研究は、multi-agent systemの一部はsingle agent + skill libraryへ置き換えられ、token usageやlatencyを下げながら競争的精度を保てる場合がある一方、skill libraryの増大・類似skillによる選択性能低下が課題になるとする[^single-agent-skills]。したがって本ツールは、常にmulti-agent化するのではなく、カードごとに `single_agent`, `single_agent_with_skills`, `multi_agent_parallel`, `multi_agent_sequential`, `human_in_the_loop` を選択できるべきである。

OpenTelemetry GenAI semantic conventionsは、LLM call、tool call、provider metadata、agent spansなどの標準化を進めているが、現時点ではtelemetry substrateであり、業務上のEvidence / Verification / Review / Human Decisionまでは表現しきれない[^otel-genai]。本ツールはOTel exportに対応しつつ、独自のEvidence Ledgerを持つ必要がある。

---

## 2. プロダクト定義

### 2.1 一文定義

**Nagare / 流 は、人間が仕事を定義し、登録済みAgent Profileで実行し、Plan・Execution・Artifact・Evidence・Verification・Review・Handoff・Workflow Memoryをカード単位で保存する、Agent作業を整流するセルフホスト可能な現場看板である。**

---

### 2.2 何であるか

```text
- Agent作業のKanban
- Agent実行のExecution Ledger
- Agent作業のEvidence Board
  - 登録済みAgent ProfileでWork Itemを実行する基盤
- Agentやautomationが読むAPI
- Agentが従うSkills / Playbook bundle
- ローカルworktree / CLI / CI / desktopも扱う現場看板
```

---

### 2.3 何ではないか

```text
- Company Agent OSそのもの
- 大企業向けID/RBAC/governance dashboard
- 汎用Agent builder
- 汎用workflow builder
- 汎用model router
- Vibe Kanban clone
- Linear clone
- ただのterminal manager
- ただのtrace viewer
```

---

## 2.4 プラットフォーム環境 / 実装方針

Nagare / 流 は、最初から大規模SaaSとしてではなく、**local-first / self-hosted single node** を第一ターゲットにする。個人・小規模チームがローカルrepo、git worktree、CLI Agent、raw logs、private artifactsを手元で扱えることを優先する。

### 2.4.1 デフォルトDB

デフォルトDBは **SQLite** とする。

```text
default:
  SQLite

optional / future:
  PostgreSQL

large artifacts:
  filesystem default
  MinIO optional
```

SQLiteをデフォルトにする理由は次である。

```text
- Docker Composeなしでも単体起動しやすい。
- ローカル常駐daemon / CLI / desktop連携と相性がよい。
- Work Item、Plan、Agent Run、Evidence、Review、Decisionなどの構造化データには十分である。
- raw logs、screenshots、videos、large tracesはDBではなくfilesystem / object storeへ逃がす設計にするため、DB肥大化を避けられる。
- 将来的にPostgreSQLへ移行しやすいよう、DB access layerを抽象化する。
```

### 2.4.2 言語・ランタイム構成

Nagare / 流 の推奨構成は、**Rust core + TypeScript UI + Python extension** とする。

```text
Core / CLI / daemon:
  Rust

API server:
  Rust + axum

Async runtime:
  tokio

DB access:
  sqlx or equivalent SQLite-first layer

Web UI:
  TypeScript + React

Agent Skill bundle:
  Markdown + JSON Schema + TypeScript client + Python examples

Verification / analysis extensions:
  Python optional
```

### 2.4.3 RustをCoreに採用する理由

Nagare / 流 の中核は、Web UIではなく、Agent Profileを登録し、Work Itemのpreview/runを行い、workspaceを準備し、heartbeatを監視し、artifact / evidence / verification / reviewを台帳化する実行制御である。ここはRustを第一候補とする。

```text
Rustが担う領域:
  - nagare CLI
  - local daemon
  - API server
  - runner
  - worker supervision
  - claim / lease / heartbeat / timeout
  - git worktree / filesystem / process management
  - SQLite ledger access
  - event stream
```

Rust採用の狙いは次である。

```text
- 単体バイナリとして配布しやすい。
- Windows / macOS / Linuxに展開しやすい。
- ローカル常駐daemonやCLI体験が強い。
- プロセス管理、ファイル管理、git worktree操作に向く。
- 将来、有償・クローズド配布を行う場合も扱いやすい。
```

### 2.4.4 TypeScriptをUI / Skill clientに採用する理由

UI、JSON Schema、OpenAPI client、Agent Skill bundleはTypeScriptとの相性が高い。

```text
TypeScriptが担う領域:
  - Web UI / Board / Card Detail / Timeline / Andon
  - API client SDK
  - Skill bundle用client script
  - schema validation helper
  - optional desktop/web frontend
```

### 2.4.5 Pythonの位置づけ

Pythonは本体Coreではなく、拡張・検証・分析用途に置く。

```text
Pythonが担う領域:
  - Verification script
  - LLM judge / evaluator
  - evidence extractor
  - data analysis / report generation
  - Skill examples
  - domain-specific worker
```

PythonをCoreにしない理由は、Nagare / 流 がローカル常駐・CLI配布・プロセス監視・workspace制御を中核とするためである。ただし、Agent / ML / 検証エコシステムとの接続ではPythonを積極的に使う。

### 2.4.6 初期コマンド案

```text
nagare init
nagare serve
nagare ui
nagare agent add
nagare agent list
nagare agent show
nagare item preview
nagare item run
nagare status
nagare verify
nagare evidence add
nagare handoff create
nagare export skill
```

### 2.4.7 初期ディレクトリ構成案

```text
nagare/
  crates/
    nagare-cli/
    nagare-core/
    nagare-api/
    nagare-runner/
    nagare-adapters/
  ui/
    web/
  skills/
    nagare-core/
      SKILL.md
      schemas/
      playbooks/
      rubrics/
      scripts/
  examples/
  docs/
  migrations/
```

MVP 0/1 の project-local Agent Profile は `.nagare/agents/*.toml` に保存する。

---

## 3. 主要コンセプト

### 3.1 UI + API + Skills + Core

```text
                 ┌──────────────────────────┐
                 │ Human UI                  │
                 │ Web / Mobile / TUI / SNS  │
                 └─────────────▲────────────┘
                               │
┌──────────────────────────────┴──────────────────────────────┐
│ Agent Kanban Core / Execution Ledger                         │
│ Work Item / Plan / Run / Artifact / Evidence / Review / Memory│
└──────────────────────────────┬──────────────────────────────┘
                               │
          ┌────────────────────┴────────────────────┐
          │                                         │
┌─────────▼──────────┐                    ┌─────────▼──────────┐
│ API / Webhook       │                    │ Skills / Playbooks  │
│ Agent interface     │                    │ Agent instructions  │
└────────────────────┘                    └────────────────────┘
```

- **UI**: 人間が状況把握、Plan承認、レビュー、差し戻し、停止、Handoffを行う。
- **API**: Agent、外部Bot、スマホ、SNS連携、別OrchestratorがKanbanを操作する。
- **Skills**: AgentがKanban APIをどう使い、どうPlanし、どうEvidenceを残すかを学ぶ。
- **Core / Ledger**: 状態、証跡、成果物、レビュー、判断、記憶の真実を保存する。

---

### 3.2 Kanbanが状態の真実を持つ

AgentやOrchestratorは状態を所有しない。

```text
正:
  Kanban Core / Ledger が状態を持つ。
  Orchestrator / Agent は状態を読んで、イベント・成果物・Evidenceを登録する。

誤:
  Orchestrator内部状態やAgentセッションだけが真実になる。
```

Hermes Kanbanも、worker agent profilesが実行してもlifecycle truthはKanban kernelが所有すると明示している[^hermes-worker-lanes]。

---

### 3.3 Agentは登録してWork Item単位で実行する

本ツールでは、まず Codex CLI と Codex App Server を **Agent Profile** として登録し、Work ItemまたはSubtaskを実行できるようにする。Shell は verification runner であり Agent Profile ではない。Claude Code、HTTP adapter、SDK adapter は初期対象外とする。

```text
Run with Codex
Retry with Codex App Server
```

どのAgentで、どのversion/model/skills/workspace/permissionsで実行したかを必ず記録する。

---

### 3.4 Doneは自己申告ではない

```text
Agentが「終わりました」と言った
  ≠ Done

Done =
  Required Verification passed
  + Required Review approved
  + Blocking Findings resolved or accepted
  + Human Decision or policy approval recorded
```

---

## 4. Agent Runtime対応方針

### 4.1 Codex

Codexは複数の入口を持つ。

- `codex exec`: scripted/CI-style runs向けのnon-interactive command。`--cd`でworkspace rootを指定でき、stdin promptも受け取れる[^codex-exec]。
- `codex app-server`: Codex App Serverを起動する。既定の `stdio://` では JSONL-over-stdio の双方向 protocol として使え、thread / turn / approval / event stream を扱える[^codex-app-server][^codex-cli-app-server]。
- `codex cloud exec`、Responses API なども存在するが、初期 adapter では扱わない。Codex MCP Server は Nagare の Agent adapter として使わない。

対応agent profile:

```text
codex_cli:
  run_mode: spawn_per_item
  command: codex exec

codex_app_server:
  run_mode: stdio_app_server
  command: codex app-server --listen stdio://

```

---

### 4.2 Hermes

Hermes Kanbanは、taskをSQLite row、handoffもrow、workerをOS processとして扱い、dispatcherがready taskをclaim/spawnし、workerは `kanban_*` toolでtaskを読み書きする[^hermes-kanban]。workerには環境変数でtask/workspace/run idなどが渡される[^hermes-worker-lanes]。structured handoff metadataやheartbeat、protocol violation、crash/timed_out/gave_upなども扱う[^hermes-handoff][^hermes-worker-lanes].

対応agent profile:

```text
hermes_profile_agent:
  run_mode: native_kanban
  command: hermes -p <profile> chat -q <prompt>

hermes_orchestrator:
  plan / create subtasks / link dependencies

hermes_external_agent:
  non-Hermes external worker via plugin spawn_fn
```

---

## 5. コアデータモデル

### 5.1 Work Item

人間が見る作業カード。

```yaml
work_item:
  id: work_123
  title: "認証mockの失敗テストを修正する"
  description: "CIで落ちているauth関連テストを調査し、修正する"
  status: ready_for_agent
  project_id: proj_auth
  owner_user_id: user_kit
  preferred_agent_profile: codex_cli_impl
  allowed_agent_profiles:
    - codex_cli_impl
    - codex_app_server_review
    - codex_app_server_impl
  priority: high
  labels: [bug, auth]
  acceptance_criteria:
    - "npm test が成功する"
    - "public APIを変更しない"
    - "PRに原因と修正内容を記載する"
```

---

### 5.2 Task Contract

Agentに渡す作業契約。

```yaml
task_contract:
  id: contract_123
  work_item_id: work_123
  goal: "auth関連テスト失敗の原因を特定し、最小変更で修正する"
  context:
    repo: "github.com/org/auth-service"
    branch: "main"
    related_files:
      - "src/auth/*"
      - "tests/auth/*"
  constraints:
    - "public APIを変更しない"
    - "snapshot更新だけで済ませない"
    - "本番DBへ接続しない"
  allowed_actions:
    - repo_read
    - branch_write
    - test_run
  disallowed_actions:
    - production_access
    - main_push
    - secrets_read
  success_criteria:
    - type: command
      command: "npm test"
      expected: "exit_code_0"
    - type: review
      reviewer: "human"
      criterion: "修正理由が説明されている"
  risk_level: medium
```

---

### 5.3 Plan

実行前に保存・レビューされる計画。

```yaml
plan:
  id: plan_001
  work_item_id: work_123
  version: 1
  created_by: planner_agent
  assumptions:
    - "失敗はauth mockの初期化順序に関係している可能性"
  steps:
    - id: step_1
      title: "失敗テストを再現する"
      expected_artifact: "test_log"
      verification:
        type: command
        command: "npm test -- auth"
    - id: step_2
      title: "関連実装を調査する"
      expected_artifact: "root_cause_note"
    - id: step_3
      title: "最小修正を行う"
      expected_artifact: "diff"
    - id: step_4
      title: "検証する"
      expected_artifact: "verification_result"
  risk_gates:
    - condition: "public API変更が必要"
      action: "needs_human"
```

---

### 5.4 Subtask Graph

DAG / dependency / claim可能な実行単位。

```yaml
subtask:
  id: sub_001
  work_item_id: work_123
  plan_id: plan_001
  title: "失敗テストを再現する"
  status: ready
  depends_on: []
  claim:
    claimed_by: null
    lease_until: null
```

---

### 5.5 Agent Profile

登録済みAgentの実行agent profile。

```yaml
agent_profile:
  id: codex_cli_impl
  display_name: "Codex CLI Implementer"
  provider: codex
  kind: spawn_per_task # spawn_per_task | persistent_server | cloud_task | pull_worker
  roles:
    - implementer
    - fixer
  capabilities:
    - code_edit
    - test_run
    - repo_analysis
  run:
    mode: cli
    command: "codex exec"
    working_dir: "."
  workspace_policy:
    type: git_worktree
    isolate_per_work_item: true
  permission_policy_id: policy_medium_code_task
  skills:
    - nagare-core
    - bugfix-playbook
  limits:
    max_parallel_runs: 2
    timeout_minutes: 60
    max_retries: 2
  healthcheck:
    command: "codex --version"
```

`working_dir` はproject rootからの相対pathであり、Agent processを開始する
ディレクトリを示す。Agent Profile定義ファイルの保存場所とは別物である。

---

### 5.6 Workspace

Agent実行の隔離単位。

```yaml
workspace:
  id: ws_001
  work_item_id: work_123
  kind: worktree # worktree | scratch | dir | container | vm
  path: "/worktrees/work_123"
  branch: "agent/work_123"
  base_ref: "main@abc123"
  dirty_state: clean
  cleanup_policy: keep # keep | archive | delete
```

---

### 5.7 Run Request

Work Item/SubtaskをAgent Profileで実行するための記録。

```yaml
run_request:
  id: req_001
  work_item_id: work_123
  subtask_id: sub_001
  agent_profile_id: codex_cli_impl
  run_mode: spawn_per_item
  requested_by:
    type: user
    id: user_kit
  reason: "コード修正とテスト実行に強いAgentとして選択"
  run_packet_id: runpkt_001
  status: queued # queued | starting | running | failed | completed | canceled
  created_at: "2026-05-24T10:00:00+09:00"
```

---

### 5.8 Run Packet

Agentへ渡す構造化入力。単なるpromptではない。

```yaml
run_packet:
  id: runpkt_001
  work_item_id: work_123
  goal: "auth関連テスト失敗の原因を特定し、最小修正で直す"
  workspace:
    kind: worktree
    path: "/worktrees/work_123"
    branch: "agent/work_123"
  constraints:
    - "public APIを変更しない"
    - "mainにpushしない"
    - "production credentialsを要求しない"
  expected_artifacts:
    - diff
    - test_log
    - summary
  verification:
    - command: "npm test"
      expected: "exit_code_0"
  reporting:
    submit_artifacts_to: "kanban_api"
    submit_evidence_to: "kanban_api"
```

---

### 5.9 Execution Attempt

同じWork Itemに対する1回の実行試行。

```yaml
execution_attempt:
  id: exec_001
  work_item_id: work_123
  task_contract_id: contract_123
  plan_id: plan_001
  state: running
  execution_mode: single_agent_with_skills
  started_at: "2026-05-24T10:00:00+09:00"
  ended_at: null
  workspace_id: ws_001
  current_summary: "失敗テストの再現に成功。mock初期化順序を調査中。"
```

---

### 5.10 Agent Run

具体的なAgent実行。

```yaml
agent_run:
  id: run_001
  work_item_id: work_123
  execution_attempt_id: exec_001
  run_request_id: req_001
  agent_profile_id: codex_cli_impl
  provider: codex
  agent_name: "Codex CLI"
  agent_version: "..."
  model: "gpt-5-codex"
  role: implementer
  run_mode: spawn_per_item
  run_packet_hash: "sha256:..."
  skills_used:
    - nagare-core
    - bugfix-playbook
  workspace_id: ws_001
  permission_policy_id: policy_medium_code_task
  state: running # queued | starting | running | waiting | failed | succeeded | canceled | timed_out
  started_at: "..."
  ended_at: null
  outcome_summary: null
```

---

### 5.11 Tool Call / Process Event

```yaml
tool_call:
  id: tool_021
  agent_run_id: run_001
  type: shell # shell | mcp | browser | desktop | api | file_edit
  name: "npm test"
  input_summary: "auth関連テストを実行"
  input_hash: "sha256:..."
  output_artifact_id: art_testlog_001
  status: failed
  started_at: "..."
  ended_at: "..."
```

---

### 5.12 Artifact

物としての成果物。

```yaml
artifact:
  id: art_001
  work_item_id: work_123
  execution_attempt_id: exec_001
  agent_run_id: run_001
  type: diff # diff | pr | test_log | screenshot | video | report | document | diagram
  uri: "file:///artifacts/work_123/pr_812.diff"
  title: "PR #812 diff"
  provenance:
    agent_run_id: run_001
    tool_call_id: tool_021
  verification_status: unverified
```

---

### 5.13 Evidence

状態判断の根拠。

```yaml
evidence:
  id: ev_001
  work_item_id: work_123
  claim: "npm test が失敗した"
  basis:
    type: command_result
    command: "npm test"
    exit_code: 1
    artifact_id: art_testlog_001
  produced_by:
    type: ci
    id: gha_042
  confidence: high
  created_at: "..."
```

---

### 5.14 Verification Result

Acceptance CriteriaやPlan stepを満たしたか。

```yaml
verification_result:
  id: ver_001
  work_item_id: work_123
  execution_attempt_id: exec_001
  criterion: "npm test exit_code_0"
  method: command
  command: "npm test"
  result: failed
  evidence_id: ev_001
  failure_summary: "3 tests failed in auth/session.test.ts"
  verified_at: "..."
```

---

### 5.15 Rubric

成果物を評価する基準。

```yaml
rubric:
  id: rubric_code_review_v1
  name: "コードレビュー基準"
  version: 1
  criteria:
    - id: correctness
      name: "正しさ"
      scale: pass_fail
      blocking: true
    - id: minimality
      name: "変更の最小性"
      scale: score_1_5
      threshold: 4
      blocking: false
    - id: test_coverage
      name: "テスト妥当性"
      scale: score_1_5
      threshold: 4
      blocking: true
    - id: security
      name: "セキュリティ"
      scale: pass_fail
      blocking: true
```

---

### 5.16 Review Request / Review Result / Finding

```yaml
review_request:
  id: rr_001
  work_item_id: work_123
  target:
    type: artifact
    id: art_pr_812
  rubric_id: rubric_code_review_v1
  requested_reviewers:
    - type: agent_profile
      id: codex_app_server_review
    - type: human
      id: user_kit
  required:
    human_approval: true
    agent_review: true
  status: pending
```

```yaml
review_result:
  id: review_001
  work_item_id: work_123
  target:
    type: artifact
    id: art_pr_812
  rubric_id: rubric_code_review_v1
  reviewer:
    type: agent_profile
    id: codex_app_server_review
  overall:
    result: needs_changes
    confidence: medium
    summary: "修正方針は妥当だが、認可エラー時のテストが不足している"
  scores:
    - criterion_id: correctness
      result: pass
    - criterion_id: test_coverage
      result: fail
    - criterion_id: security
      result: pass
```

```yaml
finding:
  id: finding_001
  review_result_id: review_001
  severity: major # blocker | major | minor | nit
  category: test_gap
  title: "認可失敗ケースのテスト不足"
  location:
    file: "tests/auth/session.test.ts"
    line: 42
  recommendation: "403ケースのテストを追加する"
  status: open # open | accepted | fixed | dismissed
  blocking: true
```

---

### 5.17 Handoff Packet

Agent間・人間間の構造化引き継ぎ。

```yaml
handoff_packet:
  id: handoff_001
  work_item_id: work_123
  from:
    type: agent_profile
    id: codex_cli_impl
  to:
    type: agent_profile
    id: codex_app_server_review
  reason: "テスト失敗の原因分析が必要"
  current_state: failed_verification
  summary: "auth mock setupを修正したが、3件のテストがまだ失敗している"
  prior_attempts:
    - exec_001
  artifacts:
    - art_diff_001
    - art_testlog_001
  evidence:
    - ev_failed_test_001
  open_questions:
    - "mockの初期化順序は仕様上正しいか"
  requested_output:
    - "原因候補"
    - "修正方針"
    - "リスク"
  verification_needed:
    - "npm test"
```

Hermesの設計でも、handoffはhuman-readable summaryとmachine-readable metadataに分け、下流Agentやreviewerがproseをスクレイプせずに読めるようにすることが推奨されている[^hermes-handoff]。

---

### 5.18 Human Decision

```yaml
human_decision:
  id: dec_001
  work_item_id: work_123
  actor_user_id: user_kit
  decision_type: approve # approve | reject | request_changes | pause | delegate | override
  target:
    type: artifact
    id: art_pr_812
  rationale: "修正範囲が限定的で、テストも通っているため承認"
  created_at: "..."
```

---

### 5.19 Workflow Memory / Skill Candidate

```yaml
workflow_memory:
  id: mem_001
  scope:
    type: repo
    repo: "auth-service"
  trigger:
    task_type: "auth test failure"
  lesson: "auth mockの初期化順序変更時はintegration testも実行する"
  evidence:
    - work_123
    - ver_001
    - finding_001
  suggested_playbook_id: playbook_auth_tests
```

---

## 6. 状態モデル

### 6.1 Work Item Status

```text
backlog
ready
specifying
planning
plan_review
ready_for_agent
claimed
agent_running
waiting_for_tool
waiting_for_agent
blocked
needs_human
needs_handoff
verifying
failed_verification
ready_for_review
reviewing
changes_requested
approved
done
canceled
archived
```

### 6.2 Agent Run Status

```text
queued
starting
running
heartbeat_missing
waiting
tool_calling
needs_input
failed
succeeded
canceled
timed_out
crashed
gave_up
```

### 6.3 代表遷移

```text
ready
  → planning
  → plan_review
  → ready_for_agent
  → claimed
  → agent_running
  → verifying
  → ready_for_review
  → reviewing
  → approved
  → done
```

検証失敗:

```text
agent_running
  → verifying
  → failed_verification
  → needs_handoff
  → ready_for_agent
```

レビュー差し戻し:

```text
ready_for_review
  → reviewing
  → changes_requested
  → ready_for_agent
```

Agent停止:

```text
agent_running
  → heartbeat_missing
  → timed_out
  → needs_human
```

---

## 7. 機能要件

### FR-01: Kanban Board

**MUST**

- Work ItemをKanban形式で表示する。
- status、project、label、human owner、agent profile、risk、verification stateでfilterできる。
- Failed Verification、Needs Human、Stale Agent、Review Blockerを強調表示する。
- カード上にhuman ownerとdelegated agentを分けて表示する。

**SHOULD**

- WIP limitを設定できる。
- Agent別・Agent Profile別の稼働状況を表示する。
- Agent Andon viewを持つ。

---

### FR-02: Task Contract

**MUST**

- goal、context、constraints、allowed/disallowed actions、success criteria、risk levelを持つ。
- Agentに渡す前に人間が確認・修正できる。
- Scope逸脱を避けるため、authorized scopeを明示する。

---

### FR-03: Plan-first Workflow

**MUST**

- Planを第一級オブジェクトとして保存する。
- Planのversion履歴を持つ。
- Plan stepごとにexpected artifactとverification methodを持つ。
- Plan Review状態を持つ。

**SHOULD**

- PlanをPlaybookへ昇格できる。
- Plan差分を表示できる。
- VeriMAP型のsubtask verification functionをサポートする。

---

### FR-04: Agent Registry

**MUST**

- Agent Profileを登録できる。
- Project-local Agent Profileを `.nagare/agents/*.toml` に保存できる。
- Nagare本体が使う `work_agent`, `review_agent`, `dispatch_agent` を設定できる。
- Project localeを設定でき、Nagareが生成する記録にlocaleを保存できる。
- Agent Profileはprovider、run mode、roles、capabilities、workspace policy、permission policy、skills、limits、healthcheckを持つ。
- Codex CLI と Codex App Server を初期agent profile候補とする。
- Shell は verification runner であり、Agent Profile ではない。
- Claude Code と HTTP adapter は初期対象外とする。

**SHOULD**

- Agent Profileごとの成功率、失敗率、レビュー差し戻し率、平均コスト、平均時間を集計する。
- Agent Profileをtag/rubric/skillで分類できる。

---

### FR-05: Work Item Run

**MUST**

- Work ItemまたはSubtaskを指定Agent Profileで実行できる。
- Agent Profileが明示されない場合は `work_agent` を既定実行先として使える。
- 実行前に `item preview` でAgent Profile、Project Rule、Skill Set、Policy、Verification、Run Packetを確認できる。
- Run Request recordを作成する。
- Run Packetを生成し、Agentへ渡す。
- run modeとして `spawn_per_item`, `persistent_server`, `cloud_task`, `pull_worker` を扱える設計にする。

**SHOULD**

- 同じWork Itemを複数Agentで実行し比較できる。
- Retry時にsame agent / different agentを選べる。
- Handoffから別Agentで再実行できる。

---

### FR-06: Claim / Lease / Heartbeat

**MUST**

- Agent Runにclaim / lease / heartbeatを持つ。
- heartbeat timeout時にstale/timed_outへ遷移する。
- cancel / pause / resume / retry / reclaimをサポートする。
- Agentが正常終了してもrequired terminatorを呼んでいなければprotocol violationとして扱う。

---

### FR-07: Workspace Isolation

**MUST**

- Work Itemごとにworkspaceを持てる。
- git worktreeを第一級workspaceとして扱う。
- branch、base_ref、dirty_state、cleanup_policyを記録する。

**SHOULD**

- container/VM workspaceへ拡張できる。
- diff preview、PR link、preview server linkを持てる。

---

### FR-08: Artifact / Evidence

**MUST**

- diff、PR、test log、screenshot、video、report、documentをArtifactとして保存またはリンクする。
- Artifactのprovenanceとしてagent_run_id、tool_call_id、workspace_idを保存する。
- 状態判断の根拠をEvidenceとして保存する。
- 「なぜこのカードがこの状態なのか」を表示する。

---

### FR-09: Verification Gate

**MUST**

- Done前にVerification Resultを確認する。
- command、CI、schema check、LLM judge、human checkを検証方法として持つ。
- 検証失敗時はfailed_verificationへ遷移する。

**SHOULD**

- Verification failureからHandoff Packetを自動生成する。
- Required / optional verificationを分ける。

---

### FR-10: Rubric / Review / Finding

**MUST**

- Rubricを第一級オブジェクトとして持つ。
- Review Request、Review Result、Findingを持つ。
- Findingにはseverity、category、location、recommendation、status、blockingを持つ。
- Review Resultは対象Artifact、Rubric、Reviewer、Evidenceに紐づく。

**SHOULD**

- Review ResultからSubtaskを生成できる。
- 複数reviewerの結果をaggregateできる。
- Review結果からRubric改善候補を作れる。

---

### FR-11: Handoff Packet

**MUST**

- Agent → Agent、Agent → Human、Human → Agentのhandoffを構造化して保存する。
- reason、current state、summary、artifacts、evidence、open questions、requested output、verification neededを含む。
- Handoffから新しいRun Requestを作成できる。

---

### FR-12: Human Decision

**MUST**

- approve、reject、request_changes、pause、delegate、overrideを保存する。
- 誰が、何を、なぜ判断したかを保存する。
- Human ownerを常に表示する。

---

### FR-13: API / Webhook

**MUST**

REST/GraphQL API:

```text
GET    /work-items/ready
POST   /work-items/{id}/claim
POST   /work-items/{id}/preview
POST   /work-items/{id}/runs
POST   /work-items/{id}/plans
POST   /work-items/{id}/artifacts
POST   /work-items/{id}/evidence
POST   /work-items/{id}/verification-results
POST   /work-items/{id}/review-requests
POST   /review-requests/{id}/results
POST   /work-items/{id}/handoffs
POST   /work-items/{id}/decisions
GET    /events/stream
```

**SHOULD**

- A2A adapterを持ち、A2A Task / Artifactを内部Work Item / Artifact / Handoffへ正規化できる。
- Webhook/SSEで外部UI、Slack、mobile、SNS連携を可能にする。

---

### FR-14: Skills形式

**MUST**

Nagare Skill Bundleを提供する。

```text
skills/
  nagare-core/
    SKILL.md
    schemas/
      task_contract.schema.json
      plan.schema.json
      evidence.schema.json
      handoff_packet.schema.json
      review_result.schema.json
    playbooks/
      bugfix.md
      code_review.md
      failed_verification_recovery.md
    rubrics/
      code_review.yaml
      security_review.yaml
    scripts/
      kanban_client.ts
      submit_evidence.py
    examples/
      bugfix_handoff.json
      review_result.json
```

`SKILL.md`は以下をAgentへ教える。

```text
1. ReadyなWork Itemを探す。
2. 作業前にclaimする。
3. Task Contractを読む。
4. Planが必要ならPlanを提出する。
5. Plan Review前に大きな実装を始めない。
6. 作業中はheartbeatを送る。
7. ArtifactとEvidenceを登録する。
8. Verification Resultを提出する。
9. Review Requestを作る。
10. 詰まったらHandoff Packetを作る。
11. DoneにするのはKanban側のVerification / Review / Decision後である。
```

---

### FR-15: Workflow Memory

**MUST**

- 完了カードからlesson / playbook候補を作成できる。
- Review Finding、Verification failure、Human DecisionをMemoryに紐づける。

**SHOULD**

- 類似タスク作成時にPlaybook / Skillを推薦する。
- Workflow MemoryからSkill Bundleを生成できる。
- `AGENTS.md`, `WORKFLOW.md`, `SKILL.md`へexportできる。

---

## 8. 非機能要件

### NFR-01: Self-host / Local-first

- 単体バイナリ + SQLiteで起動できる。
- Docker Composeでも起動できる。
- デフォルトではSQLiteに構造化データを保存する。
- PostgreSQLはoptional / future migration targetとする。
- raw logs、screenshots、videos、large tracesはfilesystemまたはMinIOへ保存する。
- 外部SaaSなしでも動作する。
- GitHub / Linear / Plane / Beads / CIなどはadapterとして扱う。

### NFR-02: Logs and Storage

Vibe Kanbanはappend-only agent logsをSQLiteからfilesystemへ移した理由として、logs tableがrelational queryの恩恵を受けないappend-only JSONLだったことを述べている[^vibe-logs]。本ツールも以下に分ける。

```text
PostgreSQL:
  Work Item, State, Relations, Evidence, Review, Decisions, Events index

Object/File Store:
  raw stdout/stderr, screenshots, videos, large transcripts, raw traces

Search Index:
  summary, metadata, evidence text, finding text
```

### NFR-03: Security

- Agent Profileごとのpermission policyを持つ。
- dangerous actionsはhuman approval requiredにする。
- secrets / production credentialsはAgentへ渡さない。
- worktree外変更、main push、production accessなどをdenyできる。
- raw logsにsecretを保存しない。
- metadataにはsecret/token/raw unrelated transcriptsを入れない。

### NFR-04: Observability

- execution_id / trace_idを持つ。
- OTel GenAI exportに対応できる設計にする。
- Agent Run、Tool Call、Artifact、Evidence、Verification、Reviewをtimelineで追える。
- raw trace、structured trace、human summary、evidenceを分ける。

### NFR-05: Portability

- Agent、UI、External toolsをadapter化する。
- Linear/Plane/GitHub/Beadsを置き換え可能にする。
- APIとSkillで外部frontendsを可能にする。
- schema export/importを持つ。

---

## 9. UI要件

### 9.1 Board View

列の初期案:

```text
Backlog
Ready
Planning
Plan Review
Ready for Agent
Running
Needs Human
Failed Verification
Ready for Review
Changes Requested
Approved
Done
```

カード表示項目:

```text
- title
- status
- human owner
- delegated agent profile
- current run
- last evidence
- verification status
- review status
- risk level
- next action
```

### 9.2 Card Detail

タブ:

```text
Overview
Contract
Plan
Subtasks
Agent Profile
Workspace
Runs
Artifacts
Evidence
Verification
Rubrics
Reviews
Findings
Handoffs
Decisions
Memory
Events
```

### 9.3 Agent Andon View

異常のみを出す。

```text
Failed Verification
Needs Human
Stale Agent
Heartbeat Missing
Scope Violation
Missing Evidence
Review Blocker
Handoff Required
Protocol Violation
```

### 9.4 Timeline View

```text
10:00 Work Item created
10:03 Plan proposed
10:05 Human approved plan
10:06 Codex run started
10:24 Diff produced
10:28 npm test failed
10:29 Verification failed
10:30 Handoff packet created
10:32 Reviewer agent run started
```

### 9.5 Mobile / SNS View

- Needs Humanだけ通知する。
- Approve / Request Changes / Pause / Handoffをボタンで行える。
- Evidence summaryだけを見せ、raw logsは深掘り時のみ開く。

---

## 10. 差別化戦略

### 10.1 大企業と戦わない

大企業は以下を取る。

```text
- モデル
- Agent builder
- クラウド実行
- 企業ID / security / governance
- 自社SaaS統合
- marketplace
```

本ツールは以下を取る。

```text
- cross-runtime execution
- self-hosted / local-first
- Agent Profile Registry
- Work Item preview/run
- run provenance
- evidence-first completion
- verification / review / rubric
- human attention / Andon
- workflow memory → skills
```

### 10.2 強くするべき5機能

1. **Agent Profile Registry + Work Item Run**  
   Codex CLI と Codex App Server を登録し、カード単位で実行・レビュー・再試行・handoffできる。Codex App Server をサーバー型入口として優先し、Claude Code、HTTP adapter、SDK adapter は初期対象外とする。

2. **Evidence / Verification / Review Pipeline**  
   Artifact、Evidence、Verification Result、Rubric、Review Result、Finding、Human Decisionを中核にする。

3. **Worktree / Workspace Isolation**  
   per-card workspaceで安全に並列Agent実行する。

4. **UI + API + Skills**  
   人間はUI、AgentやautomationはAPI、Agent教育はSkillsで行う。

5. **Agent Andon**  
   人間は全ログではなく、異常・承認待ち・検証失敗だけを見る。

---

## 11. MVP Roadmap

### MVP 1: Core Kanban + Ledger

- Work Item
- Task Contract
- Plan
- Execution Attempt
- Agent Run
- Artifact
- Evidence
- Verification Result
- Review Result
- Finding
- Human Decision
- Basic Board UI
- Docker Compose self-host

### MVP 2: Agent Registry + Work Item Run

- Agent Profile登録
- Codex CLI agent profile
- Codex App Server agent profile
- Codex App Server stdio adapter
- Run Request / Claim / Heartbeat / Timeout
- Workspace / git worktree管理

### MVP 3: Review / Rubric / Andon

- Rubric管理
- Review Request / Result
- Finding管理
- Agent Andon view
- Failed Verification → Handoff生成

### MVP 4: API / Skills

- REST API
- Event stream
- nagare-core Skill Bundle
- JSON Schema / client scripts

### MVP 5: Workflow Memory

- 完了カードからlesson抽出
- Playbook登録
- 類似タスクへのSkill推薦
- `SKILL.md` / `WORKFLOW.md` export

---

## 12. 初期スコープ外

```text
- 大企業向けCompany Agent OS
- 大規模RBAC / Entra連携 / enterprise audit dashboard
- 汎用workflow builder
- Agent marketplace
- 完全自律Agent会社
- 複数テナントSaaS
- 全SaaS connector marketplace
```

---

## 13. 未決定事項

1. 最初のWork Boardを自作するか、Plane/Huly/OpenProject上にadapterとして載せるか。
2. 最初のDBをPostgreSQLにするか、single-host MVPはSQLiteにするか。
3. raw logs保存をfilesystem固定にするか、MinIO optionalにするか。
4. OTel exportをMVPに入れるか、後続に回すか。
5. Codex CLI と Codex App Server のどちらを既定 Agent Profile にするか。
6. Agent Skills形式を特定ベンダー互換に寄せるか、汎用Skill bundleとして設計するか。

---

## 14. まとめ

Nagare / 流 は、Company Agent OSではない。  
まず必要なのは、現場が使う **Agent Kanban / Agent Andon Board / 整流板** である。

最終的な定義は以下でよい。

> **Nagare / 流**  
> 複数Agent・複数実行環境を横断して、作業カードにPlan・Execution・Artifact・Evidence・Verification・Review・Handoff・Memoryを保存し、登録済みAgent Profileで実行できる、Agent作業を整流するセルフホスト可能な現場看板。

この方向なら、大企業のモデル・Agent OSと正面衝突せず、まず Codex CLI と Codex App Server を確実に束ねる現場レイヤーになれる。Codex App Server は、その中でもサーバー型 Codex 連携の主入口になる。

---

## References

[^openai-agentkit]: OpenAI, “Introducing AgentKit,” 2025-10-06. https://openai.com/index/introducing-agentkit/

[^openai-symphony]: OpenAI, “An open-source spec for Codex orchestration: Symphony,” 2026-04-27. https://openai.com/index/open-source-codex-orchestration-symphony/

[^ms-agent365]: Microsoft, “Microsoft Agent 365: The Control Plane for Agents.” https://www.microsoft.com/en-us/microsoft-agent-365

[^ms-agent365-blog]: Microsoft, “Microsoft Agent 365: The control plane for AI agents,” 2025-11-18. https://www.microsoft.com/en-us/microsoft-365/blog/2025/11/18/microsoft-agent-365-the-control-plane-for-ai-agents/

[^google-adk-a2a]: Google Cloud, “Build and manage multi-system agents with Vertex AI,” 2025-04-09. https://cloud.google.com/blog/products/ai-machine-learning/build-and-manage-multi-system-agents-with-vertex-ai

[^google-a2a]: Google Developers Blog, “Announcing the Agent2Agent Protocol (A2A),” 2025-04-09. https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability/

[^anthropic-skills]: Anthropic, “Equipping agents for the real world with Agent Skills,” 2025-10-16. https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills

[^agent-skills-docs]: Anthropic Claude API Docs, “Agent Skills.” https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview

[^vibe-kanban]: BloopAI, “vibe-kanban,” GitHub. https://github.com/BloopAI/vibe-kanban

[^vibe-shutdown]: Vibe Kanban Blog, “Shutdown,” 2026-02-28. https://www.vibekanban.com/blog/shutdown

[^vibe-logs]: Vibe Kanban Blog, “Goodbye SQLite (for logs),” 2026-02-26. https://www.vibekanban.com/blog/goodbye-sqlite-for-logs

[^cline-kanban]: Cline, “Announcing Cline Kanban,” 2026-03-26. https://cline.bot/blog/announcing-kanban

[^cline-cli]: Cline, “Coding Agents in Your Terminal and on a Kanban Board.” https://cline.bot/cli

[^hermes-release]: NousResearch/hermes-agent, “RELEASE_v0.13.0.md,” 2026-05-07. https://github.com/NousResearch/hermes-agent/blob/main/RELEASE_v0.13.0.md

[^hermes-kanban]: Hermes Agent Docs, “Kanban (Multi-Agent Board).” https://hermes-agent.nousresearch.com/docs/user-guide/features/kanban

[^hermes-worker-lanes]: Hermes Agent Docs, “Kanban worker agent profiles.” https://hermes-agent.nousresearch.com/docs/user-guide/features/kanban-worker-lanes

[^hermes-handoff]: Hermes Agent Docs, “Recommended handoff evidence,” in Kanban docs. https://hermes-agent.nousresearch.com/docs/user-guide/features/kanban

[^vscode-agent-kanban]: appsoftwareltd, “vscode-agent-kanban,” GitHub. https://github.com/appsoftwareltd/vscode-agent-kanban

[^hn-vscode-agent-kanban]: Hacker News discussion, “VS Code Agent Kanban,” 2026. https://news.ycombinator.com/item?id=47307169

[^mast]: Cemri et al., “Why Do Multi-Agent LLM Systems Fail?” arXiv:2503.13657, 2025. https://arxiv.org/abs/2503.13657

[^verimap]: Xu et al., “Verification-Aware Planning for Multi-Agent Systems,” EACL 2026. https://aclanthology.org/2026.eacl-long.353/

[^awm]: Wang et al., “Agent Workflow Memory,” OpenReview. https://openreview.net/forum?id=NTAhi2JEEE

[^trace-assurance]: Paduraru et al., “A Trace-Based Assurance Framework for Agentic AI Orchestration: Contracts, Testing, and Governance,” arXiv:2603.18096, 2026. https://arxiv.org/abs/2603.18096

[^overeager]: Qu et al., “Overeager Coding Agents: Measuring Out-of-Scope Actions on Benign Tasks,” arXiv:2605.18583, 2026. https://arxiv.org/abs/2605.18583

[^single-agent-skills]: Li, “When Single-Agent with Skills Replace Multi-Agent Systems and When They Fail,” arXiv:2601.04748, 2026. https://arxiv.org/abs/2601.04748

[^otel-genai]: OpenTelemetry, “Semantic conventions for generative client AI spans.” https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/

[^codex-exec]: OpenAI Developers, “Codex CLI reference — codex exec.” https://developers.openai.com/codex/cli/reference

[^codex-app-server]: OpenAI Developers, “App Server – Codex.” https://developers.openai.com/codex/app-server

[^codex-cli-app-server]: OpenAI Developers, “Codex CLI reference — codex app-server.” https://developers.openai.com/codex/cli/reference
