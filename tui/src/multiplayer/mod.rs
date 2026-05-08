use std::sync::{Arc, RwLock};

use futures::{SinkExt, Stream, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use tokio_util::sync::CancellationToken;

use crate::CustomEvent;
use crate::util::toast::{self, ToastMessage};

use self::helpers::connect_to_ws;
use self::models::{LobbyInfo, NewGame, PlayerInfoSnapshot, WsMsg};

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
    player_info: Option<PlayerInfoSnapshot>,
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

    // Sends the given message to the websocket
    pub fn send_msg(&self, msg: WsMsg) {
        let _ = self.write_tx.send(msg.to_string());
    }
}

impl Drop for MultiplayerModel {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}
