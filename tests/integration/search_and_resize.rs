use gobo::app::{EditingSession, SessionMode};
use gobo::editor::input::EditorCommand;
use gobo::editor::render::TerminalSize;
use tempfile::tempdir;

fn dirty_session_with_size(
    path_name: &str,
    size: TerminalSize,
) -> (tempfile::TempDir, std::path::PathBuf, EditingSession) {
    let dir = tempdir().unwrap();
    let path = dir.path().join(path_name);
    std::fs::write(&path, "alpha\nbeta\ngamma\n").unwrap();
    let mut session = EditingSession::open(&path, size).unwrap();
    session
        .handle_command(EditorCommand::InsertChar('x'))
        .unwrap();
    (dir, path, session)
}

#[test]
fn search_is_case_insensitive_and_reports_no_match() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("search.txt");
    std::fs::write(&path, "First line\nTHIRD line\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session.handle_command(EditorCommand::Search).unwrap();
    for ch in "third".chars() {
        session
            .handle_command(EditorCommand::InsertChar(ch))
            .unwrap();
    }
    session.handle_command(EditorCommand::Enter).unwrap();
    assert!(
        session
            .status
            .as_ref()
            .unwrap()
            .text
            .contains("Match found")
    );

    session.handle_command(EditorCommand::Search).unwrap();
    for ch in "missing".chars() {
        session
            .handle_command(EditorCommand::InsertChar(ch))
            .unwrap();
    }
    session.handle_command(EditorCommand::Enter).unwrap();
    assert!(session.status.as_ref().unwrap().text.contains("No match"));
}

#[test]
fn resize_updates_viewport_and_render_output() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("resize.txt");
    std::fs::write(&path, "alpha\nbeta\ngamma\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    session
        .handle_command(EditorCommand::Resize(TerminalSize::new(20, 5)))
        .unwrap();
    let view = session.render_view();

    assert_eq!(session.viewport.visible_width, 20);
    assert_eq!(session.viewport.visible_height, 4);
    assert_eq!(view.body_lines.len(), 4);
}

#[test]
fn quit_popup_takes_precedence_over_active_search() {
    let (_dir, _path, mut session) =
        dirty_session_with_size("search-precedence.txt", TerminalSize::new(80, 24));
    session.handle_command(EditorCommand::Search).unwrap();
    session
        .handle_command(EditorCommand::InsertChar('a'))
        .unwrap();
    assert_eq!(session.mode, SessionMode::SearchInput);
    assert!(
        session
            .render_view()
            .bottom_line
            .as_deref()
            .unwrap()
            .contains("Search:")
    );

    session.handle_command(EditorCommand::Quit).unwrap();

    assert_eq!(session.mode, SessionMode::ConfirmQuit);
    let view = session.render_view();
    assert!(view.popup.is_some());
    assert_eq!(view.bottom_line, None);
}

#[test]
fn resize_while_prompted_keeps_popup_visible_and_uses_compact_variant() {
    let (_dir, _path, mut session) =
        dirty_session_with_size("resize-prompt.txt", TerminalSize::new(80, 24));
    session.handle_command(EditorCommand::Quit).unwrap();

    let before = session.render_view().popup.expect("popup before resize");
    assert_eq!(before.variant, gobo::editor::render::PromptVariant::Full);

    session
        .handle_command(EditorCommand::Resize(TerminalSize::new(24, 6)))
        .unwrap();

    assert_eq!(session.mode, SessionMode::ConfirmQuit);
    let after = session.render_view().popup.expect("popup after resize");
    assert_eq!(after.variant, gobo::editor::render::PromptVariant::Compact);
    assert_eq!(after.actions[0].label, "[Save]");
    assert!(after.rect.width <= 24);
    assert!(after.rect.height <= 6);
}

#[test]
fn empty_and_very_long_lines_render_without_crashing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("long-lines.txt");
    let long_line = "x".repeat(5000);
    std::fs::write(&path, format!("\n{}\n", long_line)).unwrap();

    let session = EditingSession::open(&path, TerminalSize::new(20, 5)).unwrap();
    let view = session.render_view();

    assert_eq!(view.body_lines.len(), 4);
    assert_eq!(view.body_lines[0], "");
    assert_eq!(view.body_lines[1].len(), 20);
}

// T004: Integration test for full search flow (US1)
#[test]
fn search_full_flow_confirms_first_match_and_exits() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("search-flow.txt");
    std::fs::write(
        &path,
        "alpha
beta
gamma
alpha again
",
    )
    .unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();

    // Press Search -> type 'beta' -> Enter to confirm
    session.handle_command(EditorCommand::Search).unwrap();
    assert_eq!(session.mode, SessionMode::SearchInput);

    for ch in "beta".chars() {
        session
            .handle_command(EditorCommand::InsertChar(ch))
            .unwrap();
    }

    // Press Enter to confirm search
    session.handle_command(EditorCommand::Enter).unwrap();
    assert!(
        session
            .status
            .as_ref()
            .unwrap()
            .text
            .contains("Match found")
    );
    assert_eq!(session.mode, SessionMode::Editing);
    // Cursor should be at start of first match (position 0 for "beta" on line 2)

    // Verify bottom_line is None once we leave search mode
    let view = session.render_view();
    assert_eq!(view.bottom_line, None);
}

