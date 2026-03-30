use std::io;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event};
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use ratatui::{DefaultTerminal, Frame};

use self::data::Data;
use self::typing_test::TypingTest;

mod data;
mod typing_test;

pub struct TypingTestState {
    typing_test: TypingTest,
}

impl TypingTestState {
    pub fn new(typing_test: TypingTest) -> Self {
        TypingTestState { typing_test }
    }
}

impl Widget for &TypingTestState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
    }
}

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
    fn handle_events(&mut self, event: Event) -> Transition {
        Transition::None
    }
}

struct App {
    state: State,
    history: Vec<State>,
    exit: bool,
    data: Data,
}

impl App {
    pub fn new(data: Data) -> Self {
        let initial_text = data.get_random_quote().quote.clone();
        App {
            state: State::TypingTestState(TypingTestState::new(TypingTest::new(&initial_text))),
            history: vec![],
            exit: false,
            data,
        }
    }

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
        if let Ok(event) = event::read() {
            let transition = self.state.handle_events(event);
            self.handle_transition(transition);
        }
        Ok(())
    }

    fn handle_transition(&mut self, transition: Transition) {
        match transition {
            Transition::Switch(next_state) => self.state = next_state,
            Transition::Quit => self.exit = true,
            Transition::Push(state) => self.history.push(state),
            Transition::Pop => {
                self.history.pop();
            }
            Transition::None => (),
        }
    }
}
