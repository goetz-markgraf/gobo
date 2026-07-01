use ropey::Rope;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseMode {
    Insensitive,
    Sensitive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchResultState {
    Idle,
    MatchFound,
    NoMatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchState {
    pub query: String,
    pub case_mode: CaseMode,
    pub last_match_char_range: Option<(usize, usize)>,
    pub last_result: SearchResultState,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            case_mode: CaseMode::Insensitive,
            last_match_char_range: None,
            last_result: SearchResultState::Idle,
        }
    }
}

/// Lowercase a single character.
fn lc(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

impl SearchState {
    /// Find the next match for `self.query` in `text`.
    /// Searches entirely in character space to avoid byte-index panics when
    /// lowercased strings differ in length from the original (e.g. İ → i + combining accent).
    pub fn find_next(&mut self, text: &Rope, start_char_index: usize) -> Option<(usize, usize)> {
        // Guard: empty query means "idle" state.
        if self.query.is_empty() {
            self.last_match_char_range = None;
            self.last_result = SearchResultState::Idle;
            return None;
        }

        // Convert rope to string — never slice with offsets from the lowered version.
        let haystack_str = text.to_string();
        let nchars: usize = haystack_str.chars().count();
        let nlen: usize = self.query.chars().count();

        // No match possible if needle longer than document.
        if nlen == 0 || nchars < nlen {
            self.last_match_char_range = None;
            self.last_result = SearchResultState::NoMatch;
            return None;
        }

        // Build lowered lookups — char arrays for O(1) access, safe to index.
        let haystack_lc: Vec<char> = haystack_str.chars().map(lc).collect();
        let needle_lc: Vec<char> = self.query.chars().map(lc).collect();

        // Character-by-character search on LOWERED chars. No slicing — panic impossible!
        let mut matches: Vec<(usize, usize)> = Vec::new();
        for i in 0..=(haystack_lc.len() - nlen) {
            if (0..nlen).all(|j| haystack_lc[i + j] == needle_lc[j]) {
                matches.push((i, i + nlen));
            }
        }

        // No match found.
        if matches.is_empty() {
            self.last_match_char_range = None;
            self.last_result = SearchResultState::NoMatch;
            return None;
        }

        // Select which match: advance past last end or wrap to first.
        let base: usize = self.last_match_char_range
            .map(|(_, e)| e.min(nchars))
            .unwrap_or(0)
            .max(start_char_index.min(nchars));

        // First match at or after 'base'; wrap to first if none.
        let next_idx = matches.iter()
            .position(|&(s, _)| s >= base)
            .unwrap_or(0);

        let (match_start, match_end) = matches[next_idx];
        self.last_match_char_range = Some((match_start, match_end));
        self.last_result = SearchResultState::MatchFound;
        Some((match_start, match_end))
    }
}
