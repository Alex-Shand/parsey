/// The position of a character in a file
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterPosition {
    /// The zero indexed line number
    pub row: usize,
    /// The zero indexed column number
    pub col: usize,
}

/// Source span of a token
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Span {
    /// The location of the first character of the token
    pub start: CharacterPosition,
    /// The location of the first character after the token
    pub end: CharacterPosition,
}

impl Span {
    pub(super) fn new(
        start_line: usize,
        end_line: usize,
        start_char: usize,
        end_char: usize,
    ) -> Self {
        Span {
            start: CharacterPosition {
                row: start_line,
                col: start_char
            },
            end: CharacterPosition {
                row: end_line,
                col: end_char
            }
        }
    }
}
