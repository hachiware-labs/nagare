# UI Plan: Nagare Desktop Work Execution Flow

## Requirements Sources

This pass uses existing durable requirements sources.

| Source | Role |
|---|---|
| `docs/design.md` | Work Item-centered UI principles, usability rules, and wording constraints |
| `docs/design-assets/nagare_screen_flow_summary.md` | First-work flow, recovery states, and diagnostic-surface boundaries |
| `docs/project-aware-control-plane-usecases.md` | Project-aware control plane and trace use cases |
| `docs/create-new-item-usecase.md` | Create New Item to completion scenario and Work Item states |

## Product Thesis

Nagare helps a person move one requested piece of AI work forward without
reading implementation logs. The detail screen must answer: what is being worked
on, what state is it in, whether the user needs to act, why that is safe, and
where deeper details can be inspected.

## Users And Roles

| Role | Owns | Should not be asked to enter |
|---|---|---|
| User / Approver | request intent, answers, recovery choice, final adoption | runtime names, internal record IDs, agent wiring |
| Organizer / Controller | routing and next workflow decision | final human approval |
| Work AI | artifact creation and progress notes | review verdict or business adoption |
| Review AI | checking the result against confirmation conditions | user approval |
| Nagare UI | state, human action, readable progress, optional details | raw logs as the default representation |

## Core Objects

| Object | User-facing wording | Role |
|---|---|---|
| Work Item | 依頼 / 作業 | primary object |
| Project | プロジェクト | work folder, participating agents, project-specific knowledge and rubric |
| Knowledge Domain | 知識 / ドメイン | shared domain knowledge and rubrics |
| Agent | エージェント | domain-bound actor with organizer, worker, or reviewer role |
| Current State | 作業中 / 質問あり / 回復が必要 / 確認待ち / 完了 | tells whether action is needed |
| Current Step | 現在の工程 | concise progress |
| Artifact | 変更案 / 成果物 | concrete output |
| Evidence | 根拠 | optional support |
| Diagnostics | 詳細 / 診断 | collapsed developer-oriented material |

## Screen Inventory

| Screen | Type | User Question | Primary Action | State | Pattern |
|---|---|---|---|---|---|
| Work Home | primary | 依頼を作るか、既存ワークに対応が必要か | 依頼を作成 / 対応が必要な作業を開く | normal | Work Intake + History Stack |
| Setup Wizard Entry | gate/onboarding | 最初に何を始めればよいか | セットアップを始める | first run | wizard entry over Work Home |
| Runtime Setup | primary/recovery | 最初に使う実行環境は接続できているか | 次へ / 再試行 | validating / failed / ready | Runtime Readiness Wizard |
| Default Workspace Setup | primary | どの作業フォルダで依頼を受け付けるか | 作成して次へ | draft / ready | First-Run Activation Checklist |
| Work Request Composer | primary | 何を頼み、何を自動推定に任せるか | 依頼を作成 | draft / warning / valid | Work Intake + History Stack |
| Work Run Trace | primary/recovery/result | 今この依頼に対して操作が必要か | 一覧に戻る / 質問に回答 / 回復案を見る / 採用する | working / needs input / recovery / review / done | Agent Execution Trace with inline user action section |

## Flow Causality

| From | Trigger | To | Why This Follows |
|---|---|---|---|
| Setup Wizard Entry | `セットアップを始める` | Runtime Setup | The first-run wizard starts with runtime connection setup |
| Runtime Setup | `次へ` after connection passes | Default Workspace Setup | Runtime is ready; the remaining required choice is the work folder |
| Runtime Setup | connection fails | setup recovery state | The user must fix the blocker before work creation |
| Default Workspace Setup | `作成して次へ` | Work Request Composer | Nagare creates the default project and default organizer/worker/reviewer, then shows a completion toast so requests can be accepted |
| Work Request Composer | `依頼を作成` | Work Run Trace | The request becomes the selected work item |
| Work Run Trace | question appears | question variant | User input blocks only the current work |
| Work Run Trace | work fails | recovery variant | Recovery choice is the next user decision |
| Work Run Trace | result becomes ready | review-ready variant | The result, review verdict, and user adoption action appear inline before the trace |
| Work Run Trace | `採用する` in review-ready variant | done variant | Human decision completes the work |

