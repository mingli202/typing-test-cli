use crossterm::event::{KeyCode, KeyEvent};

pub enum Msg {
    Tick,
    Key(KeyCode),
}
