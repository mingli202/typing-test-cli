use std::time::Duration;

use ratatui::crossterm::event::{self, KeyCode};
use ratatui::{DefaultTerminal, Frame};

use self::config::Config;
use self::state::{Action, State};

mod config;
pub mod data;
mod state;
mod typing_test;

pub struct App {
    state: State,
    exit: bool,
}

impl App {
    pub async fn new() -> Self {
        let config = Config::load().await;

        App {
            state: State::new(config.mode),
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(&self.state, frame.area());
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        if event::poll(Duration::from_millis(250))?
            && let Ok(event) = event::read()
        {
            if let Some(event::KeyEvent {
                code: KeyCode::Esc, ..
            }) = event.as_key_press_event()
            {
                self.exit = true
            }

            let transition = self.state.handle_events(event);
            self.handle_transition(transition);
        }

        let transition = self.state.on_tick();
        self.handle_transition(transition);

        Ok(())
    }

    fn handle_transition(&mut self, transition: Action) {
        match transition {
            // Action::Switch(next_state) => self.state = next_state,
            Action::Quit => self.exit = true,
            Action::None => (),
        }
    }
}
