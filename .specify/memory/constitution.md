<!--
Sync Impact Report
- Version change: 1.0.0 -> 1.1.0
- Modified principles:
  - IV. Verification Before Merge -> IV. Verification Before Merge
- Added sections:
  - None
- Removed sections:
  - None
- Templates requiring updates:
  - ✅ updated: .specify/templates/plan-template.md
  - ✅ updated: .specify/templates/spec-template.md
  - ✅ updated: .specify/templates/tasks-template.md
  - ✅ updated: specs/001-shell-text-editor/plan.md
  - ✅ checked, no change needed: README.md
  - ✅ checked, no change needed: AGENTS.md
  - ✅ checked, no files present: .specify/templates/commands/
- Follow-up TODOs:
  - None
-->
# Gobo Constitution

## Core Principles

### I. Readability First
All production code, tests, and docs MUST optimize for fast human understanding.
Names MUST reveal intent, control flow MUST stay straightforward, and files,
functions, and types MUST stay small enough to review without reconstructing hidden
state. Cleverness, dense abstractions, and surprising behavior MUST be rejected unless
they are the simplest way to satisfy a proven requirement, and that exception MUST be
explained in the relevant plan or review. Rationale: `gobo` is a local editor that
must stay easy to inspect and evolve under terminal-specific complexity.

### II. Maintainable Design
Each change MUST preserve clear boundaries between responsibilities such as CLI entry,
application lifecycle, editor state, rendering, persistence, and input handling.
Modules MUST have one primary reason to change. Shared logic MUST be extracted only
when it reduces duplication without hiding behavior. New dependencies or abstractions
MUST come with a short justification covering why they improve maintenance more than
simple direct code would. Rationale: maintainability depends on predictable structure,
not on abstraction count.

### III. Secure by Default
The project MUST fail safely. Untrusted or external inputs, including file contents,
file paths, terminal events, environment-derived configuration, and dependency output,
MUST be validated or handled defensively. Destructive actions MUST require explicit
user intent when data loss is possible. File writes MUST preserve integrity by avoiding
silent overwrite of externally changed content and by surfacing permission or encoding
errors clearly. Secrets, credentials, or telemetry that expose user document contents
MUST NOT be introduced without an explicit constitutional amendment. Rationale: good IT
security for a local editor means minimizing unsafe behavior, protecting user data, and
avoiding accidental disclosure.

### IV. Verification Before Merge
Every user-visible feature, behavior change, bug fix, and safety-relevant path MUST be
secured with automated tests that prove the intended outcome and the relevant edge
cases. Edge-case coverage MUST include invalid input, boundary conditions, permission or
I/O failures, destructive-action safeguards, and regression scenarios whenever those
risks apply. Manual validation MAY supplement tests for terminal UX, but it MUST NOT
replace automated checks for logic that can be tested repeatably. Bugs MUST be
reproduced with a failing test or a documented manual procedure before being marked
fixed, and any fix for testable logic MUST add or update automated coverage. Rationale:
readable and secure code still fails without disciplined proof, especially when edge
cases are left untested.

### V. Scope and Simplicity Control
The default decision MUST be to keep `gobo` a focused, single-binary, local-first,
single-document editor unless a broader scope is explicitly approved. Features that add
persistent background services, network dependence, plugin systems, multi-document
orchestration, or other major complexity MUST be rejected or justified as a documented
exception. When several valid designs exist, the one with the fewest moving parts and
clearest operational model MUST win. Rationale: strong scope control protects
readability, maintainability, and security simultaneously.

## Operational Constraints

- The project MUST remain understandable to a new contributor by keeping the active
  architecture and major module responsibilities documented in current plans or code
  comments near the relevant entry points.
- The default runtime model is local execution with no required external services.
- User-visible failure paths MUST provide clear feedback and MUST leave persisted data
  in a safe, explainable state.
- Dependencies MUST be kept minimal and updated intentionally; each new dependency MUST
  solve a concrete problem that is harder to solve safely in project code.
- Sensitive document content MUST NOT be transmitted, logged, or stored outside the
  user's intended file operations unless explicitly specified and approved.

## Development Workflow

- Every plan MUST pass a constitution check covering readability, maintainability,
  security, verification, and scope.
- Every spec MUST define edge cases for invalid input, failure handling, permission
  boundaries, destructive actions, and other feature-specific boundary conditions.
- Every task list MUST include explicit automated test work that covers each feature's
  main flow and relevant edge cases, plus any required safeguards for safety,
  persistence, permissions, or user data.
- Code review MUST reject changes that increase coupling, hide control flow, weaken
  failure handling, or add security-sensitive behavior without clear need.
- Before merge, contributors MUST confirm that documentation, tests, and implementation
  all reflect the same behavior.

## Governance

This constitution supersedes conflicting local habits and planning defaults.
Amendments MUST be documented in `.specify/memory/constitution.md`, include a clear
reason, and update any affected templates, plans, or guidance files in the same change
when feasible. Compliance reviews MUST happen during planning, task generation, code
review, and before merge for behavior-changing work.

Versioning policy follows semantic versioning for governance changes:
- MAJOR: remove or redefine a principle or governance rule in a backward-incompatible
  way.
- MINOR: add a new principle, section, or materially stronger requirement.
- PATCH: clarify wording, fix inconsistencies, or improve guidance without changing
  meaning.

Compliance review MUST confirm that planned and delivered automated tests cover every
implemented feature and its relevant edge cases before merge.

If a change cannot comply, the plan or review record MUST document the exception, why
it is necessary, the simpler alternative that was rejected, and the follow-up needed to
return to compliance.

**Version**: 1.1.0 | **Ratified**: 2026-06-29 | **Last Amended**: 2026-06-29