## Work Run Trace Decision Surface

Source frame alignment:

- Source question: 今この依頼に対して操作が必要か。
- Source object: README のセットアップ手順更新。
- Source pattern: Agent Execution Trace, but translated into user-facing work language.
- Primary surface: readable state sentence, current progress, no-action-needed or action-required decision.

Visibility tiers:

| Tier | Visible content | Treatment |
|---|---|---|
| Always-visible decision surface | human-readable state sentence, current object, current owner in user language, primary action, short reason | dominant top and center |
| Selected supporting detail | current progress, latest readable history, one reason why waiting is safe | right support panel |
| External diagnostics | internal IDs, runtime names, raw logs, full provenance chain, assignment explanation | outside the user-facing detail screen |

Plain-language translation:

| Internal term | User-facing term |
|---|---|
| Work Item | 依頼 / 作業 |
| Agent | 担当AI / 作業担当 |
| Runtime | 実行環境, hidden unless needed |
| Ledger | 履歴 |
| Evidence | 根拠 |
| Criteria | 確認条件 |
| Diff draft | 変更案 |
| Run/Event IDs | hidden from the main surface |

## State Variants

State variants are documented here, not packed into the active product screen.

| State | Main sentence | Primary action | Detail entry |
|---|---|---|---|
| working | いまはAIが変更案を作成中です。操作は不要です。 | 一覧に戻る | 手順を見る |
| needs input | AIから質問があります。回答すると作業を再開できます。 | 質問に回答 | 質問の理由を見る |
| recovery | 作業が止まりました。安全な再開方法を選べます。 | 回復案を見る | 失敗の詳細を見る |
| review | 変更案ができました。採用するか確認してください。 | 確認する | 結果の要点を見る |
| done | 依頼は完了しました。採用済みの内容を確認できます。 | 完了を閉じる | 履歴を見る |

## Inline User Action Boundary

`確認待ち`, `質問あり`, and `回復が必要` are Work Run Trace states, not separate product screens.

| Surface | Role | Main Question | Primary Action | What It Must Not Do |
|---|---|---|---|---|
| Work Run Trace - 質問あり | inline user action | どの質問に答えれば進むか | 回答して再開 | bury the question inside trace records |
| Work Run Trace - 確認待ち | inline user action | この変更案を採用してよいか | 採用する | move adoption to another screen |
| Work Run Trace - 完了 | completion summary | 採用済みの結果は何か | 完了を閉じる | ask for another adoption decision |

Plain-language rule:

- Work Run Trace order is always `タスクの状況` -> optional `ユーザー対応` -> `トレース記録`.
- `ユーザー対応` appears only when the user must answer, choose recovery, adopt, or send back.
- In `確認待ち`, the inline user action section owns the result summary, review verdict, key changes, evidence, and adopt/send-back decision.
- `完了` appears after adoption and shows the final accepted result plus where it was applied.
- `質問あり` blocks progress before the result exists and must show the question, why it is needed, and the answer field in the same inline user action position.

## Screen Information Plan

### Work Home

- Dominant information unit: request composer for creating a new work item.
- Supporting information: scannable work history rows with response-need and progress previews.
- Input/view intent: enter a natural language request, scan existing work states, open the item that needs action.
- Information shapes: `natural_language_request_input`, `dense_record_view`, `status_readiness_view`, `object_summary_view`.
- Unit patterns: prompt composer, `work_history_row`, `object_header`, compact status chips.
- Field responsibility: user enters only the request body; project/context is inherited; confirmation conditions are proposed after creation.
- State and recovery: normal active; setup-required is a modal over this screen; setup-completed appears as a transient toast; question/recovery/review states appear as row actions.
- Primary action: `依頼を作成`.
- Secondary actions: select project, save draft, filter by project first, filter work rows by the three checkbox states `要対応`, `処理中`, and `完了`, search by request keyword last, open work detail.
- Area budget: composer 30%, work history 60%, compact filters/status 10%.
- Gaze route: Work header -> request composer -> create action -> work rows sorted by action need.

