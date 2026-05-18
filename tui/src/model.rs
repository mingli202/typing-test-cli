use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;
use crate::args::Args;
pub use crate::msg::Msg;
use crate::multiplayer::{self, MultiplayerModel};
use crate::singleplayer::SinglePlayerModel;
use crate::util::config::{Config, ConfigUpdate};
use crate::util::data_provider::DataProvider;
use crate::util::toast::{self, Toast};
use crate::{CustomEvent, singleplayer};

pub enum Screen {
    SinglePlayer(SinglePlayerModel),
    Multiplayer(MultiplayerModel),
}

pub struct AppModel {
    pub exit: bool,
    toast: Toast,
    config: Config,
    args: Args,
    screen: Screen,
    data_provider: DataProvider,
    event_tx: UnboundedSender<CustomEvent>,
}

impl AppModel {
    pub async fn new(
        event_tx: UnboundedSender<CustomEvent>,
        args: Args,
    ) -> color_eyre::Result<Self> {
        let config = Config::new(event_tx.clone()).await;
        let toast = Toast::new(event_tx.clone());
        let data_provider = DataProvider::new(&args.words_path, &args.quotes_path)?;

        let initial_mode = config.data.mode.clone();
        let data = data_provider.get_data_from_mode(&initial_mode);

        Ok(AppModel {
            exit: false,
            screen: Screen::SinglePlayer(SinglePlayerModel::new(data, initial_mode, args.no_error)),
            toast,
            config,
            data_provider,
            event_tx,
            args,
        })
    }
}

pub fn update(model: &mut AppModel, msg: Msg) -> Option<Action> {
    match msg {
        Msg::ToastAction(action) => model.toast.handle_action(action),
        _ => {
            match msg {
                Msg::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    return Some(Action::Quit);
                }
                Msg::Key(KeyEvent {
                    code: KeyCode::Char('p'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    if let Screen::SinglePlayer(_) = model.screen {
                        return Some(Action::SwitchScreen(Screen::Multiplayer(
                            MultiplayerModel::new(model.event_tx.clone()),
                        )));
                    } else {
                        return Some(Action::SwitchToSinglePlayer);
                    }
                }
                _ => {}
            }

            return match &mut model.screen {
                Screen::SinglePlayer(singleplayer_model) => singleplayer::update(
                    singleplayer_model,
                    &model.data_provider,
                    model.args.no_error,
                    msg,
                ),
                Screen::Multiplayer(multiplayer_model) => {
                    multiplayer::update(multiplayer_model, &model.event_tx, msg)
                }
            };
        }
    };

    None
}

pub fn view(model: &AppModel, frame: &mut Frame) {
    let area = frame.area();
    let buf = frame.buffer_mut();

    match &model.screen {
        Screen::SinglePlayer(singleplayer_model) => {
            singleplayer::view(singleplayer_model, area, buf)
        }
        Screen::Multiplayer(multiplayer_model) => multiplayer::view(multiplayer_model, area, buf),
    };

    toast::view(&model.toast, area, buf);
}

pub fn handle_action(model: &mut AppModel, action: Action) -> Option<Action> {
    match action {
        Action::Quit => model.exit = true,
        Action::SwitchScreen(screen) => model.screen = screen,
        Action::SwitchToSinglePlayer => {
            let initial_mode = model.config.data.mode.clone();
            let data = model.data_provider.get_data_from_mode(&initial_mode);
            let no_error = model.args.no_error;

            return Some(Action::SwitchScreen(Screen::SinglePlayer(
                SinglePlayerModel::new(data, initial_mode, no_error),
            )));
        }
        Action::ConfigModeUpdate(mode) => {
            model.config.data.mode = mode.clone();
            model.config.handle_config_update(ConfigUpdate::Mode(mode));
        }
    };

    None
}
