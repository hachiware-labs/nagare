# Nagare Tauri UI implementation plan

This plan maps the Fable-5 prototype to the desktop app implementation.
The current UI source of truth is `docs/design-assets/prototype/index.html`;
older assets under `docs/archive/` are history only.

## Target

Nagare should ship as a Tauri desktop app whose screens answer:

> この作業は今どうなっていて、次に自分は何をすればよいか。

The implementation should keep the existing Rust core as the product logic
boundary and make the desktop UI a client of that logic.

## Phases

1. Align the existing local UI with PRD v1.0.
   - Use the new left navigation structure.
   - Keep Work as the daily home screen.
   - Remove prototype leftovers that imply unsupported operations.
   - Keep E2E coverage for first setup, work creation, review, approval,
     question handling, and recovery.

2. Introduce the Tauri desktop shell.
   - Add a desktop app package that loads the same UI flows.
   - Expose commands that call `nagare-core` use cases instead of duplicating
     business logic in the frontend.
   - Keep the local HTTP UI server available for Playwright E2E until desktop
     E2E is ready.

3. Move UI state behind a typed app boundary.
   - Add view-model functions or JSON commands for Work Home, Work Detail,
     Project, Knowledge, Agent, Skills, MCP, Runtime, and Insights.
   - Reuse those view models from both the Tauri app and the local UI server.

4. Implement the P0 prototype flow end to end.
   - First-run setup: project, detected runtime, connection test, hidden
     default agent wiring.
   - Work Home: request input, project selector, flat list, status/project
     filters, result summaries.
   - Start confirmation: request summary, team rationale, review criteria,
     confirmation policy.
   - Work Detail: status strip, user action, result, review, execution flow,
     question, recovery, approval/reject, done summary.
   - Support screens: Project, Knowledge, Agent, Skills, MCP, Runtime.

5. Implement NF-2 trace contract.
   - Write `.nagare/works/<work_id>/trace.jsonl`.
   - Record `work_header`, `organizer_decision`, `worker_output`,
     `reviewer_verdict`, `human_decision`, and `recovery_event`.
   - Render the Work Detail audit line from this structured trace.

## Completion evidence

- `cargo test --workspace` succeeds.
- Playwright E2E covers the P0 local UI flow.
- Desktop build/check succeeds for the Tauri package.
- The implemented screens follow `docs/design-assets/prototype/README.md`
  and `docs/nagare_prd_v1_0.md`, not archived wireframes.

## Current implementation status

Updated: 2026-07-05

Implemented in the desktop app:

- Tauri shell and typed command boundary over `nagare-core`.
- First-run setup as a 3-step modal flow: project folder -> detected Claude
  Code, Codex CLI, OpenCode, or OpenClaw runtime -> connection confirmation,
  with hidden default agent wiring.
- Work home with new request input, project/status/search filters, and work
  rows. The history filters now show per-option counts, preserve combined
  status/project/keyword filtering, show the visible/total count and an empty
  state for zero matches, and mark attention-needed work with a row accent and
  the Work navigation dot. Completed/reviewed rows summarize both the latest
  answer and the review score so the list can be scanned without opening the
  detail page.
- Work creation uses the active project from the current desktop root instead
  of a hard-coded project label, so setup and project switching flow into the
  next work request.
- Start confirmation before work creation, including request summary,
  predicted artifact/domain knowledge chips, review rubric preview, execution
  team rationale, runtime status, and per-work approval policy selection.
- Three approval policies are supported end to end: final manual approval,
  manual approval only when review concerns remain, and auto-complete on review
  pass.
- Work detail for status, user action, result, review, execution flow,
  questions, recovery, approval, and rejection.
- Work detail uses staged disclosure for review items, step rationale/input/
  output/knowledge, and runtime diagnostics, matching the prototype's audit-line
  hierarchy. Review steps in the execution flow keep the full rubric verdict
  table visible by default, with concern rows highlighted, while rationale,
  inputs, knowledge, and diagnostics remain behind the step disclosure.
- Question handling now exposes the source step/agent in the primary action
  panel and states that the answer resumes only the current work, matching the
  prototype's input-first detail flow.
- Recovery handling now expands the primary action panel with cause,
  failure class, impact, completed handoff context, pending handoff context,
  target agent, and warnings so recovery is presented as a first-class flow
  instead of a generic error.
- Rejection handling now lists review concerns as citeable items, inserts cited
  concerns into the reject comment, and records `cited_concerns` in the human
  decision trace from that structured comment.
- Done-state work detail now replaces the generic "no action" panel with a
  completion summary that repeats the final answer, adopted artifact filenames,
  review score/concerns, and every recorded step summary.
- Review criteria shown in Work detail now link back to the owning artifact
  rubric editor, so a user can move from a review concern to the exact
  Knowledge screen that controls the criterion.
