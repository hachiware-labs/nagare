---
name: nagare
description: Local-first execution ledger for coding-agent work. Use when work should leave an auditable record — which agent ran what, with which inputs and artifacts, how the result was reviewed, and what the human decided — especially across multiple agent backends (Claude Code, Codex CLI, etc.), with failure handoffs between agents and a human approval gate before completion.
---

# Nagare — Execution Ledger for Agent Work

Use this skill to run work through the `nagare` CLI so that every step leaves
a structured, local-first record: work items, agent runs, artifacts, evidence,
review results, handoffs, and human decisions.

## Purpose

Nagare records **decisions, not transport**: who was assigned and why, what
inputs produced which artifacts, what the reviewer concluded, and what the
human approved. It does not capture raw API traffic. Use it when the user
needs their agent work to be reviewable, resumable, and auditable afterwards.

## When to use

- The user wants a persistent record of agent work ("record this work",
  "track this task", "make this auditable").
- Work spans multiple agent backends, or a failing run should be handed off
  to another agent profile with its evidence preserved.
- Results must pass a review and an explicit human approval before the work
  is considered done.

## Requirements

- `nagare` CLI on PATH. Install: `npm install -g @hachiware-labs/nagare`
- Verify with `nagare doctor`. Initialize a ledger once per workspace with
  `nagare init` (state lives under `.nagare/`).

## Core flow

1. **Initialize once**: `nagare init`, then register agent profiles:
   `nagare agent add --id <id> --display-name <name> --runtime <rt> --adapter <adapter> --role <worker|implementer> --working-dir . --description <text> --specialties <a,b>`
   and set defaults with `nagare agent use --work-agent <id> --review-agent <id> --dispatch-agent <id>`.
2. **Create the work item**: `nagare item create --title <title> --description <desc>`.
3. **Preview and accept dispatch**: `nagare item preview <work_id>` then
   `nagare item dispatch accept <work_id>`.
4. **Run**: `nagare item run <work_id> [--agent <id>] [--command <cmd>]`.
5. **On failure, hand off instead of silently retrying**:
   `nagare handoff create <work_id> --from-agent <a> --to-agent <b> --reason <why> --summary <handoff context>`,
   then run again with the target agent. The failure stays recorded as evidence.
6. **Review**: `nagare item review <work_id> --agent <reviewer-id> ...` and
   ensure the review output states a verdict, summary, findings, and next action.
7. **Human decision**: only the user approves. Ask them, then record it:
   `nagare decision approve <work_id> --rationale <their reason>`.
8. **Confirm**: `nagare item show <work_id>` should report the final state
   (`done` after approval).

## Recording principles

- Never skip the review or the human decision steps; they are the point.
- Name concrete artifacts and evidence in run/review summaries (file paths,
  not vague descriptions).
- Unknown agent profile IDs are rejected — register profiles before use.
- Inspect state at any time with `nagare item list`, `nagare item show <id>`,
  `nagare agent list`, `nagare agent defaults`.

## Reporting

After driving a flow, report to the user: the `work_id`, its current state,
which agents ran, where the artifacts are, the review verdict, and what
decision (if any) is still pending from them.
