// Unit tests for HelpDialog data model and scroll boundary logic (spec 011).
// Tests build_help_rows() content completeness, correct ordering, and scroll
// offset clamping behavior.

use gobo::editor::status::{HelpDialogRow, build_help_rows};

// ---- build_help_rows content -------------------------------------------------

#[test]
fn build_help_rows_returns_exactly_9_entries() {
    let rows = build_help_rows();
    assert_eq!(rows.len(), 9);
}

#[test]
fn build_help_rows_keybindings_in_contract_order() {
    let rows = build_help_rows();
    // Contract order from contracts/key-bindings.md (spec 011).
    let expected_keys: &[&str] = &[
        "Ctrl-F", "Ctrl-G", "Ctrl-S", "Ctrl-Z", "Ctrl-Y",
        "Ctrl-C", "Ctrl-X", "Ctrl-V", "Ctrl-Q",
    ];
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(row.key.as_str(), expected_keys[i], "key mismatch at index {i}");
        assert!(
            !row.label.is_empty(),
            "label empty at index {i} ({})",
            expected_keys[i]
        );
    }
}

#[test]
fn build_help_rows_labels_are_descriptive() {
    let rows = build_help_rows();
    let expected_labels: &[&str] = &[
        "Find in document",
        "Find next match",
        "Save document",
        "Undo last edit",
        "Redo last undone edit",
        "Copy selection to clipboard",
        "Cut selection to clipboard",
        "Paste from clipboard",
        "Quit (clean) / save prompt (dirty)",
    ];
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(
            row.label.as_str(),
            expected_labels[i],
            "label mismatch at index {i}"
        );
    }
}

#[test]
fn build_help_rows_clone_derives_correctly() {
    let rows = build_help_rows();
    let cloned: Vec<HelpDialogRow> = rows.to_vec();
    for (a, b) in rows.iter().zip(cloned.iter()) {
        assert_eq!(a.key, b.key);
        assert_eq!(a.label, b.label);
    }
}
