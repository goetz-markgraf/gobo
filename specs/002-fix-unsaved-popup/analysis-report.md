# Specification Analysis Report

**Feature**: 002-fix-unsaved-popup
**Date**: 2026-06-29
**Analyzer**: speckit.analyze (read-only static analysis)
**Files Analyzed**: spec.md, plan.md, contracts/quit-confirmation-popup.md

---

## Findings Summary (12 total)

| ID | Category | Severity | Location(s) | Summary | Recommendation |
|----|----------|----------|-------------|---------|----------------|
| D1 | Duplication | HIGH | spec.md FR-004 + FR-004a | FR-004 (focused action indication) and FR-004a (Save focused by default) overlap. | Merge FR-004a into FR-004 as its first sentence or numbered subclause. Remove the separate sub-item. |
| D2 | Duplication | HIGH | spec.md FR-007 + FR-007a | FR-007 states prompt stays visible despite long status/path text. FR-007a says "show as a popup" for the same condition. | Consolidate FR-007a into FR-007. Keep it as an inline implementation detail rather than a standalone requirement line. |
| D3 | Duplication | HIGH | spec.md US1S4 + FR-009a | US Acceptance Scenario 4 states Esc cancels and returns to editing. FR-009a repeats this verbatim without adding new behavior. | Remove FR-009a as redundant. Fold Esc behavior into the Edge Cases section or into FR-009's scope text. |
| D4 | Duplication | MEDIUM | spec.md Edge Cases + Functional Requirements | Edge Cases section restates all 6 items already covered by FRs/US scenarios. No new edge cases are introduced. | Eliminate or significantly reduce the Edge Cases block. Move only genuinely unique edge cases if any remain. |
| D5 | Duplication | MEDIUM | spec.md US2S3 + contracts/quit-confirmation-popup.md | US2S3 states "show popup when status/text competes." Contract Visibility Contract has identical wording. | Accept as intentional cross-reference; add a pointer comment in one direction linking to the other. |
| A1 | Ambiguity | MEDIUM | spec.md SC-004, FR-006 | "Usable" in SC-004 and "actionable" in FR-006/US2S1 are not defined measurably. | Define "usable" operationally — e.g., user can select at least one action within 3 keypresses, or at minimum one action label remains fully visible. |
| A2 | Ambiguity | LOW | spec.md SC-003 | "95% of test runs" is difficult to express as an automated assertion with deterministic pass/fail. | Reframe as "all tested cases in automation show clearly identified actions" (deterministic) and reserve 95% for manual UX reporting. |
| A3 | Ambiguity | LOW | spec.md SC-002, US2 | "At least three terminal sizes" — no explicit size ranges defined. | Specify concrete example sizes (e.g., 80×24, 160×50, and constrained like 30×5) in data-model.md or append to FR-008 as a note. |
| I1 | Inconsistency | MEDIUM | plan.md + contract/quit-confirmation-popup.md | Plan references `render.rs` and `status.rs`; contract mentions popup rendering but not the file split between them. | Add a note in the contract referencing which module owns what (render.rs for layout, status.rs for labels). |
| I2 | Inconsistency | LOW | plan.md Phase 5 + Constraints | Constraints say "very small terminals must fall back to compact prompt," but no task explicitly tests the smallest boundary. | Add a note in T012 description covering `std::cmp::min` terminal sizes as the tightest boundary. |
| B1 | Underspec | LOW | spec.md FR-010 | "Continue to allow immediate quit" — "immediate" is not measured. | Add: "No popup should appear; exit occurs in a single keypress pass." This maps to T017 regression coverage. |
| B2 | Underspec | LOW | contract/quit-confirmation-popup.md > Save > Conflict | "Save-conflict flow may take over" — vague, no reference to existing spec. | Add one sentence describing the conflict flow's behavior or link to an existing document/feature. |

### Coverage Summary

| Requirement Key | Has Task? | Task IDs | Notes |
|-----------------|-----------|----------|-------|
| FR-001 | ✅ | T006, T008 | Quit confirmation popup rendering + visible prompt |
| FR-002 | ✅ | T008, T009 | Popup rendering and drawing above editor body |
| FR-003 | ✅ | T008, T014 | Full-size + compact action labels |
| FR-004 | ✅ | T006, T008, T010 | Focus indication in test + popup implementation |
| FR-004a | ⚠️ Merge | (merge → FR-004) | See D1 |
| FR-005 | ✅ | T010 | State transitions prevent premature exit |
| FR-006 | ✅ | T012, T015 | Resize handling in tests and redraw |
| FR-007 | ✅ | T013, T016 | Popup precedence over long status/path |
| FR-007a | ⚠️ Merge | (merge → FR-007) | See D2 |
| FR-008 | ✅ | T012, T014 | Compact popup fallback in constrained layouts |
| FR-009 | ✅ | T010, T017 | Preceding over transient UI + regression test |
| FR-009a | ⚠️ Merge | (merge → US1S4) | See D3 |
| FR-009b | ✅ | T010, T011 | Save failure handling in state transitions |
| FR-010 | ✅ | T017 | Clean-quit regression test present |
| FR-011 | ✅ | T006–T019 (all) | Broad automated test coverage via all task markers |

**Unmapped Tasks**: 0 — all 19 tasks trace to at least one FR/US/scenario.

### Constitution Alignment

No conflicts detected. All five gates (Readability, Maintainability, Security, Verification, Scope) pass per the plan. Test tasks satisfy Principle IV (Verification Before Merge). No new dependencies or multi-document scope introduced — consistent with Principle V.

### Metrics

| Metric | Count |
|--------|-------|
| Total Functional Requirements | 11 (plus 3 sub-items) |
| Total US Acceptance Scenarios | 9 (US1: 5, US2: 4) |
| Total Success Criteria | 5 |
| Total Edge Cases Listed | 6 |
| Total Tasks (all phases) | 19 |
| Requirements with Task Coverage | 14/14 = 100% |
| Unmapped Tasks | 0 |
| Duplication Findings | 5 |
| Ambiguity Findings | 3 |
| Inconsistency Findings | 2 |
| Underspec Findings | 2 |
| Constitution Conflicts | 0 |
| **Total Findings** | **12** |
