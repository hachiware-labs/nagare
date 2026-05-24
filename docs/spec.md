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
| 1.1.1 | Project を初期化すると、`.nagare` 配下に設定、台帳、artifact/log 保存先を作成する。 | project root が存在する | `nagare init` を実行する | `.nagare/project.toml`、`.nagare/state/ledger.json`、`.nagare/artifacts/`、`.nagare/logs/` が存在する | 実装済み |
| 1.1.2 | 既存の Nagare Project を再初期化しても、既存の設定と台帳を破壊しない。 | `.nagare` が存在する | `nagare init` を再実行する | 既存ファイルが維持され、CLI は成功する | 実装済み |
| 1.2.1 | Project locale を設定できる。 | Project が初期化済み | `nagare locale use --language <locale> --timezone <timezone>` を実行する | `.nagare/project.toml` の `[locale]` が更新される | 実装済み |
| 1.2.2 | Project locale を確認できる。 | Project が初期化済み | `nagare locale show` を実行する | language と timezone が表示される | 実装済み |
| 1.2.3 | Nagare が生成する記録には locale を保存する。 | Project locale が設定済み | Work Item、Run、Evidence、Verification、Decision、Probe を作成する | 各 ledger record に `locale` が保存される | 実装済み |
| 1.2.4 | timezone は Project 設定として保存し、日時表示の locale 対応に利用できる設計にする。 | Project locale が設定済み | 時刻を表示または記録する | timezone を参照できる | 進行中 |

## 2. Work Item

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 2.1.1 | Work Item を作成できる。 | Project が初期化済み | `nagare item create --title <title>` を実行する | `work_0001` 形式の ID を持つ Work Item が ledger に保存される | 実装済み |
| 2.1.2 | Work Item 作成時に description を保存できる。 | Project が初期化済み | `--description <text>` を付けて作成する | Work Item snapshot に description が残る | 実装済み |
| 2.1.3 | Work Item 作成時の初期 status は `ready` とする。 | Project が初期化済み | Work Item を作成する | status が `ready` になる | 実装済み |
| 2.2.1 | Work Item 一覧を表示できる。 | Work Item が存在する | `nagare item list` を実行する | ID、status、title が確認できる | 実装済み |
| 2.2.2 | Work Item 詳細を表示できる。 | Work Item が存在する | `nagare item show <work_id>` を実行する | Work Item、runs、artifacts、evidence、verification、handoffs、decisions が確認できる | 実装済み |
| 2.2.3 | 存在しない Work Item は拒否する。 | 指定 ID が ledger に存在しない | Work Item 対象 command を実行する | エラーになり、ledger は変更されない | 実装済み |

## 3. Agent Profile 管理

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 3.1.1 | Project-local Agent Profile を登録できる。 | Project が初期化済み | `nagare agent add --id <id> --runtime <runtime> --adapter <adapter>` を実行する | `.nagare/agents/<id>.toml` が作成される | 実装済み |
| 3.1.2 | Agent Profile には表示名、role、runtime、adapter、working_dir、description、specialties を保存できる。 | Project が初期化済み | `nagare agent add` に各 option を渡す | 保存された TOML に値が残る | 実装済み |
| 3.1.3 | Agent Profile の `working_dir` は Project 内の相対 path に限定する。 | Project が初期化済み | workspace 外または絶対 path を指定する | 登録または実行が拒否される | 実装済み |
| 3.1.4 | Agent Profile の dispatch routing hint を後から更新できる。 | Agent Profile が存在する | `nagare agent update <id> --description ... --specialties ...` を実行する | `.nagare/agents/<id>.toml` が更新され、Agent Profile 詳細に反映される | 実装済み |
| 3.2.1 | 登録済み Agent Profile を一覧表示できる。 | Agent Profile が存在する | `nagare agent list` を実行する | profile ID、adapter、runtime、role が確認できる | 実装済み |
| 3.2.2 | 登録済み Agent Profile の詳細を表示できる。 | Agent Profile が存在する | `nagare agent show <agent_profile_id>` を実行する | display_name、adapter、runtime、working_dir が確認できる | 実装済み |
| 3.2.3 | 未知の Agent Profile を実行対象に指定した場合は拒否する。 | 指定 ID が存在しない | `item run --agent <id>` を実行する | エラーになり、Agent Run は作成されない | 実装済み |

