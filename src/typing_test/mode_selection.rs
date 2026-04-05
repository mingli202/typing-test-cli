use ratatui::macros::{line, span, text};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::state::Mode;

#[derive(PartialEq, Clone)]
pub enum WordsOption {
    Ten,
    Twentyfive,
    Fifty,
    Hundred,
}

pub enum WordSelectionOption {
    /// When selected on "Word" and not any of the word options
    /// The placeholder keep the last selected value so when pressing down after pressing up, the
    /// previous selected will be chosen. Better UX.
    Placeholder(WordsOption),
    Selected(WordsOption),
}

impl WordsOption {
    pub fn to_num(&self) -> usize {
        match self {
            Self::Ten => 10,
            Self::Twentyfive => 25,
            Self::Fifty => 50,
            Self::Hundred => 100,
        }
    }

    pub fn to_mode(&self) -> Mode {
        Mode::Words(self.to_num())
    }

    pub fn next(self) -> Self {
        match self {
            Self::Ten => Self::Twentyfive,
            Self::Twentyfive => Self::Fifty,
            Self::Fifty => Self::Hundred,
            Self::Hundred => Self::Ten,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Ten => Self::Hundred,
            Self::Twentyfive => Self::Ten,
            Self::Fifty => Self::Twentyfive,
            Self::Hundred => Self::Fifty,
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum TimeOption {
    Fifteen,
    Thirty,
    Sixty,
    HundredAndTwenty,
}

pub enum TimeSelectionOption {
    Placeholder(TimeOption),
    Selected(TimeOption),
}

pub enum ModeOption {
    Quote,
    Words(WordSelectionOption),
    Time(TimeSelectionOption),
}

impl TimeOption {
    pub fn to_num(&self) -> usize {
        match self {
            Self::Fifteen => 15,
            Self::Thirty => 30,
            Self::Sixty => 60,
            Self::HundredAndTwenty => 120,
        }
    }

    pub fn to_mode(&self) -> Mode {
        Mode::Time(self.to_num())
    }

    pub fn next(self) -> Self {
        match self {
            Self::Fifteen => Self::Thirty,
            Self::Thirty => Self::Sixty,
            Self::Sixty => Self::HundredAndTwenty,
            Self::HundredAndTwenty => Self::Fifteen,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Fifteen => Self::HundredAndTwenty,
            Self::Thirty => Self::Fifteen,
            Self::Sixty => Self::Thirty,
            Self::HundredAndTwenty => Self::Sixty,
        }
    }
}

impl ModeOption {
    pub fn from_mode(mode: Mode) -> Self {
        match mode {
            Mode::Quote => ModeOption::Quote,
            Mode::Words(10) => ModeOption::Words(WordSelectionOption::Selected(WordsOption::Ten)),
            Mode::Words(25) => {
                ModeOption::Words(WordSelectionOption::Selected(WordsOption::Twentyfive))
            }
            Mode::Words(50) => ModeOption::Words(WordSelectionOption::Selected(WordsOption::Fifty)),
            Mode::Words(100) => {
                ModeOption::Words(WordSelectionOption::Selected(WordsOption::Hundred))
            }
            Mode::Time(15) => ModeOption::Time(TimeSelectionOption::Selected(TimeOption::Fifteen)),
            Mode::Time(30) => ModeOption::Time(TimeSelectionOption::Selected(TimeOption::Thirty)),
            Mode::Time(60) => ModeOption::Time(TimeSelectionOption::Selected(TimeOption::Sixty)),
            Mode::Time(120) => {
                ModeOption::Time(TimeSelectionOption::Selected(TimeOption::HundredAndTwenty))
            }
            _ => panic!("Impossible mode"),
        }
    }

    pub fn to_mode(&self) -> Option<Mode> {
        match self {
            Self::Quote => Some(Mode::Quote),
            Self::Words(w) => match w {
                WordSelectionOption::Placeholder(_) => None,
                WordSelectionOption::Selected(w) => Some(w.to_mode()),
            },
            Self::Time(t) => match t {
                TimeSelectionOption::Placeholder(_) => None,
                TimeSelectionOption::Selected(t) => Some(t.to_mode()),
            },
        }
    }
}

pub struct ModeSelection {
    selected_mode: ModeOption,
}

impl ModeSelection {
    pub fn new(initial_mode: Mode) -> Self {
        ModeSelection {
            selected_mode: ModeOption::from_mode(initial_mode),
        }
    }

    pub fn to_mode(&self) -> Option<Mode> {
        self.selected_mode.to_mode()
    }

    pub fn handle_left(&mut self) {
        self.selected_mode = match &self.selected_mode {
            ModeOption::Quote => {
                ModeOption::Time(TimeSelectionOption::Placeholder(TimeOption::Fifteen))
            }
            ModeOption::Words(WordSelectionOption::Placeholder(_)) => ModeOption::Quote,
            ModeOption::Words(WordSelectionOption::Selected(w)) => {
                ModeOption::Words(WordSelectionOption::Selected(w.clone().prev()))
            }
            ModeOption::Time(TimeSelectionOption::Placeholder(_)) => {
                ModeOption::Words(WordSelectionOption::Placeholder(WordsOption::Ten))
            }
            ModeOption::Time(TimeSelectionOption::Selected(t)) => {
                ModeOption::Time(TimeSelectionOption::Selected(t.clone().prev()))
            }
        }
    }

    pub fn handle_right(&mut self) {
        self.selected_mode = match &self.selected_mode {
            ModeOption::Quote => {
                ModeOption::Words(WordSelectionOption::Placeholder(WordsOption::Ten))
            }
            ModeOption::Words(WordSelectionOption::Placeholder(_)) => {
                ModeOption::Time(TimeSelectionOption::Placeholder(TimeOption::Fifteen))
            }
            ModeOption::Words(WordSelectionOption::Selected(w)) => {
                ModeOption::Words(WordSelectionOption::Selected(w.clone().next()))
            }
            ModeOption::Time(TimeSelectionOption::Placeholder(_)) => ModeOption::Quote,
            ModeOption::Time(TimeSelectionOption::Selected(t)) => {
                ModeOption::Time(TimeSelectionOption::Selected(t.clone().next()))
            }
        }
    }

    pub fn handle_up(&mut self) {
        if let ModeOption::Words(WordSelectionOption::Selected(w)) = &self.selected_mode {
            self.selected_mode = ModeOption::Words(WordSelectionOption::Placeholder(w.clone()));
        } else if let ModeOption::Time(TimeSelectionOption::Selected(t)) = &self.selected_mode {
            self.selected_mode = ModeOption::Time(TimeSelectionOption::Placeholder(t.clone()));
        }
    }

    pub fn handle_down(&mut self) {
        if let ModeOption::Words(WordSelectionOption::Placeholder(w)) = &self.selected_mode {
            self.selected_mode = ModeOption::Words(WordSelectionOption::Selected(w.clone()));
        } else if let ModeOption::Time(TimeSelectionOption::Placeholder(t)) = &self.selected_mode {
            self.selected_mode = ModeOption::Time(TimeSelectionOption::Selected(t.clone()));
        }
    }
}

impl Widget for &ModeSelection {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut quote_text = span!("Quote");
        let mut word_text = span!("Words");
        let mut time_text = span!("Time");

        let selection = match &self.selected_mode {
            ModeOption::Quote => {
                quote_text = highlight(quote_text);
                text![line![
                    quote_text,
                    span!(" "),
                    word_text,
                    span!(" "),
                    time_text
                ]]
            }
            ModeOption::Words(selected_word) => {
                let mut choices = [
                    WordsOption::Ten,
                    WordsOption::Twentyfive,
                    WordsOption::Fifty,
                    WordsOption::Hundred,
                ]
                .iter()
                .map(|w| span!(w.to_num()))
                .collect::<Vec<Span>>();

                if let WordSelectionOption::Selected(word) = selected_word
                    && let Some(chosen) = choices
                        .iter_mut()
                        .find(|choice| *choice.content == word.to_num().to_string())
                {
                    *chosen = highlight(chosen.clone());
                    word_text = word_text.fg(Color::Black).bg(Color::DarkGray);
                } else {
                    word_text = highlight(word_text);
                }

                let choices: Vec<Span> =
                    itertools::Itertools::intersperse(choices.into_iter(), span!(" ")).collect();

                text![
                    line![quote_text, span!(" "), word_text, span!(" "), time_text],
                    span!(" "),
                    Line::from(choices)
                ]
            }
            ModeOption::Time(selected_time) => {
                let mut choices = [
                    TimeOption::Fifteen,
                    TimeOption::Thirty,
                    TimeOption::Sixty,
                    TimeOption::HundredAndTwenty,
                ]
                .iter()
                .map(|w| span!(w.to_num()))
                .collect::<Vec<Span>>();

                if let TimeSelectionOption::Selected(time) = selected_time
                    && let Some(chosen) = choices
                        .iter_mut()
                        .find(|choice| *choice.content == time.to_num().to_string())
                {
                    *chosen = highlight(chosen.clone());
                    time_text = time_text.fg(Color::Black).bg(Color::DarkGray);
                } else {
                    time_text = highlight(time_text);
                }

                let choices: Vec<Span> =
                    itertools::Itertools::intersperse(choices.into_iter(), span!(" ")).collect();

                text![
                    line![quote_text, span!(" "), word_text, span!(" "), time_text],
                    span!(" "),
                    Line::from(choices)
                ]
            }
        };

        selection.centered().render(area, buf);
    }
}

fn highlight(text: Span) -> Span {
    text.fg(Color::Black).bg(Color::White)
}
