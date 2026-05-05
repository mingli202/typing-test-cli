use std::sync::{Arc, RwLock};

use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};

use crate::CustomEvent;
use crate::util::toast::{self, ToastMessage};

use self::models::{LobbyInfo, NewGame, PlayerInfoSnapshot};

mod models;

pub struct SharedModel {
    user_id: Option<String>,
    player_info: Option<PlayerInfoSnapshot>,
    lobby_info: Option<LobbyInfo>,
}

pub struct MultiplayerModel {
    shared_model: Arc<RwLock<SharedModel>>,
    write_tx: UnboundedSender<String>,
}

// Connects to the ws
pub async fn connect_to_ws(model: &mut MultiplayerModel, event_tx: UnboundedSender<CustomEvent>) {
    let request = "ws://localhost:8080/ws".into_client_request().unwrap();

    let (stream, _) = connect_async(request).await.unwrap();
    let (mut write, mut read) = stream.split();

    let (write_tx, mut write_rx) = mpsc::unbounded_channel::<String>();

    let shared_model = Arc::clone(&model.shared_model);

    let read_handle: JoinHandle<color_eyre::Result<()>> = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            let msg = msg?;

            if !msg.is_text() {
                return Ok(());
            }

            let text = msg.to_text()?;

            if let Err(err) = parse_ws_msg(text, Arc::clone(&shared_model)) {
                let _ = toast::send(&event_tx, ToastMessage::error(err));
            }
        }

        Ok(())
    });

    let write_handle: JoinHandle<color_eyre::Result<()>> = tokio::spawn(async move {
        while let Some(msg) = write_rx.recv().await {
            let send_msg = Message::Text(Utf8Bytes::from(msg));
            write.send(send_msg).await?;
        }

        Ok(())
    });

    model.write_tx = write_tx;
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
        "Countdown" => {}
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
}
