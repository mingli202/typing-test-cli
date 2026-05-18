use std::cmp::Ordering;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::event::{KeyCode, KeyModifiers};
use itertools::Itertools;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Offset, Rect};
use ratatui::macros::{line, span};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::text::ToSpan;
use ratatui::widgets::{Block, LineGauge, Paragraph, Widget};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_util::sync::CancellationToken;

use crate::CustomEvent;
use crate::model::Screen;
use crate::msg::Msg;
use crate::singleplayer::SinglePlayerScreen;
use crate::typing::{Typing, view_typing_test};
use crate::util::toast::ToastMessage;
use crate::util::{toast, view_helpers};

use self::connect_helpers::connect_to_ws;
use self::models::{LobbyInfo, PlayersInfoSnapshot, WsMsg};

mod connect_helpers;
mod models;

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum GameStatus {
    Waiting,
    Countdown(i8),
    Playing,
    End,
}

pub struct Lobby {
    lobby_info: LobbyInfo,
    typing: Typing,
}

#[derive(Default)]
pub struct PendingLobby {
    lobby_id: Option<String>,
    pending_players: Option<PlayersInfoSnapshot>,
}

#[derive(Default)]
pub struct GameModel {
    user_id: Option<String>,
    active_lobby_id: Option<String>,
    pending_lobby: PendingLobby,
    players_info: Option<PlayersInfoSnapshot>,
    lobby: Option<Lobby>,
    game_status: Option<GameStatus>,
}

pub struct MultiplayerModel {
    game_model: Arc<RwLock<GameModel>>,
    write_tx: UnboundedSender<String>,
    input_lobby_id: Vec<char>,

    cancel_token: CancellationToken,
    event_tx: UnboundedSender<CustomEvent>,
}

impl MultiplayerModel {
    pub fn new(event_tx: UnboundedSender<CustomEvent>) -> Self {
        let (write_tx, write_rx) = mpsc::unbounded_channel::<String>();

        let model = MultiplayerModel {
            game_model: Arc::new(RwLock::new(GameModel::default())),
            write_tx,
            cancel_token: CancellationToken::new(),
            input_lobby_id: vec![],
            event_tx: event_tx.clone(),
        };

        let game_model = Arc::clone(&model.game_model);
        let cancel_token = model.cancel_token.clone();
        tokio::spawn(async move {
            if let Err(err) =
                connect_to_ws(game_model, cancel_token.clone(), event_tx.clone(), write_rx).await
            {
                let _ = toast::send(&event_tx, ToastMessage::error(err.to_string()));
                cancel_token.cancel();
            }
        });

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
            let mut lock = self.game_model.write().unwrap();
            lock.pending_lobby.lobby_id = Some(group_id);
        }
    }
}

impl Drop for MultiplayerModel {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

pub fn update(model: &mut MultiplayerModel, msg: Msg) -> Option<crate::action::Action> {
    let is_in_lobby = {
        let lock = model.game_model.read().unwrap();
        lock.lobby.is_some()
    };

    if model.cancel_token.is_cancelled() {
        let _ = toast::send(
            &model.event_tx,
            ToastMessage::error("Multiplayer crashed, back to singleplayer".to_string()),
        );
        return Some(crate::action::Action::SwitchToSinglePlayer);
    }

    if is_in_lobby {
        update_lobby_info(model, msg)
    } else {
        update_no_lobby_info(model, msg)
    }
}

pub fn view(model: &MultiplayerModel, area: Rect, buf: &mut Buffer) {
    let lock = model.game_model.read().unwrap();

    match lock.lobby {
        None => {
            let t = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => n.as_secs(),
                Err(_) => 0,
            };

            let lobby_text: String = model.input_lobby_id.iter().collect();
            let cursor = span![" "].bg(if t % 2 == 0 {
                Color::White
            } else {
                Color::Reset
            });

            let line = line![lobby_text, cursor];
            let typing_box = Paragraph::new(line).block(Block::bordered().title("Lobby Id"));
            let enter_text = "Enter".to_span();
            let create_text = "Create Lobby <C-n>".to_span();

            let centered = area.centered(
                Constraint::Length(10 + enter_text.width() as u16 + enter_text.width() as u16),
                Constraint::Length(3),
            );

            let layout = Layout::new(
                Direction::Horizontal,
                vec![
                    Constraint::Length(10 + enter_text.width() as u16),
                    Constraint::Length(enter_text.width() as u16),
                ],
            )
            .spacing(2)
            .split(centered);

            typing_box.render(layout[0], buf);
            enter_text.render(layout[1].centered_vertically(Constraint::Length(1)), buf);

            let create_text_area = area
                .centered(
                    Constraint::Length(create_text.width() as u16),
                    Constraint::Length(1),
                )
                .offset(Offset {
                    x: 0,
                    y: layout[0].height as i32,
                });
            create_text.render(create_text_area, buf);

            view_helpers::view_bottom_menu(&["Singleplayer <C-p>"], area, buf);
        }
        Some(ref lobby) => {
            let lobby_line = line!("Id: ", span!(lobby.lobby_info.lobby_id));
            let lobby_line_area =
                area.centered_horizontally(Constraint::Length(lobby_line.width() as u16));
            lobby_line.render(lobby_line_area, buf);

            if let Some(ref players) = lock.players_info
                && let Some(ref user_id) = lock.user_id
            {
                view_players(players, user_id, area, buf);
            }

            let data_area = area.centered(Constraint::Max(80), Constraint::Length(3));
            view_typing_test(&lobby.typing, data_area, buf);

            view_helpers::view_bottom_menu(&["Singleplayer <C-p>  Leave <Esc>"], area, buf);
        }
    };
}

