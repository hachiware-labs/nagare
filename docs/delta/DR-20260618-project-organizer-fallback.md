# Project Organizer Fallback

## Decision

Nagare projects may set a project-specific organizer agent. If a project has no organizer configured, Nagare uses a built-in organizer instead of blocking work creation.

This fallback must be visible in the UI. The project settings screen should show that the project currently uses the built-in organizer and that a project-specific organizer is not configured.

## Rationale

Users should be able to start a work item soon after creating a project. Requiring an organizer agent before the first run adds setup friction.

At the same time, the fallback must not be hidden. If a project uses the built-in organizer, users need to understand which session receives requests handed off from the Nagare or Tomoshibikan entry point.

## UX Model

1. The user sends a request to Nagare or Tomoshibikan.
2. The entry session identifies or asks for the target project.
3. Nagare routes the request to the project's organizer session.
4. If the project has no organizer set, Nagare uses the built-in organizer.
5. The organizer creates or refines the Work Item and assigns worker or reviewer agents.

## UI Implications

- Project settings include an `Organizer` section.
- The selector includes `Use built-in organizer`.
- The current fallback state is shown as `Project-specific organizer not configured`.
- Candidate project organizer agents are limited to agents with the organizer role.
- Creating a new organizer is a secondary action from the same section.

## Implementation Mapping

- Config key: `[nagare_agents].organizer_agent`
- Existing fallback key: `[nagare_agents].dispatch_agent`
- CLI update command: `nagare agent use --organizer-agent <agent-profile-id>`
- Runtime behavior: workflow dispatch decisions use `organizer_agent` when set, otherwise `dispatch_agent`.
- Compatibility: `item preview` and handoff dispatch commands continue to use `dispatch_agent` unless explicitly overridden.

Reference screen:

- `docs/design-assets/svg/54-project-organizer-settings.svg`
