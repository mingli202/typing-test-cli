use crate::message::Message;
use crate::model::Mode;

pub enum Action {
    Quit,
    UpdateConfigMode(Mode),
    Message(Message),
}
