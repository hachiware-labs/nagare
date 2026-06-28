# Nagare Screen Flow Summary

This note preserves the current screen-flow decisions before replacing the
existing visual screen assets with a new hachi-ui wireframe pass.

## Product Direction

Nagare is a control plane for running work through AI runtimes, agents, skills,
MCP connections, domains, and quality records. The UI priority is not visual
polish first; it is a clear path from an empty app to the first useful work, then
repeatable tracking of work execution and review.

## Core Principles

- Ask only for what is needed to create the first work: project and runtime.
- Do not expose default agents as a setup burden.
- Do not mix list and detail in the same work screen unless both must stay
  visible. Work lists should scan; details should open as their own screen.
- Avoid two-column decision screens. Prefer a left tab/step selector and a
  single focused working surface.
- Show tool, skill, and MCP availability only where it changes the next action.
- Skills and MCPs are global libraries, but assignment is limited to selected
  agents.
- MCP addition requires connection test success before it becomes assignable to
  agents.
- Project organizer can be configured; if unset, Nagare uses the built-in
  organizer and should make that fallback visible.

## Main Flow

1. Empty Work state
   - User sees that no project/runtime exists.
   - Primary action opens setup.

2. Setup Project
   - User enters project name and selects the project folder.
   - Optional rubric/evaluation criteria can be edited, but should not block the
     first work.

3. Setup Runtime
   - User selects a detected runtime.
   - Undetected runtimes are not normal dropdown options.

4. AI Connection Check
   - Nagare validates that at least one usable AI runtime is connected.
   - If auth is required or connection fails, the user fixes that before work
     creation.

5. Work Home
   - User sees work by state: action required, in progress, questions, completed.
   - New work is available once setup and connection checks pass.

6. Work Request Composer
   - The request text is the primary input.
   - Project and mode are secondary.
   - Automatic structuring is recorded, but should not distract from the request.

7. Work Start Confirmation
   - User confirms the prompt summary, AI composition, permissions, and review
     gates.
   - Primary action starts the work.

8. Work Running
   - User tracks the selected step, responsible agent, current conclusion, and
     next expected action.
   - If a question is needed, transition to a human-answer state.
   - If a failure occurs, transition to failure handoff.

9. Artifact Review
   - User reviews the produced artifact, criteria, evidence, and decision.
   - Primary action adopts the artifact.
   - Secondary actions request more information or reject/return the work.

10. Work Completed Summary
    - The work list shows completion badges and answer summary.
    - Full detail opens from a Detail action.

11. Work Detail
    - User reviews the final conclusion, adopted artifacts, review result, and
      agent history.

## Recovery And Improvement

- Human Question: blocks only the current work; after answering, work returns to
  running or review.
- Failure Handoff: shows cause, impact, recovery option, handoff context, and log
  summary. Reopen via start confirmation when configuration changes are needed.
- Quality Records: captures repeated review results and can propose domain,
  agent, or rubric changes.
- Quality Change Preview: improvements should be reviewed before applying.

## Management Areas

- Projects: list projects and configure organizer fallback.
- Domains: list domains; edit domain basics and artifact-specific knowledge or
  rubrics.
- Runtimes: list connected runtimes and model settings.
- Agents: list agents; edit role, runtime, prompt, domain scope, and assignment.
- Skills: manage installed skills and add skills from predefined lists, ClawHub
  search, or GitHub URL/manual source.
- MCP: manage MCP connections independently from skills; add/test MCPs before
  assigning them to agents.
- Inspectors: evidence, AI composition, and raw logs are diagnostic surfaces and
  should stay out of the default workflow unless explicitly opened.

## Current Known Cleanup

- Old screens numbered 01-44 represent earlier concepts and should not drive the
  new UI.
- Recent adopted screens 49-96 captured useful decisions, but many were detailed
  visual comps rather than wireframe-first flow contracts.
- The next pass should replace screen-by-screen visual assets with a smaller
  wireframe flow that proves navigation, state, validation, and user decisions.
