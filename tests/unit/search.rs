use gobo::editor::search::{SearchResultState, SearchState};
use ropey::Rope;

#[test]
fn search_is_case_insensitive_by_default() {
    let text = Rope::from_str("First\nTHIRD line\n");
    let mut search = SearchState {
        query: "third".into(),
        ..SearchState::default()
    };

    let result = search.find_next(&text, 0);

    assert_eq!(result, Some((6, 11)));
    assert_eq!(search.last_result, SearchResultState::MatchFound);
}

#[test]
fn search_reports_no_match() {
    let text = Rope::from_str("alpha\nbeta\n");
    let mut search = SearchState {
        query: "missing".into(),
        ..SearchState::default()
    };

    assert_eq!(search.find_next(&text, 0), None);
    assert_eq!(search.last_result, SearchResultState::NoMatch);
}
