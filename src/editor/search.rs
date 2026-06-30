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

            // Collect ALL match byte positions in order first.
          let mut all_matches: Vec<usize> = Vec::new();
          let mut search_pos = 0usize;
          while search_pos < haystack_cmp.len() {
              let needle_len = needle_cmp.len();
               if search_pos + needle_len > haystack_cmp.len() {
                   break;
                   }

               if let Some(pos) = haystack_cmp[search_pos..].find(&needle_cmp) {
                   let abs_pos = search_pos + pos;
                    if abs_pos + needle_len > haystack_cmp.len() {
                        break;
                       }
                   all_matches.push(abs_pos);
                   search_pos = (abs_pos + needle_len).min(haystack_cmp.len());
                   } else {
                   break;
                   }
               }

           if all_matches.is_empty() {
               self.last_match_char_range = None;  
               self.last_result = SearchResultState::NoMatch;
               return None; 
             }

            // Pick the match based on state from previous call.
            // Compare in byte space for consistency: use the larger of
            // (a) the byte position right after the last match's end, and
            // (b) the byte position corresponding to the requested start char index.
            // This advances past the last match for repeated Ctrl-G, but wraps
            // to the first match when the caller requests a position past the end.
          let last_end_byte = self.last_match_char_range
              .map(|(_, end_char)| char_to_byte_index(&haystack, end_char.min(text.len_chars())))
              .unwrap_or(0);
          let start_byte = char_to_byte_index(&haystack, start_char_index.min(text.len_chars()));
          let base = last_end_byte.max(start_byte);

              // Find first match starting at or after 'base'.
              // If past all matches, wrap to first one.
          let next_idx = all_matches.iter().position(|&m| m >= base)
                                      .unwrap_or(0);

           let byte_pos = *all_matches.get(next_idx).unwrap();
           let byte_end = byte_pos + needle_cmp.len();

            let cs = haystack[..byte_pos].chars().count();
           let ce = haystack[..byte_end].chars().count();
            self.last_match_char_range = Some((cs, ce));
           self.last_result = SearchResultState::MatchFound;
           return Some((cs, ce)); 
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
