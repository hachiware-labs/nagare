# Nagare / 流

![Nagare logo](logo.png)

[日本語 README](README_ja.md) | [PRD](docs/nagare_prd_v1_0.md) | [UI Prototype](docs/design-assets/prototype/) | [Trace Schema (NF-2)](docs/nagare_trace_schema_v1_0.md) | [Archived docs](docs/archive/)

Nagare is an adapter-first execution ledger for coding agents.

The goal is to keep work items, run packets, agent runs, artifacts, evidence,
review results, handoffs, and human decisions in one local-first control
layer while letting agent backends change underneath.

## Use Nagare as an Agent Skill

This repository ships a skill (`skills/nagare/SKILL.md`) that teaches coding
agents such as Claude Code or Codex CLI how to drive the `nagare` ledger.

**What the skill is for** — it makes agent work *auditable*: every task leaves
a structured local record of who ran what, with which inputs and artifacts,
how the result was reviewed, and what the human decided. Nagare records
decisions, not transport: no raw API traffic is captured.

**Install**

```bash
# 1. Install the skill into your agent environment
npx skills add hachiware-labs/nagare

# 2. Install the CLI the skill drives
npm install -g @hachiware-labs/nagare
nagare doctor
```

**How to use** — after installing, ask your agent things like:

- "Track this refactoring in Nagare and get it reviewed before finishing."
- "Record this task as a work item, run it, and hand off to another agent if it fails."
- "Show me the Nagare ledger state for work_0001."

The agent will initialize the ledger (`nagare init`), register agent
profiles, create work items, record runs and failures as evidence, create
handoffs between agents, run reviews, and stop for your explicit approval
before a work item reaches `done`.

**When it pays off**

- Work that needs a paper trail: client deliverables, release tasks,
  anything you may need to explain afterwards.
- Multi-agent setups: switching between Claude Code / Codex etc. while
  keeping one continuous history, including failure handoffs with context.
- Review-gated work: results must pass a review and an explicit human
  decision instead of being silently accepted.

## Current Slice

This repository now includes the first end-to-end user scenario:

- initialize a local Nagare ledger
- register project-local Agent Profiles in `.nagare/agents/*.toml`
- create a work item
- run a failing `codex-cli` agent profile
- capture the failure as evidence
- create a handoff to `codex-app-server`
- run a succeeding retry
- review the work, including any CI/test/artifact checks
- approve it as a human decision
- reach `done`

## Local Development

```powershell
npm test
npm run build
nagare doctor
nagare init
```

`npm run build` builds the release CLI binary, stages it into the local npm
package, and links the workspace package globally so `nagare` runs the current
development build.

## First Scenario

Run the scenario as normal user commands:

```powershell
$env:NAGARE_ROOT = "$env:TEMP\nagare-first"
nagare init
nagare locale use --language en-US --timezone America/Los_Angeles
nagare agent add --id codex-impl-smoke --display-name "Codex CLI Smoke Implementer" --runtime codex-local --adapter process.codex-cli --role worker --working-dir . --description "Implementation and review checks" --specialties implementation,review-checks
nagare agent add --id codex-app-smoke --display-name "Codex App Server Smoke Implementer" --runtime codex-app-local --adapter stdio.codex-app-server --role implementer --working-dir . --description "Planning and review" --specialties planning,review
nagare agent list
nagare agent use --work-agent codex-impl-smoke --review-agent codex-app-smoke --dispatch-agent codex-impl-smoke
nagare agent defaults
nagare agent doctor codex-impl-smoke
nagare agent probe codex-impl-smoke
nagare item create --title "Repair failing agent run" --description "Demonstrate cross-agent evidence and handoff."
nagare item preview work_0001 --command "echo dispatch preview && exit /B 0"
nagare item dispatch accept work_0001
nagare item run work_0001 --command "echo codex run failed && exit /B 1"
nagare handoff create work_0001 --from-agent codex-impl-smoke --to-agent codex-app-smoke --reason "Codex agent profile produced a failing run" --summary "Retry with Codex App Server agent profile using the captured run log as evidence."
nagare item run work_0001 --agent codex-app-smoke --command "echo codex app server retry fixed the task && exit /B 0"
nagare item review work_0001 --agent codex-app-smoke --command "echo ## Nagare Review && echo verdict: pass && echo summary: && echo - review passed && echo completed: && echo - reviewed result and checks && echo findings: && echo - none && echo questions: && echo next_notes: && echo - ready for approval && echo next_action: approve"
nagare decision approve work_0001 --rationale "Required review passed after cross-agent handoff."
nagare item show work_0001
Remove-Item Env:\NAGARE_ROOT
```

Expected snapshot header:

```text
work_0001	done	Repair failing agent run
```

The scenario uses registered agent profile IDs while running local demo
commands. This keeps the first workflow deterministic while preserving the
adapter-first shape of the product. Unknown agent profile IDs are rejected.

## `nagare` Command

After installation, all user-facing flows are available through the `nagare`
command:

```powershell
nagare doctor
nagare init
nagare locale show
nagare agent list
nagare agent show codex-cli
nagare agent defaults
nagare agent doctor codex-cli
nagare agent probe codex-cli
nagare item preview work_0001
nagare item dispatch accept work_0001
nagare item review work_0001
nagare handoff dispatch work_0001
nagare item list
nagare item show work_0001
```

The npm package is only the installation/distribution path. The product
interface is the `nagare` command.

## Documentation Language Policy

User-facing README documents are maintained in English and Japanese pairs:

- `README.md` / `README_ja.md`

The canonical product design is maintained in Japanese:

- `docs/nagare_prd_v1_0.md` (PRD and feature list)
- `docs/nagare_trace_schema_v1_0.md` (NF-2 trace schema)
- `docs/design-assets/prototype/` (UI source of truth)

Pre-redesign documents (spec, architecture, tutorials, old wireframes) are
kept under `docs/archive/`.
