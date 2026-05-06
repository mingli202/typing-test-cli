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

use self::models::{LobbyInfo, NewGame, PlayerInfoSnapshot};

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
    pub async fn new(event_tx: UnboundedSender<CustomEvent>) -> Self {
        let (write_tx, write_rx) = mpsc::unbounded_channel::<String>();

        let model = MultiplayerModel {
            shared_model: Arc::new(RwLock::new(SharedModel::default())),
            write_tx,
            cancel_token: CancellationToken::new(),
        };

        connect_to_ws(&model, event_tx, write_rx).await;

        model
    }

    // Sends the given message to the websocket
    pub fn send_msg(&self, msg: String) {
        let _ = self.write_tx.send(msg);
    }
}

impl Drop for MultiplayerModel {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

// Connects to the ws
pub async fn connect_to_ws(
    model: &MultiplayerModel,
    event_tx: UnboundedSender<CustomEvent>,
    write_rx: UnboundedReceiver<String>,
) {
    let request = "ws://localhost:8080/ws".into_client_request().unwrap();

    let (stream, _) = connect_async(request).await.unwrap();
    let (write, read) = stream.split();

    let (read_tx, read_rx) = mpsc::unbounded_channel::<String>();

    let shared_model = Arc::clone(&model.shared_model);

    init_write_task(write, write_rx, model.cancel_token.clone());
    init_read_task(read, read_tx, model.cancel_token.clone());
    init_recv_msg_task(shared_model, read_rx, event_tx, model.cancel_token.clone());
}

// inits the task that will listen for messages to be sent to the websocket
fn init_write_task<T: SinkExt<Message> + Unpin + Send + 'static>(
    mut write: T,
    mut write_rx: UnboundedReceiver<String>,
    cancel_token: CancellationToken,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(msg) = write_rx.recv() => {
                    let send_msg = Message::Text(Utf8Bytes::from(msg));
                    let _ = write.send(send_msg).await;
                }
                _ = cancel_token.cancelled() => {
                    return;
                }
            }
        }
    });
}

// inits the task that will listen for messages received from the websocket
fn init_read_task<E, T: Stream<Item = Result<Message, E>> + Unpin + Send + 'static>(
    mut read: T,
    read_tx: UnboundedSender<String>,
    cancel_token: CancellationToken,
) where
    E: std::error::Error,
{
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(msg) = read.next() => {
                    let msg = match msg {
                        Ok(m) => m,
                        Err(_) => {
                            return;
                        }
                    };

                    if !msg.is_text() {
                        return;
                    }

                    let text = match msg.to_text() {
                        Ok(t) => t,
                        Err(_) => {
                            return;
                        }
                    };

                    let _ = read_tx.send(text.to_string());
                }
                _ = cancel_token.cancelled() => {
                    return
                }
            }
        }
    });
}

// inits the task that will listen for messages send through the given read channel
// its to have a dedicated task for handling shared_model state change
fn init_recv_msg_task(
    shared_model: Arc<RwLock<SharedModel>>,
    mut read_rx: UnboundedReceiver<String>,
    event_tx: UnboundedSender<CustomEvent>,
    cancel_token: CancellationToken,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(msg) = read_rx.recv() => {
                    if let Err(err) = parse_ws_msg(&msg, Arc::clone(&shared_model)) {
                        let _ = toast::send(&event_tx, ToastMessage::error(err));
                    }
                }
                _ = cancel_token.cancelled() => {
                        return;
                }
            }
        }
    });
}

// parses the msg into the commands and execute them
fn parse_ws_msg(msg: &str, shared_model: Arc<RwLock<SharedModel>>) -> Result<(), String> {
    let words: Vec<&str> = msg.split(" ").collect();

    if words.is_empty() {
        return Err("msg did not contain a cmd".to_string());
    }

    let cmd = words[0];

    match cmd {
        "LobbyInfo" => {
            let lobby_info = parse_payload_json::<LobbyInfo>(&words)?;
            let mut lock = shared_model.write().unwrap();

            lock.lobby_info = Some(lobby_info);
            lock.game_status = Some(GameStatus::Waiting);
        }
        "NewGame" => {
            let new_game = parse_payload_json::<NewGame>(&words)?;
            let mut lock = shared_model.write().unwrap();
            if let Some(lobby_info) = &mut lock.lobby_info {
                lobby_info.data = new_game.data;
            }
            lock.player_info = Some(new_game.players_info)
        }
        "EndGame" => {
            let player_info = parse_payload_json::<PlayerInfoSnapshot>(&words)?;
            let mut lock = shared_model.write().unwrap();
            lock.player_info = Some(player_info);
            lock.game_status = Some(GameStatus::End);
        }
        "Error" => {
            let msg = get_payload_from_words(&words)?;
            return Err(msg);
        }
        "UserId" => {
            let user_id = get_payload_from_words(&words)?;
            let mut lock = shared_model.write().unwrap();
            lock.user_id = Some(user_id);
        }
        "PlayersInfo" => {
            let player_info = parse_payload_json::<PlayerInfoSnapshot>(&words)?;
            let mut lock = shared_model.write().unwrap();
            lock.player_info = Some(player_info);
        }
        "Countdown" => {
            let countdown = parse_payload_json::<i8>(&words)?;

            let mut lock = shared_model.write().unwrap();
            lock.game_status = Some(GameStatus::Countdown(countdown));
        }
        _ => {}
    };

    Ok(())
}