Plain-language rule for this screen:

- Work rows must say whether the user needs to act.
- Do not show internal IDs, raw logs, runtime names, or diagnostic chains in the list.
- Row preview should include a readable answer/progress summary, not only a prompt and timestamp.
- Do not duplicate the primary creation button in the page header when the composer is visible.
- Work project is a user selection shown as a dropdown, not a fixed inherited label.
- Work history filtering order is project dropdown first, work-state checkboxes second, and request keyword search last.
- Work-state filtering uses three checkboxes: `要対応`, `処理中`, and `完了`.
- Row status chips use icons and concrete labels such as `要対応・確認`, `要対応・質問`, `要対応・回復`, `処理中`, and `完了`.
- Row action labels are unified as `詳細`; state-specific actions appear inside the detail screen.
- After first-run setup, show `セットアップが完了しました` as a temporary toast on the Work Request Composer; do not add a separate setup-complete screen.

### Setup Wizard Entry

- Dominant information unit: single wizard entry panel that invites the user to start setup.
- Supporting evidence: none in the primary surface; prerequisites are handled on the setup screen.
- Input/view intent: start first-run setup; no happy-path typing on this screen.
- Information shapes: `empty_first_run_view`, `status_readiness_view`, `object_summary_view`.
- Unit patterns: `empty_state_activation_panel`, `object_header`.
- Field responsibility: user does not enter work details here; workspace path and connection are handled in setup.
- State and recovery: wizard entry over normal Work Home; blocked/failed connection belongs to the setup screen.
- Primary action: `セットアップを始める`.
- Secondary action: none.
- Area budget: primary activation surface only; no progress, support panel, work list preview, or diagnostics.
- Gaze route: friendly setup invitation -> brief outcome sentence -> primary setup action.

Plain-language rule for this screen:

- Do not expose runtime, agent, domain, rubric, run IDs, raw logs, or diagnostic terms.
- Do not show setup prerequisites as a checklist on this screen; the setup screen owns that detail.
- The user should feel invited to start the setup wizard, not blamed for a missing setup.

### Runtime Setup

- Dominant information unit: selected runtime readiness for the first work.
- Supporting information: detected agent tool, selected tool-specific settings, and test-message result.
- Input/view intent: choose one detected agent tool, choose the settings required by that tool, and confirm a test message returns.
- Information shapes: `single_choice_agent_tool`, `model_choice`, `effort_choice`, `status_readiness_view`.
- Unit patterns: compact setup form, selected-tool setting panel, dropdown controls, test-message result.
- Field responsibility: user selects an agent tool and the tool-specific choices; Nagare owns execution format and session continuation.
- State and recovery: ready, validating, failed. Undetected tools are excluded from the dropdown and belong to diagnostics.
- Primary action: `次へ`.
- Secondary action: `接続を再確認`, `戻る`.
- Area budget: agent tool selection 20%, selected-tool settings 35%, test message result 30%, footer actions 15%.
- Gaze route: setup step -> agent tool -> model -> effort -> connection result -> continue.

Plain-language rule for this screen:

- Do not show runtime-specific command explanations on this screen.
- Agent tool is a dropdown containing detected tools only.
- Execution format and session behavior are not setup fields.
- Agents keep using the project session continuously by default.
- Do not show session continuation as explanatory text or a checkbox in the Runtime Setup decision surface.
- Selected tool settings are separated into their own panel, such as `Codex の設定`; Claude Code and OpenCode can replace that panel with different fields.
- Model and effort are user-facing choices; for Codex show selected dropdown values such as `GPT-5.5` and `High`.
- Tool-specific setting panel examples: Codex uses `モデル` and `エフォート`; Claude Code uses `モデル` and a simple `確認モード`; OpenCode uses `プロバイダー` and `モデル`.
- Do not show extra model or effort option labels beside the selected dropdown unless the dropdown is open.
- Connection test is a single test-message round trip, not a diagnostic checklist.

### Default Workspace Setup

