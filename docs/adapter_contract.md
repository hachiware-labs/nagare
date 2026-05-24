# Agent Adapter Contract

Nagare treats every agent runtime as an adapter behind a stable execution
contract. The product should not depend on one vendor's CLI, UI, memory model,
permission system, or plugin ecosystem.

This contract sits under the broader [architecture](architecture.md) and
[Agent Management Model](agent_management.md).
The management model chooses runtime, agent profile, skill set, policy, and
project rule before the adapter receives a run packet.

## Design Position

Nagare owns:

- Work Item
- Run Packet
- Workspace
- Agent Run
- Run Event
- Artifact
- Evidence
- Verification Result
- Handoff Packet
- Human Decision

Agent runtimes own:

- model choice
- internal planning
- tool execution details
- native session state
- native permission prompts

The adapter translates between those two worlds.

## Adapter Lifecycle

```text
healthcheck() -> AdapterHealth
prepare(run_packet, workspace_policy, permission_policy) -> PreparedRun
start(prepared_run) -> ExternalRunId
stream(external_run_id) -> RunEvent[]
collect(external_run_id) -> CollectedOutput
cancel(external_run_id) -> CancelResult
```

## Initial Adapter Priority

Supported agent adapters:

- `process.codex-cli`
- `stdio.codex-app-server`

Not agent adapters:

- shell commands used by `nagare verify --command`
- local smoke-test commands
- SDK wrappers in other languages

Explicitly out of initial scope:

- `process.opencode-run`
- `cloud.codex`
- HTTP worker adapters
- OpenCode HTTP server adapters
- `process.claude-code`
- `sdk.claude-agent`
- Codex MCP Server

Codex has a server-style entry point through `codex app-server`. Nagare should
prefer the default stdio transport because it avoids an HTTP control plane while
exposing Codex-specific threads, turns, approval flow, and streamed events.
Codex MCP Server is not used as a Nagare agent adapter. SDKs in other languages
are integration helpers, not Nagare agent adapters.

## Normalized Run Event Types

```text
run.started
run.heartbeat
run.stdout
run.stderr
run.tool_call.started
run.tool_call.completed
run.artifact.produced
run.evidence.produced
run.verification.produced
run.needs_input
run.completed
run.failed
run.timed_out
run.canceled
run.protocol_violation
```

## Minimum Run Packet

```yaml
run_packet:
  id: runpkt_001
  work_item_id: work_001
  goal: "Fix failing auth tests with the smallest safe change"
  workspace:
    kind: worktree
    path: ".nagare/workspaces/work_001"
    branch: "nagare/work_001"
  working_dir: ".nagare/workspaces/work_001"
  constraints:
    - "Do not push to main"
    - "Do not access production credentials"
  expected_artifacts:
    - diff
    - test_log
    - summary
  verification:
    - type: command
      command: "npm test"
      expected: "exit_code_0"
  reporting:
    artifact_sink: "nagare"
    evidence_sink: "nagare"
```

## Adapter Acceptance Tests

Every adapter must pass the same fixtures:

- healthcheck reports missing dependency clearly
- start creates one Nagare Agent Run
- stdout/stderr are captured as run events
- command failure creates failed run status
- produced files can be collected as artifacts
- cancel is idempotent or explicitly unsupported
- adapter never marks a Work Item done directly