## 4. Nagare Agent Defaults / Dispatch 準備

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 4.1.1 | Nagare 本体が使う既定 Agent Profile を設定できる。 | Agent Profile が存在する | `nagare agent use --work-agent ... --review-agent ... --dispatch-agent ...` を実行する | `.nagare/project.toml` の `[nagare_agents]` が更新される | 実装済み |
| 4.1.2 | 既定 Agent Profile を確認できる。 | Project が初期化済み | `nagare agent defaults` を実行する | work_agent、review_agent、dispatch_agent が表示される | 実装済み |
| 4.1.3 | `item run` で agent と採用済み DispatchPlan が省略された場合は `work_agent` を使う。 | `work_agent` が設定済み | `nagare item run <work_id>` を実行する | Agent Run の agent_profile_id が `work_agent` になる | 実装済み |
| 4.2.1 | dispatch_agent は Work Item の実行前確認に使う。 | dispatch_agent が設定済み | `nagare item preview <work_id>` を実行する | dispatch_agent の AgentRun が `dispatch_preview` として記録される | 実装済み |
| 4.2.2 | dispatch は Work Item の実作業を進めない。 | Work Item が存在する | Preview を実行する | AgentRun と Evidence は残るが、Work Item status は実行結果で進まない | 実装済み |
| 4.2.3 | review_agent は実行後の評価に使う。 | review_agent が設定済み | `nagare item review <work_id>` を実行する | review_agent の AgentRun が `review` として記録される | 実装済み |
| 4.2.4 | dispatch preview の結果は Dispatch Plan として保存する。 | dispatch preview が成功する | Preview または Handoff Dispatch を実行する | DispatchPlan が AgentRun、ResolvedRunPacket、Artifact と紐づいて ledger に保存される | 実装済み |
| 4.2.5 | dispatch_agent には小さな候補 Agent Profile リストだけを渡す。 | dispatch preview を開始する | `nagare item preview` を実行する | Project Rule、既定 agent、登録 profile から最大 5 件の候補 summary が prompt に含まれる | 実装済み |
| 4.2.6 | dispatch_agent は候補リストから target Agent Profile を選べる。 | dispatch_agent が JSON を返す | `target_agent_profile_id` を含む dispatch output を保存する | 存在する Agent Profile なら DispatchPlan.target_agent_profile_id に採用され、不正 ID は fallback target になる | 実装済み |
| 4.2.7 | DispatchPlan は `draft`、`accepted`、`superseded` の lifecycle を持つ。 | DispatchPlan が存在する | Preview または accept を実行する | 新しい preview は古い draft を superseded にし、accept は選択 plan を accepted にする | 実装済み |
| 4.2.8 | DispatchPlan を実行前に採用できる。 | draft DispatchPlan が存在する | `nagare item dispatch accept <work_id>` を実行する | 対象 plan が accepted になり、同じ Work Item の他 plan は superseded になる | 実装済み |
| 4.2.9 | dispatch output contract 違反は fallback として記録する。 | dispatch_agent が JSON なし、target 未指定、または未知 target を返す | dispatch preview を保存する | fallback target が使われ、DispatchPlan.selection_warnings に理由が残る | 実装済み |