- Dominant information unit: selected work folder for the default project.
- Supporting information: review-only summary of what Nagare will create automatically: default project, organizer, worker, and reviewer.
- Input/view intent: select one folder, review the generated initial structure, then create it.
- Information shapes: `file_source_input`, `object_summary_view`, `status_readiness_view`, `empty_first_run_view`.
- Unit patterns: compact folder selector, `requirement_checklist_row`, initial structure summary.
- Field responsibility: user selects the folder; Nagare auto-fills project name and default agents; agent roles are review-only on this screen.
- State and recovery: ready, missing folder, invalid folder. Folder errors stay in this screen with a retry/select path.
- Primary action: `作成して次へ`.
- Secondary action: `戻る`.
- Area budget: folder selection 45%, auto-created summary 35%, footer actions 20%.
- Gaze route: setup step -> folder selector -> auto-created structure -> create action.

Plain-language rule for this screen:

- Do not ask the user to name or configure the default project unless the folder name cannot be used.
- Do not expose detailed agent settings, domains, rubrics, prompts, or runtime/session details.
- Show the organizer, worker, and reviewer as auto-created review-only items, not editable rows.
- The screen should answer only: where will work happen, and what minimum structure will be created so requests can start.

### Work Run Trace

- Dominant information unit: task status surface first, then a role-owned detailed step trace for the selected work.
- Supporting evidence: for each step, the owner role, large input block, role-specific decision/created artifact/review result, large output or next-handoff block, and status.
- Input/view intent: let the user see the request text and current task state before reading the trace. When processing, show the current processing agent; when a question or confirmation exists, show it; when a result exists, show the result.
- Information shapes: `object_summary_view`, `status_readiness_view`, `timeline_audit_view`, `evidence_provenance_view`, `relationship_dependency_view`.
- Unit patterns: `object_header`, `primary_answer_result_block`, large role-owned step cards, stacked input/output blocks, `validation_state_block`.
- Field responsibility: no happy-path user input; answers, recovery notes, adopt/send-back choices, and send-back reasons appear only in the optional inline `ユーザー対応` section.
- State and recovery: working active with multi-step progress; question/recovery/review shown as optional user-action sections immediately after `タスクの状況`.
- Primary action in working state: no required action; the visible safe action is `一覧に戻る`.
- Secondary action: stop, return to work list.
- Area budget: task status surface first, then a vertically scrollable full-width detailed role-owned trace whose cards can grow beyond the initial viewport.
- Gaze route: title -> task status -> request text -> optional user action -> trace records -> input -> decision/created artifact/review -> output/next handoff.

Plain-language rule for this screen:

- Each step has exactly one owner role. Do not render organizer, worker, and reviewer as repeated columns inside every row.
- Organizer steps show the input and the decision that routes work to the next process.
- Worker steps show the input and the concrete artifact or change being created.
- Reviewer steps show the artifact received as input and the review result or pending review point.
- The trace is for detailed following, so each row must have enough height for long multi-line input examples, long action/result explanations, multiple output lines, and a clear handoff.
- Do not force the trace into many narrow horizontal columns. Use full-width step cards and stack input, action/result, and output vertically when the content needs room.
- Long examples are acceptable and expected in this screen; the card should grow vertically instead of compressing the text into a single horizontal line.
- Completed result, user questions, and user confirmations must appear before the flow detail so the user can decide quickly without reading the trace.
- The top status area is named `タスクの状況`, not `ユーザー判断`.
- Always include the request text in the task status area.
- The request text can be long. Show it as a multi-line block in the task status area, and let the task status area grow vertically instead of compressing or truncating the request.
- In processing state, show the current processing agent and what it is doing.
- In question or confirmation state, keep the status readable, then add `ユーザー対応` before the trace.
- In result-ready or done state, show the task result before trace details; only result-ready has adoption actions.
- Make the screen type explicit as `ワーク詳細`; the global `ワーク` nav can stay selected, but the main surface must not read as the work list.
- Do not show raw logs, runtime names, internal IDs, or all possible state variants in the primary surface.
- Do not place a generic further-detail button inside this detail screen; review checkpoints must be visible on the screen, and developer diagnostics belong outside the primary product UI.
- If user action becomes necessary, insert a `ユーザー対応` section after `タスクの状況`, then continue with `トレース記録`.
- In `確認待ち`, do not create a separate review screen. Show the concrete result content, review pass/fail, confirmation checks, and `採用する` / `差し戻す` in the inline user action section.

