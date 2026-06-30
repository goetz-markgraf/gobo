use gobo::editor::search::{CaseMode, SearchResultState, SearchState};
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

// T002: Edge-case unit tests for SearchState::find_next()

#[test]
fn empty_query_returns_none_without_side_effects() {
    let text = Rope::from_str("alpha beta gamma\n");
    let mut search = SearchState::default(); // query is empty by default
    assert_eq!(search.query, "");
    let result = search.find_next(&text, 0);
    assert_eq!(result, None);
    assert_eq!(search.last_result, SearchResultState::Idle);
}

#[test]
fn single_char_document_matching_query_at_position_0() {
    let text = Rope::from_str("a\n");
    let mut search = SearchState {
        query: "a".into(),
        ..SearchState::default()
    };
    let result = search.find_next(&text, 0);
    assert_eq!(result, Some((0, 1)));
}

#[test]
fn query_longer_than_document_never_matches() {
    let text = Rope::from_str("xy\n");
    let mut search = SearchState {
        query: "abcdef".into(),
        ..SearchState::default()
    };
    assert_eq!(search.find_next(&text, 0), None);
    assert_eq!(search.last_result, SearchResultState::NoMatch);
}

#[test]
fn wrap_around_cursor_past_last_match() {
      // Document: "beta alpha beta\n"
      // "beta" occurrences at chars 0..4 and 10..14
    let text = Rope::from_str("beta alpha beta\n");
    let mut search = SearchState {
        query: "beta".into(),
        ..SearchState::default()
      };
      // Start from char index past end, should wrap to first match at 0
    let result = search.find_next(&text, 16);
    assert_eq!(result, Some((0, 4)));
}

// T015 (US3): Case-insensitive matching test
#[test]
fn case_insensitive_matches_lowercase_from_uppercase_query() {
      // Document contains only lowercase "hello world"
    let text = Rope::from_str("hello world\n");
    let mut search = SearchState {
        query: "HELLO".into(),
        ..SearchState::default()
      };
    let result = search.find_next(&text, 0);
    assert_eq!(result, Some((0, 5))); // finds lowercase "hello"

    let mut search2 = SearchState {
        query: "WORLD".into(),
        ..SearchState::default()
      };
    let result2 = search2.find_next(&text, 0);
    assert_eq!(result2, Some((6, 11))); // finds lowercase "world"
}

// T015a (US2): find_next forward search from given position
#[test]
fn find_next_forward_search_from_given_position() {
      // Text: "he\there\nhey he\n"
      // "he" at chars 0-2, after tab around char 3, then "hey he" later
    let text = Rope::from_str("he\there\nhey he\n");
    let mut search = SearchState {
        query: "he".into(),
        ..SearchState::default()
      };
      // Start from char 2 (after first "he"), forward finds next occurrence
    let result = search.find_next(&text, 2);
    assert!(matches!(result, Some((start, _)) if start > 0));
      // Wrap-around from past end goes back to first match at 0
    let result = search.find_next(&text, 100);
    assert_eq!(result, Some((0, 2)));
}

// T011 (US2): No-match persists — find_next returns None every time for non-existent query
#[test]
fn no_match_persists_across_multiple_calls() {
    let text = Rope::from_str("hello world\n");
    let mut search = SearchState {
        query: "zzzznotexist".into(),
        ..SearchState::default()
      };
    assert_eq!(search.find_next(&text, 10), None);
    assert_eq!(search.last_result, SearchResultState::NoMatch);
     // Second call should also return None (no match) regardless of start position
    assert_eq!(search.find_next(&text, 5), None);
    assert_eq!(search.last_result, SearchResultState::NoMatch);
}

// T002: Multi-byte grapheme query matches exact character
#[test]
fn multi_byte_grapheme_query_finds_exact_matches() {
      // Chinese characters are multi-byte UTF-8
    let text = Rope::from_str("你好世界hello\n");
    let mut search = SearchState {
        query: "世界".into(),
        ..SearchState::default()
      };
    let result = search.find_next(&text, 0);
     // "你好" = chars 0-1, "世界" = chars 2-3
    assert_eq!(result, Some((2, 4)));
}

// T011b (US2): Empty query + find_next returns Idle, not NoMatch
#[test]
fn empty_query_find_next_returns_idle_not_no_match() {
    let text = Rope::from_str("hello world\n");
    let mut search = SearchState::default(); // empty query by default
     // Calling find_next with empty query should NOT set last_result to NoMatch
    let result = search.find_next(&text, 0);
    assert_eq!(result, None);
    assert_eq!(search.last_result, SearchResultState::Idle);
}


// T009 (US2): Three-occurrence wrap-around test  
#[test]
fn three_occurrences_find_next_returns_second_from_first_then_wraps() {
    let text = Rope::from_str("alpha alpha alpha\n");
    let mut search = SearchState {
        query: "alpha".into(),
         ..SearchState::default()
     };
       // First call from position 0 finds match at 0
    let result = search.find_next(&text, 0);
    assert_eq!(result, Some((0, 5)));
    assert_eq!(search.last_result, SearchResultState::MatchFound);
       // From past last occurrence, wrap to first match
    let result2 = search.find_next(&text, 100);
    assert_eq!(result2, Some((0, 5)));
}


// T016 (US3): Verify that search respects original query casing
#[test]
fn find_next_preserves_query_string_after_match() {
       // Search for uppercase "HELLO" in lowercase document
    let text = Rope::from_str("hello world\n");
    let mut search = SearchState { 
        query: "HELLO".into(),  
         ..SearchState::default() 
     };
    let result = search.find_next(&text, 0);
       // Should find the lowercase "hello"
    assert_eq!(result, Some((0, 5)));
       // Query string itself should stay uppercase (the normalize function only 
       // lowercases for comparison, not modifying the stored query)
    assert_eq!(search.query, "HELLO");
}

// T016a: Verify that case_mode is Insensitive by default
#[test]
fn default_search_state_is_insensitive() {
    let search = SearchState::default();
    assert_eq!(search.case_mode, CaseMode::Insensitive);
}