## 5. Agent Health / Capability Probe

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 5.1.1 | Agent Profile の runtime が利用可能か確認できる。 | Agent Profile が存在する | `nagare agent doctor <agent_profile_id>` を実行する | runtime command の存在と healthcheck 結果が表示される | 実装済み |
| 5.1.2 | Agent Profile の capability snapshot を取得できる。 | Agent Profile が存在する | `nagare agent probe <agent_profile_id>` を実行する | CapabilityProbe が ledger に保存される | 実装済み |
| 5.1.3 | CapabilityProbe は runtime、adapter、利用可否、発見 capability、instruction source、locale を保存する。 | Probe が実行される | Probe が完了する | 後続の解決処理で参照できる snapshot が残る | 実装済み |
| 5.1.4 | Run / Preview 前に CapabilityProbe を自動更新する。 | Agent Run を開始する | Probe が未取得、古い、runtime / adapter / runtime_version が一致しない | 新しい CapabilityProbe が ledger に保存され、その probe が ResolvedSkillContext に紐づく | 実装済み |
| 5.2.1 | Skill Set の適用可否は Probe 結果または adapter capability を使って判断する。 | Declared Skill Set と Agent capability が存在する | Run Packet を解決する | required capability を満たさない Skill Set は `skipped_skill_set_ids` に記録され、制約として残る | 実装済み |

## 6. Skill Set / Project Rule / Run Packet

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 6.1.1 | Skill Set は Agent に渡したい instruction、schema、playbook、rubric、script の束として宣言する。 | Project config が存在する | Skill Set を設定する | 宣言は config-owned entity として保存される | 進行中 |
| 6.1.2 | Skill Set は Agent Profile に接続されるが、実際に使えるかは Probe または adapter capability で決める。 | Agent Profile と Skill Set が存在する | Run Packet を解決する | applied / skipped skill set が記録される | 実装済み |
| 6.2.1 | Project Rule は path / glob に応じて Agent Profile、Skill Set、Policy、Verification を選ぶ。 | Project Rule が存在する | `nagare rule check <path>` を実行する | matching rule と選択根拠が表示される | 実装済み |
| 6.2.2 | `item preview` は dispatch_agent で実行前確認を記録する。 | Work Item と dispatch_agent が存在する | `nagare item preview <work_id>` を実行する | `dispatch_preview` 目的の AgentRun、Artifact、Evidence、DispatchPlan が保存される | 実装済み |
| 6.2.3 | `item preview` は Project Rule、Skill Set、Policy、Verification を解決して表示する。 | Project Rule が存在する | `nagare item preview <work_id> --path <path>` を実行する | Agent Profile、Project Rule、Skill Set、Policy、Verification が表示され、dispatch prompt に含まれる | 実装済み |
| 6.2.4 | dispatch prompt は Agent instruction source の全文を含めない。 | Agent Profile と Probe が存在する | dispatch preview prompt を生成する | 候補 context は profile summary に限定され、大きな AGENTS.md / SOUL.md などは直接展開しない | 実装済み |
| 6.3.1 | Resolved Skill Context は実行時に使った Rule、Skill Set、Capability、Instruction source を固定する。 | Preview または Run が実行される | AgentRun を作成する | `ResolvedSkillContext` が ledger と artifact に保存される | 実装済み |
| 6.3.2 | Resolved Run Packet は実行時に使った Work Item、Agent Profile、実行目的、working_dir、goal、DispatchPlan、Policy、Verification、Resolved Skill Context を固定する。 | Preview または Run が実行される | AgentRun を作成する | `ResolvedRunPacket` が ledger と artifact に保存され、Adapter 実行入力として使われる | 実装済み |
| 6.3.3 | Work Item 詳細で解決済み Skill Context と Run Packet を確認できる。 | 解決済み記録が存在する | `nagare item show <work_id>` を実行する | resolved_skill_contexts と resolved_run_packets が表示される | 実装済み |
| 6.3.4 | Context Budget は初期MVPでは固定上限とする。 | dispatch prompt を生成する | 候補 Agent Profile を選ぶ | 候補数は最大5件に固定され、設定化はしない | 実装済み |

