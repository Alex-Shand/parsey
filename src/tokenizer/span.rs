/// Source span of a token
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Span {
    /// The line the token starts on
    pub start_line: usize,
    /// The line the token ends on
    pub end_line: usize,
    /// The character position on `start_line` the token starts at
    pub start_char: usize,
    /// The character position on `end_line` the token ends at
    pub end_char: usize,
}

impl Span {
    pub(super) fn new(
        start_line: usize,
        end_line: usize,
        start_char: usize,
        end_char: usize,
    ) -> Self {
        Span {
            start_line,
            end_line,
            start_char,
            end_char,
        }
    }
}
