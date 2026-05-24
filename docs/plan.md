# plan.md（必ず書く：最新版）

# current
- [ ] [SEED-agent-data-model] Agent Profile / Skill / Capability Probe / Resolved Skill Context / Resolved Run Packet のデータ形式を `docs/agent_data_model.md` に固定し、実装時の schema seed とする
- [ ] [SEED-adapter-kernel] MVP 1 として Agent Profile、Run Packet、Run Event、Artifact、Evidence、Verification Result、Permission Policy、Workspace Policy の正規モデルを `nagare-core` に整理する。完了条件: `cargo test --workspace` と CLI smoke test が PASS する

# future
- MVP 1: Agent Management Kernel。Runtime、Adapter、Agent Profile、Skill Set、Project Rule、Permission Policy、Run Packet Preview を実装する
- MVP 2: Agent Adapter Kernel。Agent Profile、Run Packet、Run Event、Artifact、Evidence、Verification Result、Workspace Policy の正規モデルを adapter 実行に接続する
- MVP 3: First-Class Codex Adapters。`process.codex-cli`、`stdio.codex-app-server` を実装する
- MVP 4: Workspace + Supervision。git worktree、branch/base ref、heartbeat、timeout、cancel、retry、log capture、diff artifact、cleanup policy を実装する
- MVP 5: Verification + Cross-Agent Handoff。failed verification、handoff packet、retry with different agent profile、reviewer agent profile、finding、human decision を実装する
- MVP 6: Minimal Board UI。Work Items、Agent Profiles、Runs、Artifacts、Evidence、Verification、Handoffs、Decisions を表示する
- MVP 7: Public API / Skills。REST API、SSE、JSON Schema、`nagare-core` Skill Bundle を提供する
- MVP 8: Hosted / External Adapters。必要になった時点で外部 hosted agent / CI worker を検討する
- Later: Workflow Memory、Andon、metrics、OTel export、PostgreSQL backend を検討する

# archive
- [x] [SEED-origin-main] `https://github.com/hachiware-labs/nagare.git` を `origin` として初期化した
- [x] [SEED-mvp0-installable-cli] Rust workspace と npm wrapper の MVP 0 skeleton を作成し、`cargo test --workspace` と npm wrapper smoke test を通した
- [x] [SEED-mvp0-installable-cli] `nagare init` / `nagare doctor` を最初の配布体験として実装した
- [x] [SEED-adapter-kernel] Agent Adapter Contract を文書化し、Claude Code と HTTP adapter を初期対象外にし、Codex App Server をサーバー型入口として扱う優先順位を固定した
- [x] [SEED-first-scenario] Codex CLI agent profile failure、Evidence、Codex App Server handoff、retry、Verification、Human approval、`done` まで到達する最初の CLI scenario を実装した
- [x] [SEED-npm-tutorial] README とチュートリアルの操作入口を npm package 由来の `nagare` コマンドに統一し、個別コマンド列で first scenario を検証できるようにした
- [x] [SEED-nagare-command-tutorial] README とチュートリアルを `nagare` コマンド中心に戻した。まとめコマンドはユーザー向けから外し、個別コマンド列で first scenario を完走できることを検証した
- [x] [SEED-architecture-doc] 全体構想、レイヤー、永続化、adapter 境界、CLI 体系を `docs/architecture.md` にまとめ、主要 docs から参照できるようにした
- [x] [SEED-agent-profile-registry] `.nagare/agents/*.toml` に project-local Agent Profile を登録し、`nagare agent add/list/show` と登録Profileで完走する smoke scenario を実装した
- [x] [SEED-agent-health] `nagare agent doctor <agent_profile>` と `nagare agent probe <agent_profile>` を実装し、runtime health と capability snapshot を ledger に保存するようにした
- [x] [SEED-agent-working-dir] Agent Profile に project-relative `working_dir` を追加し、`codex exec --cd` と smoke command cwd に反映した
- [x] [SEED-nagare-agent-defaults] Nagare本体が使う `work_agent` / `review_agent` / `dispatch_agent` を設定できるようにし、`item run` の既定実行先に `work_agent` を使うようにした
- [x] [SEED-locale-records] Project locale を設定できるようにし、Work Item / Run / Evidence / Verification / Decision / Probe の記録へ locale を保存するようにした
- [x] [SEED-numbered-spec] 機能仕様の正本 `docs/spec.md` を追加し、機能ごとの三階層連番で現在実装済み / 進行中 / 計画の仕様を整理した
- [x] [SEED-nagare-agent-usage] `item preview` / `handoff dispatch` は `dispatch_agent`、`item review` は `review_agent` を既定で使用し、AgentRun purpose として記録するようにした
- [x] [SEED-rule-resolution] path から Project Rule、Agent Profile、Skill Set、Policy、Verification を解決する `nagare rule check <path>` と `item preview --path` / `item run --path` の最小形を実装した
- [x] [SEED-resolved-run-records] `item preview` / `item run` で ResolvedSkillContext と ResolvedRunPacket を ledger と artifact に保存するようにした
- [x] [SEED-architecture-split] 機能追加前に `nagare-core/src/lib.rs` と `nagare-cli/src/main.rs` を責務別 module へ分割した。全 Rust 実装ファイルを1000行未満にし、`cargo test --workspace` と CLI help smoke を通した
- [x] [SEED-run-packet-adapter-input] 旧 `RunPacket` を廃止し、`ResolvedRunPacket` に purpose / working_dir / goal を持たせて Adapter 実行入力とログ記録の中心にした
- [x] [SEED-codex-adapters] `process.codex-cli` / `stdio.codex-app-server` を adapter trait 経由で実行するようにした。`stdio.codex-app-server` は JSON-RPC over stdio で thread/turn transcript を保存する
- [x] [SEED-dispatch-plan-records] dispatch preview / handoff dispatch の結果を DispatchPlan として ledger に保存し、AgentRun、ResolvedRunPacket、raw output Artifact と紐づけた
- [x] [SEED-skill-set-resolution] Skill Set required capability を Agent capability と照合し、applied / skipped と skip 理由を ResolvedSkillContext / Run Packet constraints に記録するようにした
- [x] [SEED-auto-probe-refresh] Run / Preview 前に CapabilityProbe の未取得・stale・runtime / adapter / version 不一致を検出し、自動更新するようにした
- [x] [SEED-dispatch-agent-routing] Agent Profile に description / specialties を追加し、dispatch_agent が最大 5 件の compact candidate から target_agent_profile_id を JSON で選べるようにした