## Field Responsibility Matrix

| Screen | User Input | User Selection | Auto-filled | Review Only | Exception-only Input | Evidence / Display |
|---|---|---|---|---|---|---|
| Runtime Setup | none on happy path | detected agent tool, tool-specific model choice | execution format, session behavior | selected settings | connection details when required | readiness result |
| Default Workspace Setup | none on happy path | work folder | default project name, organizer, worker, reviewer | initial structure summary | project name correction only if folder name is unusable | folder validity |
| Composer | request body | confirmation policy | project, scope hints, work folder | inferred confirmation conditions | manual scope correction | routing preview |
| Run Trace | none while working | answer text / recovery choice / adopt-sendback only when needed | current state, owner, progress, final answer, artifacts, checks | latest progress, reason, stop conditions, review and verification result | answer/recovery note/send-back reason in variants | readable history, selected root evidence, collapsed details |

## Input Friction Audit

| Input | Source | Reuse Strategy | Shown As |
|---|---|---|---|
| Runtime choice | user_selects | detected tools only | dropdown and selected-tool settings |
| Work folder | user_selects | choose once during Default Workspace Setup | folder selector with path preview |
| Default project name | system_inferred | derive from selected folder | review-only summary |
| Default agents | system_inferred | create organizer, worker, and reviewer automatically | review-only initial structure rows |
| Work request | user_enters_new | captured in Composer | read-only request summary |
| Confirmation conditions | system_inferred | generated from request and editable before run | readable checklist |
| Human answer | exception_only | ask only when question blocks progress | question variant |
| Recovery note | exception_only | ask only after recovery choice | recovery variant |
| Raw logs | evidence_display | structured product details first; raw fallback belongs outside the user-facing detail screen | developer diagnostics |

## Plain-Language Review

Applied to `work-run-trace-1440.svg`:

- No internal IDs in the primary decision surface.
- No runtime names in the primary decision surface.
- `agent`, `ledger`, `evidence`, `criteria`, and `diff draft` are translated.
- User can answer "Do I need to act now?" from the main sentence and primary action.
- The detail screen has no further generic detail button; review checkpoints are visible, while developer diagnostics stay outside the product UI.

## Screen Family Continuity

- Navigation model: desktop left pane workspace for repeated work.
- Common navigation order: ワーク, プロジェクト, 知識, エージェント, 設定.
- Navigation labels must use object names. Avoid `担当AI` and `ライブラリ` as primary nav labels.
- Shared state language: 作業中, 質問あり, 回復が必要, 確認待ち, 完了.
- Evidence hierarchy: state/action first, readable reason second, detailed diagnostics last.
- Direction choice: quiet operational tool, monochrome wireframe first, then a restrained left-to-right Flow visual language for high-fidelity screens.
- High-fidelity flow rule: Nagare means Flow, so background, panels, primary actions, navigation selection, the page-header flow line, and repeated Work rows should share a left-to-right gradient direction. Flow accents and Work rows start lighter on the left and gain visual strength toward the detail/action side on the right, so visual energy travels in the same direction as the workflow.
- Work row border rule: repeated Work rows keep solid thin neutral-gray outlines; do not use status-colored borders, gradient strokes, or gradient top-edge lines, because the row fill and status chip already carry the state.
- Work row elevation rule: repeated Work rows do not use individual drop shadows. Row-level shadows create a false lower line that can look like a reversed gradient; separation comes from spacing and neutral borders instead.

## Rejected Patterns

| Pattern | Reason |
|---|---|
| Raw log viewer | It hides the user's next action |
| Equal three-column audit sheet | It overexposes diagnostics |
| Internal-ID timeline | It fails the plain-language review |
| Inactive state controls in active screen | They look actionable when unavailable |
| Generic dashboard | Metrics do not answer whether the user must act now |

## Self-review

This redraw targets 90+ under the new score caps by removing internal IDs and
implementation terms from the primary decision surface. If internal model terms
must be shown later, they belong in a separate developer diagnostics surface.
