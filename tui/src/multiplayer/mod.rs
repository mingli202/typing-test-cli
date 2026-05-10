use std::sync::{Arc, RwLock};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_util::sync::CancellationToken;

use crate::CustomEvent;
use crate::msg::Msg;

use self::helpers::connect_to_ws;
use self::models::{LobbyInfo, PlayersInfoSnapshot, WsMsg};

mod helpers;
mod models;

pub enum GameStatus {
    Waiting,
    Countdown(i8),
    Playing,
    End,
}

#[derive(Default)]
pub struct SharedModel {
    user_id: Option<String>,
    active_lobby_id: Option<String>,
    pending_join_lobby_id: Option<String>,
    players_info: Option<PlayersInfoSnapshot>,
    lobby_info: Option<LobbyInfo>,
    game_status: Option<GameStatus>,
}

pub struct MultiplayerModel {
    shared_model: Arc<RwLock<SharedModel>>,
    write_tx: UnboundedSender<String>,

    cancel_token: CancellationToken,
}

impl MultiplayerModel {
    pub fn new(event_tx: UnboundedSender<CustomEvent>) -> Self {
        let (write_tx, write_rx) = mpsc::unbounded_channel::<String>();

        let model = MultiplayerModel {
            shared_model: Arc::new(RwLock::new(SharedModel::default())),
            write_tx,
            cancel_token: CancellationToken::new(),
        };

        tokio::spawn(connect_to_ws(
            Arc::clone(&model.shared_model),
            model.cancel_token.clone(),
            event_tx,
            write_rx,
        ));

        model
    }

    /// Sends the given message to the websocket
    pub fn send_msg(&self, msg: WsMsg) {
        let pending_join_lobby_id = match &msg {
            WsMsg::JoinGroup(group_id) => Some(group_id.clone()),
            _ => None,
        };
        let did_send = self.write_tx.send(msg.to_string()).is_ok();

        if did_send && let Some(group_id) = pending_join_lobby_id {
            let mut lock = self.shared_model.write().unwrap();
            lock.pending_join_lobby_id = Some(group_id);
        }
    }
}

impl Drop for MultiplayerModel {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

pub fn update(model: &mut MultiplayerModel, msg: Msg) -> Option<crate::action::Action> {
    match msg {
        Msg::Key(key) => {}
        Msg::Tick => {}
        _ => {}
    };

    None
}

pub fn view(model: &MultiplayerModel, area: Rect, buf: &mut Buffer) {}
