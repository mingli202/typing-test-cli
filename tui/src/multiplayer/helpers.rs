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

use super::models::{LobbyInfo, NewGame, PlayerInfoSnapshot};
use super::{GameStatus, SharedModel};

/// Connects to the ws
pub async fn connect_to_ws(
    shared_model: Arc<RwLock<SharedModel>>,
    cancel_token: CancellationToken,
    event_tx: UnboundedSender<CustomEvent>,
    write_rx: UnboundedReceiver<String>,
) {
    let request = match "ws://localhost:8080/ws".into_client_request() {
        Ok(res) => res,
        Err(e) => {
            let _ = toast::send(&event_tx, ToastMessage::error(e.to_string()));
            return;
        }
    };

    let (stream, _) = match connect_async(request).await {
        Ok(ok) => ok,
        Err(e) => {
            let _ = toast::send(&event_tx, ToastMessage::error(e.to_string()));
            return;
        }
    };

    let (write, read) = stream.split();

    let (read_tx, read_rx) = mpsc::unbounded_channel::<String>();

    init_write_task(write, write_rx, cancel_token.clone());
    init_read_task(read, read_tx, cancel_token.clone());
    init_recv_msg_task(shared_model, read_rx, event_tx, cancel_token.clone());
}

/// inits the task that will listen for messages to be sent to the websocket
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

/// inits the task that will listen for messages received from the websocket
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
                        continue;
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

/// inits the task that will listen for messages send through the given read channel
/// its to have a dedicated task for handling shared_model state change
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

/// parses the msg into the commands and execute them
fn parse_ws_msg(msg: &str, shared_model: Arc<RwLock<SharedModel>>) -> Result<(), String> {
    let words: Vec<&str> = msg.split(" ").collect();

    if words.is_empty() || words[0].is_empty() {
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
        _ => return Err(format!("cmd {} unsupported", cmd)),
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
    use std::collections::VecDeque;
    use std::fmt::Display;
    use std::task::Poll;
    use std::time::Duration;

    use crate::multiplayer::MultiplayerModel;
    use crate::util::data_provider::Data;

    use super::*;
    use futures::{Sink, StreamExt};
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

    #[derive(Debug, PartialEq, PartialOrd)]
    struct MockError {
        err: String,
    }

    impl Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.err)
        }
    }

    impl std::error::Error for MockError {}

    struct MockWebsocketStream {
        messages: VecDeque<Message>,
    }

    impl MockWebsocketStream {
        fn new(messages: Vec<String>) -> MockWebsocketStream {
            MockWebsocketStream {
                messages: messages
                    .into_iter()
                    .map(|msg| Message::Text(Utf8Bytes::from(&msg)))
                    .collect(),
            }
        }
    }

    impl Stream for MockWebsocketStream {
        type Item = Result<Message, MockError>;
        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            let msg = self.get_mut().messages.pop_front();
            Poll::Ready(msg.map(Ok))
        }
    }

    impl Sink<Message> for MockWebsocketStream {
        type Error = MockError;

        fn poll_ready(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn start_send(self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
            self.get_mut().messages.push_back(item);
            Ok(())
        }
        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn poll_close(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            self.poll_flush(cx)
        }
    }

    #[tokio::test]
    async fn test_init_write_task() {
        // Arrange
        let s = MockWebsocketStream::new(vec![
            "NewGroup".to_string(),
            "JoinGroup asdfgh".to_string(),
            "LeaveGroup".to_string(),
        ]);
        let (write, mut read) = s.split();

        let (write_tx, write_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        let shared_model: Arc<RwLock<SharedModel>> = Arc::new(RwLock::new(SharedModel::default()));
        let model = MultiplayerModel {
            cancel_token: CancellationToken::new(),
            shared_model,
            write_tx,
        };

        // Act
        init_write_task(write, write_rx, model.cancel_token.clone());

        // Assert
        assert_eq!(
            read.next().await,
            Some(Ok(Message::Text(Utf8Bytes::from("NewGroup"))))
        );
        assert_eq!(
            read.next().await,
            Some(Ok(Message::Text(Utf8Bytes::from("JoinGroup asdfgh"))))
        );
        assert_eq!(
            read.next().await,
            Some(Ok(Message::Text(Utf8Bytes::from("LeaveGroup"))))
        );
        assert_eq!(read.next().await, None);

        // Cleanup
        model.cancel_token.cancel();
    }

    #[tokio::test]
    async fn test_init_read_task() {
        // Arrange
        let s = MockWebsocketStream::new(vec![
            "NewGroup".to_string(),
            "JoinGroup asdfgh".to_string(),
            "LeaveGroup".to_string(),
        ]);
        let (_, read) = s.split();

        let (read_tx, mut read_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        let cancel_token = CancellationToken::new();

        // Act
        init_read_task(read, read_tx, cancel_token.clone());

        // Assert
        assert_eq!(read_rx.recv().await, Some("NewGroup".to_string()));
        assert_eq!(read_rx.recv().await, Some("JoinGroup asdfgh".to_string()));
        assert_eq!(read_rx.recv().await, Some("LeaveGroup".to_string()));

        // Cleanup
        cancel_token.cancel();
    }

    #[tokio::test]
    async fn test_init_recv_msg_task() {
        // Arrange
        let (read_tx, read_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let (write_tx, _) = tokio::sync::mpsc::unbounded_channel::<String>();
        let (event_tx, _) = tokio::sync::mpsc::unbounded_channel::<CustomEvent>();

        let shared_model: Arc<RwLock<SharedModel>> = Arc::new(RwLock::new(SharedModel::default()));
        let model = MultiplayerModel {
            cancel_token: CancellationToken::new(),
            shared_model: Arc::clone(&shared_model),
            write_tx,
        };

        let lobby_info = LobbyInfo {
            lobby_id: "asdfgh".to_string(),
            data: Data {
                text: "text".to_string(),
                source: "source".to_string(),
            },
        };

        // Act
        init_recv_msg_task(
            Arc::clone(&model.shared_model),
            read_rx,
            event_tx,
            model.cancel_token.clone(),
        );

        let msg1 = "LobbyInfo ".to_string() + &serde_json::to_string(&lobby_info).unwrap();

        let _ = read_tx.send(msg1);
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Assert
        {
            let lock = shared_model.read().unwrap();
            assert_eq!(lock.lobby_info, Some(lobby_info));
        }

        // Cleanup
        model.cancel_token.cancel();
    }
}
