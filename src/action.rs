use crate::model::{Mode, Screen};
use crate::msg::Msg;

pub enum Action {
    Quit,
    UpdateConfigMode(Mode),
    SwitchScreen(Screen),
}
