# Design Assets

This directory stores the current UI flow summary and wireframe assets for
Nagare. The previous detailed screen set was intentionally removed so the next
iteration can start from the end-to-end flow instead of accumulated individual
screens.

## Current Flow

- Flow summary: `nagare_screen_flow_summary.md`
- Full-page screen flow SVG: `svg/nagare-full-page-flow-wireframe.svg`
- Full-page screen flow PNG: `png/nagare-full-page-flow-wireframe.png`
- Modal flow SVG: `svg/nagare-modal-flow-wireframe.svg`
- Modal flow PNG: `png/nagare-modal-flow-wireframe.png`

## Current Full-Scale Wireframes

- Work home: `work-home-1440.svg`
- Designed work home: `work-home-designed-1440.svg`
- Runtime setup: `runtime-setup-1440.svg`
- Work run trace: `work-run-trace-1440.svg`
- Work run trace variants: `work-run-trace-question-1440.svg`, `work-run-trace-confirmation-1440.svg`, `work-run-trace-done-1440.svg`
- Project list: `project-list-wireframe-1440.svg`
- Project create: `project-create-wireframe-1440.svg`
- Project settings: `project-settings-wireframe-1440.svg`
- Knowledge list: `knowledge-list-wireframe-1440.svg`
- Knowledge create: `knowledge-create-wireframe-1440.svg`
- Knowledge edit: `knowledge-edit-wireframe-1440.svg`
- Agent settings: `agent-settings-wireframe-1440.svg`

## Split Rule

Use full-page wireframes for screens where the user reads, compares, tracks, or
reviews work. Use modal wireframes only for temporary setup or confirmation
tasks that return to a full-page screen.

Full-page wireframes should show the Nagare app shell. The `ワーク` section is
the selected item in the left navigation, not a standalone title bar label.

In the work list, `詳細` opens a single work detail screen. That detail screen
uses tabs such as `結果`, `実行状況`, `レビュー`, and `履歴`; it should not branch
to separate unrelated pages for execution status and artifacts. The work list
should also show enough result summary for completed items, including the answer
and the produced artifacts.

The `結果` tab should show concrete produced file names, not only artifact
types. Reviews should include score, evidence, concerns, and confirmation notes.
If the user sends work back, the UI must provide a comment field so the next run
has actionable feedback. Confirmation policy should be selectable when starting
a work item, for example final confirmation, important-only confirmation, or
automatic adoption.

The result screen should not reserve a permanent action card. Whether user
confirmation is shown depends on the work confirmation policy. When confirmation
is required, place the confirmation entry point in `結果サマリー`; clicking it
opens a modal where the user can approve or send the work back with a message.
Use the freed space for wider artifact and review sections, with vertical
scrolling when the content is long.
Do not draw the confirmation modal as permanent content inside the result
screen. Keep the full-page result screen and the modal confirmation state as
separate wireframes.

The `実行状況` tab should not require the user to switch through each step.
Show a compact overview at the top, then list each step as a chronological panel
with the responsible agent, input, action, output, and current state. Long runs
should be followed by scrolling the step list.

## Current Design Direction

Nagare should first make the main work journey understandable:

1. Start from an empty work state.
2. Complete the minimum setup in a modal: project and runtime.
3. Return to the work screen and create a work request.
4. Follow running steps and required user decisions.
5. Review the outcome and close the work.

Management screens for projects, domains, runtimes, agents, skills, and MCP are
secondary paths. They should support the main journey without appearing as
required first-run work unless the user explicitly opens settings.

## Asset Policy

- Keep adopted flow assets small and easy to review.
- Prefer wireframes while the navigation model is still moving.
- Keep full-page screens and modal flows separate so full-page information
  architecture has enough space to be evaluated.
- Keep exploratory or detailed alternatives under `docs/design-assets/scratch/`
  until they are promoted into this README.
