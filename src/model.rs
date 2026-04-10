use ratatui::Frame;
use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::data::Data;
pub use crate::msg::Msg;
use crate::toast::ToastMessage;
use crate::typing_test::TypingModel;

#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Mode {
    #[default]
    Quote,
    Words(usize),
    Time(usize),
}

impl Mode {
    pub fn get_data(&self) -> Data {
        match self {
            Mode::Quote => Data::get_random_quote(),
            Mode::Words(n) => Data::get_n_random_words(*n),
            // TODO: new lines as the user reaches the end
            // max 80 char per line -> ~16 words
            // preload 4 lines
            //
            // NOTE: require refactor of current architecture or it will become messy
            // for now, just assume the user won't type more than 240 wpm
            Mode::Time(t) => {
                let mut data = Data::get_n_random_words(t * 4);
                data.source = format!("{} seconds", t);
                data
            }
        }
    }
}

struct Toast {
    pub messages: Vec<ToastMessage>,
}

struct Config {}

pub enum Screen {
    Typing(TypingModel),
    End,
}

pub struct SharedModel {
    pub mode: Mode,
    // (time, wpm)
    pub history: Vec<(f64, f64)>,
    pub data: Data,
}

pub struct AppModel {
    pub exit: bool,
    // toast: Toast,
    // config: Config,
    screen: Screen,
    shared_model: SharedModel,
}

impl AppModel {
    pub fn init(initial_mode: Mode) -> Self {
        let data = initial_mode.get_data();
        let text = &data.text;
        AppModel {
            exit: false,
            screen: Screen::Typing(TypingModel::new(text, initial_mode.clone())),
            shared_model: SharedModel {
                mode: initial_mode,
                history: vec![],
                data,
            },
        }
    }
}

pub fn update(model: &mut AppModel, msg: Msg) -> Option<Action> {
    None
}

pub fn view(model: &AppModel, frame: &mut Frame) {}
