use crossterm::event::KeyEvent;

use crate::util::toast::ToastAction;

pub enum Msg {
    Tick,
    Key(KeyEvent),
    FocusGained,
    FocusLost,
}
