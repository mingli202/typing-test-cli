use crossterm::event::KeyEvent;

pub enum Msg {
    Tick,
    Key(KeyEvent),
}
