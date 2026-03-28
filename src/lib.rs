use std::io;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event};
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, Frame};

pub struct TypingTestState {}
pub struct EndScreenState {}

pub enum Transition {
    None,
    Switch(State),
    Push(State),
    Pop,
    Quit,
}

pub enum State {
    TypingTestState(TypingTestState),
    EndScreenState(EndScreenState),
}

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        todo!()
    }
}

impl State {
    fn handle_events(self, event: Event) -> Transition {
        Transition::None
    }
}

struct Config {}

struct App {
    state: State,
    config: Config,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(&self.state, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        Ok(())
    }
}