/// renders the players
fn view_players(players: &PlayersInfoSnapshot, me: &str, area: Rect, buf: &mut Buffer) {
    let area = area
        .centered_horizontally(Constraint::Max(80))
        .offset(Offset { x: 0, y: 2 });

    let players = players
        .players
        .iter()
        .sorted_by(|(id_a, player_a), (_, player_b)| {
            if *id_a == me {
                return Ordering::Less;
            }

            Ord::cmp(&player_a.progress_percent, &player_b.progress_percent)
        });

    for (i, (id, player)) in players.enumerate() {
        let area = area.offset(Offset { x: 0, y: i as i32 });

        let id = &id[..6];
        LineGauge::default()
            .label(format!("{} {}%", id, player.progress_percent))
            .filled_style(Style::new().white())
            .filled_symbol(symbols::line::THICK_HORIZONTAL)
            .ratio(player.progress_percent as f64 / 100.0)
            .render(area, buf)
    }
}

/// the update part of the view without a lobby
fn update_no_lobby_info(model: &mut MultiplayerModel, msg: Msg) -> Option<crate::action::Action> {
    match msg {
        Msg::Key(key) => {
            match key.code {
                KeyCode::Char(c) => {
                    if c == 'n' && matches!(key.modifiers, KeyModifiers::CONTROL) {
                        model.send_msg(WsMsg::NewGroup);
                        return None;
                    }

                    if !c.is_ascii_lowercase() || model.input_lobby_id.len() >= 6 {
                        return None;
                    }

                    model.input_lobby_id.push(c);
                }
                KeyCode::Backspace => {
                    model.input_lobby_id.pop();
                }
                KeyCode::Enter => {
                    let lobby_id = model.input_lobby_id.iter().collect();
                    model.send_msg(WsMsg::JoinGroup(lobby_id));
                }
                _ => {}
            };
        }
        Msg::Tick => {}
        _ => {}
    };

    None
}

/// the update part of the view with a lobby
fn update_lobby_info(model: &mut MultiplayerModel, msg: Msg) -> Option<crate::action::Action> {
    match msg {
        Msg::Key(key) => match key.code {
            KeyCode::Char(c) => {
                let is_playing = {
                    let lock = model.game_model.read().unwrap();
                    lock.game_status == Some(GameStatus::Playing)
                };

                if is_playing {
                    let mut lock = model.game_model.write().unwrap();
                    if let Some(lobby) = &mut lock.lobby {
                        lobby.typing.on_type(c);
                    }
                }
            }
            KeyCode::Esc => {
                model.send_msg(WsMsg::LeaveGroup);
            }
            KeyCode::Backspace => {
                let is_playing = {
                    let lock = model.game_model.read().unwrap();
                    lock.game_status == Some(GameStatus::Playing)
                };

                if is_playing {
                    let mut lock = model.game_model.write().unwrap();
                    if let Some(lobby) = &mut lock.lobby {
                        lobby.typing.on_backspace();
                    }
                }
            }
            _ => {}
        },
        Msg::Tick => {}
        _ => {}
    };

    None
}
