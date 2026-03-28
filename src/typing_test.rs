use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Letter {
    pub letter: char,
    pub char_id: usize,
    pub word_id: usize,
}

#[derive(Debug, Clone)]
pub struct Word {
    pub letters: Vec<Letter>,
    pub is_error: bool,
    pub id: usize,
    pub word: String,
    pub last_typed: usize,
}

impl Word {
    pub fn from_str(text: &str, id: usize) -> Word {
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
            last_typed: 0,
        }
    }
}

pub struct TypingTest {
    pub words: Vec<Word>,
    pub word_index: usize,
    pub letter_index: usize,
    pub time_started: Instant,
    pub started: bool,
    pub wrongs: usize,
    pub char_typed: i32,
}

impl TypingTest {}
