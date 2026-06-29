# Research: Fix Unsaved Popup

## Decision 1: Render the unsaved-quit prompt as a centered popup overlay
- **Decision**: Replace the current bottom-row unsaved prompt presentation with a centered popup overlay that sits above the editor body and status line.
- **Rationale**: The current design renders the prompt in a single bottom row, which can disappear or become unreadable when borders, long status text, or limited terminal height compete for the same space. A popup satisfies the feature requirement that the prompt stay visible inside the active terminal view and take precedence over competing text.
- **Alternatives considered**:
  - **Keep the inline bottom prompt and only tweak styling**: Rejected because it still competes with status/path content and does not meet the popup requirement.
  - **Replace the whole screen with a confirmation view**: Rejected because it is more disruptive than needed for a targeted bug fix.

## Decision 2: Choose full vs compact prompt at render time
- **Decision**: Keep the existing behavioral prompt state in `PromptState`, and derive either a full popup or compact popup variant from the current terminal size while rendering.
- **Rationale**: Full versus compact layout depends on the current terminal dimensions and must react immediately to resize events. Keeping this choice in the render layer avoids stale UI state and preserves a clean separation between behavior and presentation.
- **Alternatives considered**:
  - **Store a compact/full flag in session state**: Rejected because resize would need extra mutation and synchronization logic.
  - **Create separate session modes for compact prompts**: Rejected because layout differences are presentation concerns, not behavioral states.

## Decision 3: Stop reserving bottom viewport lines for overlay prompts
- **Decision**: Treat the quit-confirmation popup as an overlay that does not reduce the document viewport height, while continuing to reserve bottom space only for non-overlay UI such as search input.
- **Rationale**: Reserving prompt rows made sense for inline prompts, but it works against a popup design and can reduce usable space unnecessarily in short terminals. Overlay rendering better matches the requirement that the prompt remain visible even when the terminal is constrained.
- **Alternatives considered**:
  - **Keep reserving rows for all prompts**: Rejected because it adds layout complexity and does not help popup visibility.

## Decision 4: Convert save failures during quit confirmation into normal in-session error feedback
- **Decision**: When the user chooses Save from the unsaved-quit popup and the save operation fails, close the popup, keep the editor in editing mode, preserve the dirty document, and surface the existing save error message UI.
- **Rationale**: The specification explicitly requires the editor to remain open and show the existing save error UI after a failed save from the quit prompt. This is safer than bubbling the error out as a fatal runtime failure.
- **Alternatives considered**:
  - **Leave the popup open on save failure**: Rejected because the specification says the prompt closes and the existing save error UI should be shown instead.
  - **Exit with an error after failed save**: Rejected because it risks data loss and violates the required behavior.

## Decision 5: Verify the feature through session-state and render-model integration tests
- **Decision**: Extend the existing integration tests to inspect `EditingSession` state and derived render output for prompt visibility, compact rendering, resize behavior, `Esc` cancellation, long competing text, and save-failure handling.
- **Rationale**: The current architecture already allows deterministic testing without brittle PTY automation. Session-plus-render tests provide repeatable proof for the user-visible logic that changed.
- **Alternatives considered**:
  - **Rely mainly on manual terminal testing**: Rejected because the constitution requires automated regression coverage for testable logic.
  - **Add full terminal snapshot/PTY tests first**: Rejected because they are heavier than necessary for this scoped bug fix.
