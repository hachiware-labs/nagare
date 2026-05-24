# Nagare Tutorial

[日本語](tutorial_ja.md)

This tutorial covers the first completed user scenario: a failing Codex agent
profile attempt, evidence capture, handoff to a Codex App Server agent profile, successful retry,
verification, human approval, and final `done` state.

## Prerequisites

- `nagare` command installed and available on `PATH`
- Codex CLI

## 1. Check the CLI

```powershell
nagare version
```

## 2. Initialize a Nagare Project

```powershell
nagare init
```

This creates:

```text
.nagare/
  project.toml
  agents/
  artifacts/
  logs/
  state/
```

## 3. Check the Environment

```powershell
nagare doctor
nagare locale show
```

The command reports the current project root, whether `.nagare/project.toml`
exists, and whether common local tools are available.

## 4. Create a Work Item

Register project-local Agent Profiles first:

```powershell
nagare locale use --language en-US --timezone America/Los_Angeles
nagare agent add --id codex-impl-smoke --display-name "Codex CLI Smoke Implementer" --runtime codex-local --adapter process.codex-cli --role implementer --working-dir . --description "Implementation and verification" --specialties implementation,verification
nagare agent add --id codex-app-smoke --display-name "Codex App Server Smoke Implementer" --runtime codex-app-local --adapter stdio.codex-app-server --role implementer --working-dir . --description "Planning and review" --specialties planning,review
nagare agent list
nagare agent use --work-agent codex-impl-smoke --review-agent codex-app-smoke --dispatch-agent codex-impl-smoke
nagare agent defaults
nagare agent doctor codex-impl-smoke
nagare agent probe codex-impl-smoke
nagare rule check README.md
```

```powershell
nagare item create --title "Repair failing agent run" --description "Demonstrate cross-agent evidence and handoff."
```

Expected output:

```text
created work_0001 ready
```

## 5. Run the First Agent Attempt

```powershell
nagare item preview work_0001 --command "echo dispatch preview && exit /B 0"
nagare item dispatch accept work_0001
```

Preview stores a `draft` DispatchPlan. Accepting it allows `item run` to use
the selected target agent by default.

```powershell
nagare item run work_0001 --command "echo codex attempt failed && exit /B 1"
```

The command records a failed run and evidence:

```text
run run_0002 failed agent=codex-impl-smoke
```

## 6. Create a Handoff

```powershell
nagare handoff create work_0001 --from-agent codex-impl-smoke --to-agent codex-app-smoke --reason "Codex agent profile produced a failing run" --summary "Retry with Codex App Server agent profile using the captured run log as evidence."
```

## 7. Retry with Another Agent Profile

```powershell
nagare item run work_0001 --agent codex-app-smoke --command "echo codex app server retry fixed the task && exit /B 0"
```

## 8. Verify the Work

```powershell
nagare verify work_0001 --command "echo verification passed && exit /B 0"
```

## 9. Approve the Work

```powershell
nagare decision approve work_0001 --rationale "Required verification passed after cross-agent handoff."
```

Inspect the resulting work item:

```powershell
nagare item show work_0001
```

The snapshot should include:

```text
runs:
  run_0002  codex-impl-smoke     failed
  run_0006  codex-app-smoke  succeeded
evidence:
  Agent run failed with profile `codex-impl-smoke`
  Agent run succeeded with profile `codex-app-smoke`
  Verification passed
handoffs:
  codex-impl-smoke -> codex-app-smoke
decisions:
  approve
```

## 10. Run in a Temporary Root

```powershell
$tmp = Join-Path $env:TEMP "nagare-first"
$env:NAGARE_ROOT = $tmp
nagare init
nagare locale use --language en-US --timezone America/Los_Angeles
nagare agent add --id codex-impl-smoke --runtime codex-local --adapter process.codex-cli --working-dir . --description "Implementation and verification" --specialties implementation,verification
nagare agent add --id codex-app-smoke --runtime codex-app-local --adapter stdio.codex-app-server --working-dir . --description "Planning and review" --specialties planning,review
nagare agent use --work-agent codex-impl-smoke --review-agent codex-app-smoke --dispatch-agent codex-impl-smoke
nagare agent probe codex-impl-smoke
nagare item create --title "Repair failing agent run"
nagare item preview work_0001 --command "echo dispatch preview && exit /B 0"
nagare item dispatch accept work_0001
nagare item run work_0001 --command "echo codex attempt failed && exit /B 1"
nagare handoff create work_0001 --from-agent codex-impl-smoke --to-agent codex-app-smoke --reason "Codex agent profile produced a failing run"
nagare item run work_0001 --agent codex-app-smoke --command "echo codex app server retry fixed the task && exit /B 0"
nagare verify work_0001 --command "echo verification passed && exit /B 0"
nagare decision approve work_0001
nagare item show work_0001
Remove-Item Env:\NAGARE_ROOT
```

## Real Codex App Server

For a live Codex App Server run, use `--prompt` with an Agent Profile whose
adapter is `stdio.codex-app-server`. Nagare starts `codex app-server --listen
stdio://`, creates a thread, starts a turn, and records the app-server transcript
as the run artifact.
