use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::{KeyCode, KeyModifiers};
use itertools::Itertools;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Offset, Rect, Size};
use ratatui::macros::{line, span};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols;
use ratatui::text::ToSpan;
use ratatui::widgets::{Block, LineGauge, Paragraph, Widget, Wrap};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_util::sync::CancellationToken;

use crate::CustomEvent;
use crate::msg::Msg;
use crate::typing::{Typing, view_typing_test};
use crate::util::toast::ToastMessage;
use crate::util::{toast, view_helpers};

use self::connect_helpers::connect_to_ws;
use self::models::{LobbyInfo, PlayerInfo, PlayersInfoSnapshot, WsMsg};

mod connect_helpers;
mod models;

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum GameStatus {
    Waiting,
    Countdown(i8),
    Playing,
    Done,
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
    last_sent_update: Instant,

    cancel_token: CancellationToken,
}

impl MultiplayerModel {
    pub fn new(event_tx: UnboundedSender<CustomEvent>) -> Self {
        let (write_tx, write_rx) = mpsc::unbounded_channel::<String>();

        let model = MultiplayerModel {
            game_model: Arc::new(RwLock::new(GameModel::default())),
            write_tx,
            cancel_token: CancellationToken::new(),
            input_lobby_id: vec![],
            last_sent_update: Instant::now(),
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

pub fn update(
    model: &mut MultiplayerModel,
    event_tx: &UnboundedSender<CustomEvent>,
    msg: Msg,
) -> Option<crate::action::Action> {
    let is_in_lobby = {
        let lock = model.game_model.read().unwrap();
        lock.lobby.is_some()
    };

    if model.cancel_token.is_cancelled() {
        let _ = toast::send(
            event_tx,
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
                        let done = lobby.typing.on_type(c);

                        if done {
                            send_update_stats(model, lobby);
                            lock.game_status = Some(GameStatus::Done);
                        }
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
            KeyCode::Enter => {
                let can_start = {
                    let lock = model.game_model.read().unwrap();

                    let is_leader = if let Some(ref players) = lock.players_info
                        && let Some(ref user_id) = lock.user_id
                    {
                        players
                            .players
                            .get(user_id)
                            .is_some_and(|player| player.is_leader)
                    } else {
                        false
                    };

                    let is_waiting = if let Some(ref game_status) = lock.game_status
                        && (*game_status == GameStatus::Waiting || *game_status == GameStatus::End)
                    {
                        true
                    } else {
                        false
                    };

                    is_leader && is_waiting
                };

                if can_start {
                    model.send_msg(WsMsg::StartGame);
                }
            }
            _ => {}
        },
        Msg::Tick => {
            if model.last_sent_update.elapsed() > Duration::from_millis(200) {
                let lock = model.game_model.read().unwrap();

                if lock.game_status == Some(GameStatus::Playing)
                    && let Some(ref lobby) = lock.lobby
                {
                    send_update_stats(model, lobby);
                }

                model.last_sent_update = Instant::now();
            }
        }
        _ => {}
    };

    None
}

// Sends the user's stats to the server
fn send_update_stats(model: &MultiplayerModel, lobby: &Lobby) {
    let wpm = lobby.typing.net_wpm();
    let mut progress = lobby.typing.letters_typed() * 100 / lobby.lobby_info.data.text.len();

    if progress > 100 {
        progress = 100;
    }

    model.send_msg(WsMsg::UpdateStats(wpm, progress as u8));
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

            let is_leader = {
                if let Some(ref players) = lock.players_info
                    && let Some(ref user_id) = lock.user_id
                {
                    players
                        .players
                        .get(user_id)
                        .is_some_and(|player| player.is_leader)
                } else {
                    false
                }
            };

            let data_area = area.centered(Constraint::Max(80), Constraint::Length(3));
            let mut status_area = data_area.offset(Offset { x: 0, y: -2 });
            if let Some(ref game_status) = lock.game_status {
                status_area = view_game_status(game_status, is_leader, status_area, buf);
            }

            let max_offset = status_area.y.saturating_sub(1);
            if let Some(ref players) = lock.players_info
                && let Some(ref user_id) = lock.user_id
            {
                view_players(players, user_id, max_offset, area, buf);
            }

            if let Some(ref game_status) = lock.game_status
                && *game_status == GameStatus::Done
            {
            } else {
                view_typing_test(&lobby.typing, data_area, buf);
            }

            view_helpers::view_bottom_menu(&["Singleplayer <C-p>  Leave <Esc>"], area, buf);
        }
    };
}

/// Show the game status
fn view_game_status(
    game_status: &GameStatus,
    is_leader: bool,
    area: Rect,
    buf: &mut Buffer,
) -> Rect {
    let txt = match game_status {
        GameStatus::Done => line!("Waiting for others to finish..."),
        GameStatus::Waiting => {
            if is_leader {
                line!("Press ENTER to start")
            } else {
                line!("Waiting for leader")
            }
        }
        GameStatus::Countdown(count_down) => line!("Start in ", count_down.to_span()),
        GameStatus::Playing => line!("Go!"),
        GameStatus::End => {
            if is_leader {
                line!("Game has ended, press ENTER to restart")
            } else {
                line!("Game has ended, wait for leader to start again or leave")
            }
        }
    };

    let p = Paragraph::new(txt).wrap(Wrap { trim: true }).centered();
    let n_line = p.line_count(area.width);

    let area = area.offset(Offset {
        x: 0,
        y: -(n_line as i32 - 1),
    });

    p.render(area, buf);

    area
}

/// renders the players
fn view_players(
    players_info: &PlayersInfoSnapshot,
    me: &str,
    max_offset: u16,
    area: Rect,
    buf: &mut Buffer,
) {
    let area = area
        .centered_horizontally(Constraint::Max(80))
        .offset(Offset { x: 0, y: 2 });

    let mut area = area.resize(Size {
        width: area.width,
        height: (area.height / 2).saturating_sub(6),
    });

    if area.height == 0 {
        return;
    }

    let players = players_info
        .players
        .iter()
        .filter(|(id, _)| *id != me)
        .sorted_by_key(|(id, player)| (player.progress_percent, *id))
        .rev();

    if let Some(me_info) = players_info.players.get(me) {
        view_player(me_info, true, area, buf);
        area.y += 1;
    }

    for (i, (_, player)) in players.enumerate() {
        let area = area.offset(Offset { x: 0, y: i as i32 });

        if area.y > max_offset {
            break;
        };

        view_player(player, false, area, buf);
    }
}

fn view_player(player: &PlayerInfo, is_me: bool, area: Rect, buf: &mut Buffer) {
    let ratio = player.progress_percent as f64 / 100.0;

    let mut label = span!(format!("{} {}%", player.name, player.progress_percent));

    if is_me {
        label = label.underlined();
    }

    LineGauge::default()
        .label(label)
        .filled_style(Style::new().white())
        .filled_symbol(symbols::line::THICK_HORIZONTAL)
        .unfilled_symbol(" ")
        .ratio(ratio.clamp(0.0, 1.0))
        .render(area, buf)
}