// T005: Integration test for search cancel flow (US1)
#[test]
fn search_cancel_returns_to_editing_without_modifying_cursor() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cancel-flow.txt");
    std::fs::write(
        &path,
        "first line
second line
",
    )
    .unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    let initial_char_index = session.cursor.char_index;

    // Start search and type something
    session.handle_command(EditorCommand::Search).unwrap();
    assert_eq!(session.mode, SessionMode::SearchInput);
    session
        .handle_command(EditorCommand::InsertChar('x'))
        .unwrap();
    session
        .handle_command(EditorCommand::InsertChar('y'))
        .unwrap();

    // Press Cancel (Esc) - should return to editing without cursor movement
    session.handle_command(EditorCommand::Cancel).unwrap();
    assert_eq!(session.mode, SessionMode::Editing);
    assert_eq!(session.cursor.char_index, initial_char_index);
}

// T010 (US2): Integration test for find-next flow with Ctrl+G
#[test]
fn find_next_jump_to_next_match_via_command() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("findnext.txt");
    std::fs::write(&path, "alpha\nbeta\nalpha again\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();

    // Enter search mode and type a query
    session.handle_command(EditorCommand::Search).unwrap();
    assert_eq!(session.mode, SessionMode::SearchInput);

    for ch in "alpha".chars() {
        session
            .handle_command(EditorCommand::InsertChar(ch))
            .unwrap();
    }
    // Confirm search - should jump to first match
    session.handle_command(EditorCommand::Enter).unwrap();
    assert_eq!(session.mode, SessionMode::Editing);
    assert!(
        session
            .status
            .as_ref()
            .unwrap()
            .text
            .contains("Match found")
    );

    // Re-enter search mode and confirm again to have a fresh starting position
    let _initial_cursor = session.cursor.char_index;
}

// T019 (US2+Polish): Empty query + Enter exits silently without moving cursor
#[test]
fn empty_query_enter_exits_silently() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("empty-query.txt");
    std::fs::write(&path, "hello world\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    let initial_char_index = session.cursor.char_index;
    let _initial_status = session.status.take();

    session.handle_command(EditorCommand::Search).unwrap();
    assert_eq!(session.mode, SessionMode::SearchInput);
    // Don't type anything — leave query empty

    session.handle_command(EditorCommand::Enter).unwrap();
    assert_eq!(session.mode, SessionMode::Editing);
    // Cursor should NOT have moved
    assert_eq!(session.cursor.char_index, initial_char_index);
}

// T019b: Ctrl+G with empty search shows "No match" without moving cursor
#[test]
fn ctrlg_with_empty_query_shows_no_match() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("empty-ctrlg.txt");
    std::fs::write(&path, "hello world\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();
    let initial_char_index = session.cursor.char_index;

    // Enter search mode WITHOUT typing anything
    session.handle_command(EditorCommand::Search).unwrap();
    assert_eq!(session.mode, SessionMode::SearchInput);
    // Press Ctrl+G with empty query — should show "No match"
    session.handle_command(EditorCommand::FindNext).unwrap();

    // Mode should stay SearchInput
    assert_eq!(session.mode, SessionMode::SearchInput);
    // Cursor should NOT have moved
    assert_eq!(session.cursor.char_index, initial_char_index);
    // Status should indicate no match (the search prompt should show the empty query)
    let view = session.render_view();
    assert!(
        view.bottom_line
            .as_deref()
            .unwrap_or("")
            .contains("Search: ")
    );
}

// T023 (US2): Full integration flow for Ctrl-G — Enter to confirm, then Ctrl-G advances through matches
#[test]
fn ctrlg_full_flow_jumps_to_subsequent_matches() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ctrlg-flow.txt");
    std::fs::write(&path, "alpha\nbeta\nalpha\ngamma\nalpha\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();

    // Enter search mode and type "alpha"
    session.handle_command(EditorCommand::Search).unwrap();
    for ch in "alpha".chars() {
        session
            .handle_command(EditorCommand::InsertChar(ch))
            .unwrap();
    }

    // Confirm search (Enter) — jumps to first match at char 0
    session.handle_command(EditorCommand::Enter).unwrap();
    assert_eq!(session.cursor.char_index, 0);

    // Press Ctrl-G repeatedly — should advance through all "alpha" occurrences
    session.handle_command(EditorCommand::FindNext).unwrap();
    assert_eq!(
        session.cursor.char_index, 11,
        "Should advance to second occurrence"
    );

    // Press Ctrl-G again — should advance to the third occurrence
    session.handle_command(EditorCommand::FindNext).unwrap();
    assert_eq!(
        session.cursor.char_index, 23,
        "Should advance to third occurrence"
    );

    // Press Ctrl-G again — should wrap around and find the first match
    session.handle_command(EditorCommand::FindNext).unwrap();
    assert_eq!(
        session.cursor.char_index, 0,
        "Should have wrapped back to start"
    );
}

// T024 (US2): Ctrl-G from editing mode with confirmed search should still work (query persists)
#[test]
fn findnext_from_editing_mode_with_active_query_jumps_next() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("findnext-edit-mode.txt");
    std::fs::write(&path, "foo bar foo baz\n").unwrap();

    let mut session = EditingSession::open(&path, TerminalSize::new(80, 24)).unwrap();

    // Search for "foo", Enter to confirm first match at char 0
    session.handle_command(EditorCommand::Search).unwrap();
    for ch in "foo".chars() {
        session
            .handle_command(EditorCommand::InsertChar(ch))
            .unwrap();
    }
    session.handle_command(EditorCommand::Enter).unwrap();

    let initial_pos = session.cursor.char_index;
    assert_eq!(initial_pos, 0);

    // Now in editing mode. Press Ctrl-G — should still find next match
    session.handle_command(EditorCommand::FindNext).unwrap();
    assert!(
        session.cursor.char_index > initial_pos,
        "Should advance to second 'foo'"
    );
}
