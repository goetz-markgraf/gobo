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

impl SearchState {
    pub fn find_next(&mut self, text: &Rope, start_char_index: usize) -> Option<(usize, usize)> {
        if self.query.is_empty() {
            self.last_match_char_range = None;
            self.last_result = SearchResultState::Idle;
            return None;
        }

        let haystack = text.to_string();
        let haystack_cmp = normalize(&haystack, &self.case_mode);
        let needle_cmp = normalize(&self.query, &self.case_mode);
        let start_byte = char_to_byte_index(&haystack, start_char_index.min(text.len_chars()));

        let search_attempt = haystack_cmp[start_byte..]
            .find(&needle_cmp)
            .map(|offset| start_byte + offset)
            .or_else(|| haystack_cmp[..start_byte].find(&needle_cmp));

        if let Some(byte_start) = search_attempt {
            let byte_end = byte_start + needle_cmp.len();
            let char_start = haystack[..byte_start].chars().count();
            let char_end = haystack[..byte_end].chars().count();
            let range = (char_start, char_end);
            self.last_match_char_range = Some(range);
            self.last_result = SearchResultState::MatchFound;
            Some(range)
        } else {
            self.last_match_char_range = None;
            self.last_result = SearchResultState::NoMatch;
            None
        }
    }
}

fn normalize(input: &str, case_mode: &CaseMode) -> String {
    match case_mode {
        CaseMode::Insensitive => input.to_lowercase(),
        CaseMode::Sensitive => input.to_string(),
    }
}

fn char_to_byte_index(input: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }

    input
        .char_indices()
        .nth(char_index)
        .map(|(idx, _)| idx)
        .unwrap_or_else(|| input.len())
}
