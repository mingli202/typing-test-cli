use super::Mode;

pub enum Action {
    Root(crate::action::Action),
    ModeChange(Mode),
    NewTypingScreen,
    NewEndScreen { final_wpm: f64, accuracy: usize },
}