- Project settings with organizer fallback and workflow/approval defaults.
- Project management now shows the current project as a primary list row and
  supports creating/switching to a new project folder from a focused modal,
  then opens organizer/workflow/approval defaults in a focused settings modal.
  Project creation/settings persist the project display name and icon in the
  project config, so the list, work creation selector, and settings modal use
  the same user-facing identity instead of falling back to the folder name.
  The settings modal also exposes participating agents and shared
  domain/artifact knowledge with links to the owning Agent and Knowledge
  screens, and includes the danger-zone project deletion flow. Deletion removes
  only the `.nagare` project state and leaves files in the target folder.
- Knowledge, agent, skill, MCP, and runtime management screens.
- Agent management now follows the prototype's list-first pattern: the screen
  separates the built-in organizer from work/review agents, supports role,
  domain, and keyword filtering, and create/edit opens a focused modal instead
  of keeping a long inline form on the list screen.
- Agent management persists an optional PNG/JPG/SVG avatar image from the
  edit modal, shows it in the list, and falls back to name initials when no
  image is set.
- Agent runtime/model editing is linked to the detected runtime: Codex and
  Claude show default/fixed/manual model choices, OpenCode/OpenClaw expose
  provider-based entry, and Ollama/LMStudio-style providers require Base URL
  only when selected.
- Agent editing includes a prompt-assist draft flow that builds role/scope/
  skill/MCP-based instructions and lets the user insert them into the editable
  behavior field.
- Skill management now follows the same library-first pattern: the screen shows
  installed skill sets and packages, while add/delete operations use focused
  modals and assignment remains centralized in agent settings.
- Skill package deletion with agent detachment and scoped body removal.
- MCP connection management follows a connection-library pattern: the list is
  primary, add/edit/delete use focused modals, connection tests remain per row,
  and tested MCP assignment stays in the agent settings screen.
- Knowledge management now follows the prototype's domain-first pattern:
  domains and artifact types are shown as lists, while domain knowledge and
  artifact-specific rubric editing use focused modals with larger text areas.
- Artifact rubric editing validates the prototype Markdown format on save:
  `## item (points)` headings must have criteria text, duplicate items are
  rejected, and score totals must be 100. Rubric changes are persisted as a
  new rubric version and the list shows version, item count, and total score.
- Artifact editing includes a rubric-assist draft flow that builds a
  source-attributed rubric from the selected domain, artifact description, and
  artifact knowledge, then inserts it for user editing.
- Agent prompt, domain knowledge, and artifact rubric editors now use the same
  two-part AI assistance pattern: user-triggered draft generation plus
  review-derived improvement proposals that link to the inbox/full-diff preview.
- Runtime support for Claude Code and OpenCode is wired through core
  `AgentToolKind`, default runtime/adapter declarations, desktop first-run
  setup, and process adapters. Claude Code uses print mode, and OpenCode uses
  run mode.
- Runtime management shows the CLI command, detection detail, model selection
  mode, model/provider handling, and per-runtime refresh actions backed by the
  Tauri command boundary. The management list now shows detected runtimes only,
  matching the prototype rule that unavailable runtimes are not execution
  candidates.
- Insights screen backed by desktop view-model data: review count, average
  score, concern count, agent score summaries, low-scoring rubric/criteria
  items, improvement proposal inbox, and recent review evidence. Proposals open
  a rooted diff/evidence preview first, then route to the target settings
  screen/editor for human editing; they are not auto-applied.
- NF-2 trace JSONL writer and contract test for decision-flow records.
- Playwright coverage for the Tauri frontend shell using a mocked
  `window.__TAURI__.core.invoke` boundary. The covered flows are first-run
  setup -> start confirmation -> approval, project settings, runtime refresh,
  knowledge management, skill management, MCP registration -> connection test
  -> agent assignment, and review-derived insights -> improvement target
  preview and target routing.
- Packaged-window smoke coverage is available through
  `npm run test:e2e:desktop-window`. It launches `tauri-driver` against the
  built `nagare-desktop.exe`, uses an isolated `NAGARE_ROOT`, and verifies that
  the real desktop window loads the Work setup screen. The script uses a
  matching `msedgedriver` when provided, or downloads the matching Windows
  driver for the installed WebView2 runtime when needed. It skips with a clear
  message only when the driver cannot be prepared, or fails strictly when
  `NAGARE_DESKTOP_E2E_STRICT=1`.

Remaining P0/P1 gaps to close before calling the prototype implemented:

- Actual external-LLM-backed prompt/rubric generation is still intentionally
  limited to local draft/preview assistance. The prototype behavior is present:
  user-triggered drafts can be inserted and edited, while review-derived
  improvements show diff/evidence previews before human editing.