/// Returns the string after the cmd.
/// Returns an error if there is nothing after the first command
/// Assumes the shape of the words is <cmd> <...payload>, meaning everything after cmd is joined
/// into a singular string
fn get_payload_from_words(words: &[&str]) -> Result<String, String> {
    if words.len() < 2 {
        return Err("msg did not contain a payload".to_string());
    }

    let payload_str = words[1..].join(" ");

    Ok(payload_str)
}

/// Deserializes the payload into the given type
fn parse_payload_json<T: for<'a> Deserialize<'a>>(words: &[&str]) -> Result<T, String> {
    let payload = get_payload_from_words(words)?;

    serde_json::from_str::<T>(&payload).map_err(|err| err.to_string())
}

#[cfg(test)]
mod test {
    use crate::util::data_provider::Data;

    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_get_payload_from_words() {
        let s = "cmd asdfasdf".split(" ").collect::<Vec<&str>>();

        let res = get_payload_from_words(&s);

        assert_eq!(res.is_ok(), true)
    }

    #[test]
    fn test_get_payload_from_words_no_payload() {
        let s = "cmd".split(" ").collect::<Vec<&str>>();

        let res = get_payload_from_words(&s);

        assert_eq!(res, Err("msg did not contain a payload".to_string()))
    }

    #[test]
    fn test_get_payload_json() {
        let json_str = json!({
            "lobby_id": "some-id",
            "data": {
                "source": "test source",
                "text": "test text"
            }
        })
        .to_string();

        let s = "cmd ".to_string() + &json_str;
        let s = s.split(" ").collect::<Vec<&str>>();

        let res = parse_payload_json::<LobbyInfo>(&s);

        assert_eq!(
            res,
            Ok(LobbyInfo {
                lobby_id: "some-id".to_string(),
                data: Data {
                    source: "test source".to_string(),
                    text: "test text".to_string()
                }
            })
        );
    }

    #[test]
    fn test_get_payload_json_wrong_format() {
        let json_str = json!({
            "lobby_id": "some-id",
            "data": {
                "source": "test source",
                "wrong format": 123
            }
        })
        .to_string();

        let s = "cmd ".to_string() + &json_str;
        let s = s.split(" ").collect::<Vec<&str>>();

        let res = parse_payload_json::<LobbyInfo>(&s);
        println!("res: {:?}", res);

        assert_eq!(res.is_err(), true)
    }

    #[test]
    fn test_parse_ws_msg_lobby_info() {
        let shared_model: Arc<RwLock<SharedModel>> = Arc::new(RwLock::new(SharedModel::default()));

        let json_str = json!({
            "lobby_id": "some-id",
            "data": {
                "source": "test source",
                "text": "test text"
            }
        })
        .to_string();

        let msg = "LobbyInfo ".to_string() + &json_str;

        assert_eq!(parse_ws_msg(&msg, Arc::clone(&shared_model)), Ok(()));
        assert_eq!(
            shared_model.read().unwrap().lobby_info,
            Some(LobbyInfo {
                lobby_id: "some-id".to_string(),
                data: Data {
                    text: "test text".to_string(),
                    source: "test source".to_string()
                }
            })
        )
    }

    #[test]
    fn test_parse_ws_msg_user_id() {
        let shared_model: Arc<RwLock<SharedModel>> = Arc::new(RwLock::new(SharedModel::default()));

        assert_eq!(
            parse_ws_msg("UserId test-user-id", Arc::clone(&shared_model)),
            Ok(())
        );
        assert_eq!(
            shared_model.read().unwrap().user_id,
            Some("test-user-id".to_string())
        )
    }
}