## 7. Work Item Run / Adapter

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 7.1.1 | Work Item を Agent Profile で実行できる。 | Work Item と Agent Profile が存在する | `nagare item run <work_id> --agent <agent_profile_id>` を実行する | AgentRun、Artifact、Evidence が保存される | 実装済み |
| 7.1.2 | `--command` は smoke / verification 用 fallback として実行できる。 | Work Item と Agent Profile が存在する | `item run --command <command>` を実行する | command log が Artifact として保存される | 実装済み |
| 7.1.3 | `--prompt` は `process.codex-cli` adapter 経由で `codex exec` に渡す。 | adapter が `process.codex-cli` で Codex CLI が利用可能 | `item run --prompt <text>` を実行する | `codex exec --cd <working_dir> <prompt>` の結果が AgentRun に保存される | 実装済み |
| 7.1.4 | Agent Run の cwd は Agent Profile の `working_dir` を使う。 | Agent Profile に working_dir がある | Run を開始する | process cwd または Codex `--cd` が working_dir になる | 実装済み |
| 7.1.5 | `item run --path` は Project Rule で解決した Agent Profile を使う。 | Project Rule が存在し、`--agent` が省略されている | `nagare item run <work_id> --path <path>` を実行する | matching rule の default_agent で AgentRun が作成される | 実装済み |
| 7.1.6 | `item run` は採用済み DispatchPlan の target Agent Profile を使える。 | accepted DispatchPlan が存在し、`--agent` が省略されている | `nagare item run <work_id>` または `--dispatch-plan <id>` を実行する | AgentRun の agent_profile_id が DispatchPlan.target_agent_profile_id になる | 実装済み |
| 7.2.1 | `stdio.codex-app-server` は Agent Profile として登録・確認できる。 | Codex app-server runtime が設定済み | agent add/list/show/doctor/probe を実行する | profile と probe 結果が扱える | 実装済み |
| 7.2.2 | `stdio.codex-app-server` の実実行は stdio JSON-RPC adapter で扱う。 | Run Packet が存在する | app-server adapter で run を開始する | `initialize`、`thread/start`、`turn/start`、`turn/completed` の transcript が AgentRun artifact に保存される | 実装済み |
| 7.3.1 | Codex MCP Server、Claude Code、HTTP adapter、SDK adapter は初期 Agent adapter に含めない。 | Adapter を登録または選定する | 初期 adapter scope を確認する | 対応予定は `process.codex-cli` と `stdio.codex-app-server` のみになる | 実装済み |

## 8. Artifact / Evidence

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 8.1.1 | Agent Run の stdout/stderr と終了状態を Artifact として保存する。 | Agent Run が開始される | Run が終了する | `.nagare/artifacts/` 配下に log artifact が残る | 実装済み |
| 8.1.2 | Artifact は Work Item と Agent Run に紐づく。 | Artifact が作成される | Ledger に保存する | artifact_id、work_item_id、agent_run_id を辿れる | 実装済み |
| 8.2.1 | Agent Run の結果から Evidence を生成する。 | Run が終了する | Ledger を更新する | 成功または失敗 claim と basis が保存される | 実装済み |
| 8.2.2 | Evidence の自動生成文言は Project locale に合わせる。 | Project locale が設定済み | Evidence を生成する | 日本語または英語の claim / basis が保存される | 実装済み |
| 8.2.3 | Evidence は Artifact を根拠として参照する。 | Artifact が存在する | Evidence を作成する | evidence.artifact_id から根拠 artifact を参照できる | 実装済み |

## 9. Verification

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 9.1.1 | Work Item に対して検証 command を実行できる。 | Work Item が存在する | `nagare verify <work_id> --command <command>` を実行する | VerificationResult と log Artifact が保存される | 実装済み |
| 9.1.2 | 検証成功時は Work Item を approval 可能な状態にする。 | Verification command が exit code 0 で終了する | Verification を保存する | VerificationResult が `passed` になる | 実装済み |
| 9.1.3 | 検証失敗時は Work Item を `failed_verification` にする。 | Verification command が non-zero で終了する | Verification を保存する | VerificationResult が `failed` になり status が `failed_verification` になる | 実装済み |
| 9.1.4 | Verification の自動生成文言は Project locale に合わせる。 | Project locale が設定済み | Verification を保存する | 日本語または英語の evidence が保存される | 実装済み |
| 9.2.1 | command 以外の CI、schema check、LLM judge、human check を検証方法として扱える設計にする。 | Verifier declaration が存在する | Verification を実行する | method ごとの結果が VerificationResult に正規化される | 計画 |

