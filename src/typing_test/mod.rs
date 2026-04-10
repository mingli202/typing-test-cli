use std::time::{Duration, Instant};

use crossterm::event::KeyCode;

use crate::action::Action;
use crate::model::{Mode, SharedModel};

use self::mode_selection::ModeSelection;
use self::typing::TypingTest;

mod letter;
mod mode_selection;
mod typing;
mod word;

pub enum Msg {
    Key(KeyCode),
    Tick,
}

#[derive(Debug, Default)]
pub struct TypingStats {
    wpm: f64,
    current_index: usize,
    elapsed: Duration,
}

pub struct TypingModel {
    typing_test: TypingTest,
    stats_last_updated_time: Instant,
    stats: TypingStats,
    selected_mode: ModeSelection,
}

impl TypingModel {
    pub fn new(text: &str, initial_mode: Mode) -> Self {
        TypingModel {
            typing_test: TypingTest::new(text),
            stats_last_updated_time: Instant::now(),
            stats: TypingStats::default(),
            selected_mode: ModeSelection::new(initial_mode),
        }
    }
}

pub fn update(
    typing_model: &mut TypingModel,
    shared_model: &mut SharedModel,
    msg: Msg,
) -> Option<Action> {
    let TypingModel {
        typing_test,
        stats_last_updated_time,
        stats,
        selected_mode,
    } = typing_model;

    let mut actions = vec![];

    match msg {
        Msg::Key(key) => match key {
            KeyCode::Char(c) => {
                typing_test.start();

                let has_ended = typing_test.on_type(c);
                if has_ended {
                    let wpm = typing_test.net_wpm();
                    let accuracy = typing_test.accuracy();

                    if let Some(elapsed) = typing_test.elapsed_since_start_sec() {
                        shared_model.history.push((elapsed.as_secs_f64(), wpm));
                    }
                }
            }
            KeyCode::Backspace => {
                typing_test.on_backspace();
            }
            KeyCode::Tab => {}
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                if let Some(action) = handle_arrow_keys(selected_mode, &mut shared_model.mode, key)
                {
                    actions.push(action);
                }
            }
            _ => {}
        },
        Msg::Tick => {
            if stats_last_updated_time.elapsed() > Duration::from_secs(1) {
                *stats_last_updated_time = Instant::now();
            }
        }
    };

    None
}

fn handle_arrow_keys(
    selected_mode: &mut ModeSelection,
    current_mode: &mut Mode,
    key: KeyCode,
) -> Option<Action> {
    match key {
        KeyCode::Left => {
            selected_mode.handle_left();
        }
        KeyCode::Right => {
            selected_mode.handle_right();
        }
        KeyCode::Up => {
            selected_mode.handle_up();
        }
        KeyCode::Down => {
            selected_mode.handle_down();
        }
        _ => {}
    }
    let selected_mode = selected_mode.selected_mode();

    if let Some(selected_mode) = selected_mode
        && selected_mode != *current_mode
    {
        *current_mode = selected_mode.clone();
        return Some(Action::UpdateConfigMode(selected_mode.clone()));
    }

    None
}

pub fn view(
    typing_model: &TypingModel,
    area: ratatui::prelude::Rect,
    buf: &mut ratatui::prelude::Buffer,
) -> color_eyre::Result<()> {
    typing::view_typing_test(&typing_model.typing_test, area, buf);

    Ok(())
}
