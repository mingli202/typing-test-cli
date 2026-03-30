use std::fmt::Display;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum TypedState {
    Typed(char),
    NotTyped,
    Extra,
}

/// Represents a single letter of a word
#[derive(Debug)]
pub struct Letter {
    /// Its letter
    letter: char,

    /// states for the letter.
    /// used to style this letter white (typed), red (error), gray (not typed)
    pub(super) typed_state: TypedState,

    /// Used to position the cursor correctly in the UI
    char_id: usize,
    word_id: usize,
}

impl Letter {
    /// Creates a new Letter with the given letter, char_id, and word_id
    pub fn new(letter: char, char_id: usize, word_id: usize) -> Self {
        Letter {
            letter,
            typed_state: TypedState::NotTyped,
            char_id,
            word_id,
        }
    }

    /// factory with typed letter
    pub fn with_typed_letter(self, typed_letter: TypedState) -> Self {
        Letter {
            typed_state: typed_letter,
            ..self
        }
    }

    /// Whether this letter is right!
    pub fn is_error(&self) -> bool {
        match self.typed_state {
            TypedState::Typed(c) => c != self.letter,
            _ => true,
        }
    }
}

impl Display for Letter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.letter)
    }
}
