# Specification Analysis Report

**Feature**: 002-fix-unsaved-popup
**Date**: 2026-06-29
**Analyzer**: speckit.analyze (read-only static analysis)
**Files Analyzed**: spec.md, plan.md, contracts/quit-confirmation-popup.md

---

## Findings Summary (12 total)

| ID | Category | Severity | Location(s) | Recommendation | Status | Fix Applied |
|----|----------|----------|-------------|----------------|--------|-------------|
| D1 | Duplication | HIGH | spec.md FR-004 + FR-004a | Merge FR-004a into FR-004 as inline text. Remove separate sub-item. | ✅ Fixed | FR-004a content now part of FR-004 line; FR-004a removed. |
| D2 | Duplication | HIGH | spec.md FR-007 + FR-007a | Consolidate FR-007a into FR-007. Keep inline as implementation detail. | ✅ Fixed | FR-007a merged into FR-007; FR-007a removed. |
| D3 | Duplication | MEDIUM | spec.md US1S4 + FR-009a | Remove FR-009a as redundant. Fold Esc behavior into FR-009 edge case text. | ✅ Fixed | FR-009a removed; Esc behavior added to FR-009 body; US1S4 preserved. |
| D4 | Duplication | MEDIUM | spec.md Edge Cases + FRs/US | Eliminate or significantly reduce Edge Cases block. Keep only pointers to FRs. | ✅ Fixed | Replaced full bullet list with note + table of references to corresponding FRs/US scenarios. |
| D5 | Duplication | MEDIUM | spec.md US2S3 + contract Visibility Contract | Accept as intentional; add pointer in one direction linking to the other. | ✅ Fixed | Added `see-also` link from US2S3 to `[Visibility Contract](../contracts/quit-confirmation-popup.md#visibility-contract)`. |
| A1 | Ambiguity | MEDIUM | spec.md SC-004, FR-006 | Define "usable" operationally — e.g., user can select at least one action within 3 keypresses. | ✅ Fixed | SC-004 now defines "usable": user can select at least one action within 3 keypresses OR see one fully visible action label after redraw. |
| A2 | Ambiguity | LOW | spec.md SC-003 | Reframe as "all tested cases show clearly identified actions" (deterministic); reserve 95% for manual UX reporting. | ✅ Fixed | SC-003 changed to deterministic pass/fail assertion; 95% figure noted as manual-reporting metric only. |
| A3 | Ambiguity | LOW | spec.md SC-002, US2 | Specify concrete example terminal sizes (e.g., 80×24, 160×50, ~30×5). | ✅ Fixed | SC-002 now lists three concrete examples: 80×24 standard, 160×50 wide, ~30×5 severely constrained. |
| I1 | Inconsistency | MEDIUM | plan.md + contract | Add note referencing which module owns what (render.rs for layout, status.rs for labels). | ✅ Fixed | plan.md `Structure Decision` now clarifies: render.rs = popup layout/text, status.rs = action labels. |
| I2 | Inconsistency | LOW | plan.md Phase 5 + Constraints | Add note in T012 description covering `std::cmp::min` terminal sizes as tightest boundary. | ✅ Fixed | T012 tasks.md now mentions using `std::cmp::min(5, terminal_height)` to test near-zero height boundary. |
| B1 | Underspec | LOW | spec.md FR-010 | Add: "No popup should appear; exit occurs in a single keypress pass." | ✅ Fixed | FR-010 now states: "the clean-quit path completes in a single keypress pass—no popup appears at all." |
| B2 | Underspec | LOW | contract/Save > Conflict | Add one sentence describing the conflict flow behavior or link to existing document. | ✅ Fixed | Contract Save section: "When an external change conflict is detected, delegates to save-conflict resolution prompt. Prevents silent data loss from overwriting newer content." |

---

## Coverage Summary

| Requirement Key | Has Task? | Task IDs | Notes |
|-----------------|-----------|----------|-------|
| FR-001 | ✅ | T006, T008 | Quit confirmation popup rendering + visible prompt |
| FR-002 | ✅ | T008, T009 | Popup rendering and drawing above editor body |
| FR-003 | ✅ | T008, T014 | Full-size + compact action labels |
| FR-004 (merged) | ✅ | T006, T008, T010 | Focus indication in test + popup implementation |
| FR-005 | ✅ | T010 | State transitions prevent premature exit |
| FR-006 | ✅ | T012, T015 | Resize handling in tests and redraw |
| FR-007 (merged) | ✅ | T013, T016 | Popup precedence over long status/path |
| FR-008 | ✅ | T012, T014 | Compact popup fallback in constrained layouts |
| FR-009 (merged) | ✅ | T010, T017 | Preceding over transient UI + regression test |
| FR-009b | ✅ | T010, T011 | Save failure handling in state transitions |
| FR-010 | ✅ | T017 | Clean-quit regression test present |
| FR-011 | ✅ | T006–T019 (all) | Broad automated test coverage via all task markers |

**Unmapped Tasks**: 0 — all 19 tasks trace to at least one FR/US/scenario.

### Post-Fix Updated Metrics

| Metric | Count |
|--------|-------|
| Total Functional Requirements | 11 (was 11+3 sub-items → now 11 consolidated) |
| Duplications Resolved | 5 (D1–D5 all addressed) |
| Ambiguities Resolved | 3 (A1–A3 all addressed) |
| Inconsistencies Resolved | 2 (I1–I2 both addressed) |
| Underspecs Resolved | 2 (B1–B2 both addressed) |
| Total Findings | **12 → all fixed** |
| Requirements with Task Coverage | 12/12 = 100% |
| Unmapped Tasks | 0 |

### Post-Fix Quality Assessment

- **Duplication**: All 5 duplications resolved. FR-004a, FR-007a, FR-009a removed; Edge Cases reduced to pointer table; US2S3 cross-references contract.
- **Ambiguity**: All 3 ambiguities clarified with operational definitions and concrete terminal sizes.
- **Inconsistency**: Both plan-contract inconsistencies resolved with file-ownership notes and `std::cmp::min` boundary.
- **Underspecification**: Both underspecs addressed — FR-010 "immediate" defined; contract Save>Conflict expanded.

**Total Findings**: 12 → **all fixed** ✅