## 10. Handoff

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 10.1.1 | Work Item の Handoff Packet を作成できる。 | Work Item と from/to Agent Profile が存在する | `nagare handoff create <work_id> --from-agent <id> --to-agent <id> --reason <text>` を実行する | HandoffPacket が ledger に保存される | 実装済み |
| 10.1.2 | Handoff Packet は reason と summary を保存する。 | Handoff を作成する | summary を指定または省略する | reason、summary、from_agent_profile、to_agent_profile が保存される | 実装済み |
| 10.1.3 | Handoff Packet には locale を保存する。 | Project locale が設定済み | Handoff を作成する | HandoffPacket.locale が保存される | 実装済み |
| 10.2.1 | Handoff から別 Agent Profile で再実行できる。 | HandoffPacket が存在する | `item run --agent <to_agent>` を実行する | 新しい AgentRun が同じ Work Item に追加される | 実装済み |
| 10.2.2 | Handoff 後に dispatch_agent で再確認できる。 | HandoffPacket が存在する | `nagare handoff dispatch <work_id>` を実行する | `dispatch_preview` 目的の AgentRun、Artifact、Evidence が保存される | 実装済み |
| 10.2.3 | 将来の Handoff は current state、artifact、evidence、open questions、requested output、verification needed を含む。 | Work Item に実行履歴がある | Handoff を作成する | 次 Agent が判断できる構造化 packet が残る | 計画 |

## 11. Human Decision

| ID | 仕様 | Given | When | Done | 状態 |
| --- | --- | --- | --- | --- | --- |
| 11.1.1 | 検証済み Work Item を人間が approve できる。 | Work Item に passing verification がある | `nagare decision approve <work_id>` を実行する | HumanDecision が保存され、Work Item が `done` になる | 実装済み |
| 11.1.2 | passing verification がない Work Item の approve は拒否する。 | passing verification が存在しない | `decision approve` を実行する | エラーになり、Work Item は `done` にならない | 実装済み |
| 11.1.3 | rationale が省略された場合は Project locale に合わせた既定理由を保存する。 | Project locale が設定済み | rationale なしで approve する | 日本語または英語の rationale が保存される | 実装済み |
| 11.2.1 | approve 以外の reject、request_changes、pause、delegate、override を保存できる設計にする。 | Review または Human action が発生する | Decision を記録する | decision_type と rationale が ledger に残る | 計画 |

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
| ERR-VERIFY-0001 | 承認前検証不足 | passing verification が存在しない | `nagare verify` を実行する | 可 | 11.1.2 |
| ERR-RUN-0001 | Agent 実行失敗 | process exit code が non-zero | Artifact と Evidence を確認し、handoff または再実行する | 可 | 7.1.1 |

| ID | 文面テンプレ | 出力先 | 発生条件 | 関連仕様 / ERR |
| --- | --- | --- | --- | --- |
| MSG-PRJ-0001 | Project initialized at `<root>` | stdout | `nagare init` 成功 | 1.1.1 |
| MSG-LOC-0001 | locale updated | stdout | `nagare locale use` 成功 | 1.2.1 |
| MSG-WORK-0001 | created `<work_id>` | stdout | Work Item 作成成功 | 2.1.1 |
| MSG-AGENT-0001 | added agent profile `<id>` | stdout | Agent Profile 登録成功 | 3.1.1 |
| MSG-RUN-0001 | run `<run_id>` completed | stdout | Agent Run 完了 | 7.1.1 |
| MSG-VERIFY-0001 | verification `<id>` passed | stdout | 検証成功 | 9.1.2 |
| MSG-DEC-0001 | work item approved | stdout | Human approval 成功 | 11.1.1 |
