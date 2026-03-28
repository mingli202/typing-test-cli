use std::time::Instant;

/// Represents a single letter of a word
#[derive(Debug, Clone)]
pub struct Letter {
    letter: char,
    char_id: usize,
    word_id: usize,
}

/// Represent a single word of the text to type
#[derive(Debug, Clone)]
pub struct Word {
    id: usize,
    letters: Vec<Letter>,

    /// Whether the word has been typed wrong
    is_error: bool,

    /// The underlying word. Kept so we can easily render the word
    word: String,

    /// Which letter the user last typed
    last_typed_letter_index: usize,
}

impl Word {
    pub fn new(text: &str, id: usize) -> Word {
        Word {
            letters: text
                .chars()
                .enumerate()
                .map(|(i, letter)| Letter {
                    letter,
                    char_id: i,
                    word_id: id,
                })
                .collect(),
            is_error: false,
            id,
            word: text.to_string(),
            last_typed_letter_index: 0,
        }
    }
}

/// Represents a single typing test
pub struct TypingTest {
    /// All the words of the text to type
    ///
    ///  0        1        2       3     4      5       6        7
    /// [[hello], [world], [this], [is], [the], [best], [thing], [ever]]
    ///   01234    01234    0123    01    012    0123    01234    0123
    words: Vec<Word>,

    /// The current word the user is at
    word_index: usize,

    /// The current letter in the current word
    letter_index: usize,

    /// When the test has started
    time_started: Instant,

    /// Whether the test has started
    started: bool,

    /// How many wrong words
    wrongs: usize,

    /// How many characters typed in total (includes spaces)
    n_letter_typed: i32,
}

impl TypingTest {
    pub fn new(text: &str) -> Self {
        let words: Vec<Word> = text
            .split(" ")
            .enumerate()
            .map(|(id, word)| Word::new(word, id))
            .collect();

        TypingTest {
            word_index: 0,
            letter_index: 0,
            time_started: Instant::now(),
            started: false,
            words,
            wrongs: 0,
            n_letter_typed: 0,
        }
    }

    /// Processes the typed character. Returns whether the test is done.
    pub fn on_type(&self, c: char) -> bool {
        false
    }
}
