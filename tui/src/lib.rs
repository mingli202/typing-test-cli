use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::DefaultTerminal;
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::interval;

use self::action::Action;
use self::args::Args;
use self::model::{AppModel, handle_action, handle_toast_action, update, view};
use self::msg::Msg;
use self::util::toast::ToastAction;

pub mod action;
pub mod args;
mod model;
mod msg;
mod multiplayer;
mod singleplayer;
pub mod typing;
mod util;

pub const BACKEND_DOMAIN: &str = if cfg!(debug_assertions) {
    "localhost:8080"
} else {
    "typing-test-tui-backend.onrender.com"
};

pub fn ws_url() -> String {
    let schema = if cfg!(debug_assertions) { "ws" } else { "wss" };

    format!("{}://{}/ws", schema, BACKEND_DOMAIN)
}

pub enum CustomEvent {
    Quit,
    Tick,
    Render,
    Key(KeyEvent),
    ToastAction(ToastAction),
    FocusGained,
    FocusLost,
}

pub async fn run(terminal: &mut DefaultTerminal, args: Args) -> color_eyre::Result<()> {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    init_event_loop(event_tx.clone(), args.fps, args.tps);

    let mut app_model = AppModel::new(event_tx, args).await?;

    while !app_model.exit {
        let mut maybe_action: Option<Action> = tokio::select! {
            Some(custom_event) = event_rx.recv() => {
                match custom_event {
                    CustomEvent::Quit => Some(Action::Quit),
                    CustomEvent::Tick => update(&mut app_model, Msg::Tick),
                    CustomEvent::Render => {
                        terminal.draw(|frame| view(&app_model, frame))?;
                        None
                    }
                    CustomEvent::Key(key) => update(&mut app_model, Msg::Key(key)),
                    CustomEvent::ToastAction(action) => handle_toast_action(&mut app_model, action),
                    CustomEvent::FocusGained => update(&mut app_model, Msg::FocusGained),
                    CustomEvent::FocusLost => update(&mut app_model, Msg::FocusLost),
                }

            },
        };

        while let Some(action) = maybe_action {
            maybe_action = handle_action(&mut app_model, action);
        }
    }

    Ok(())
}

fn init_event_loop(event_tx: UnboundedSender<CustomEvent>, fps: usize, tps: usize) {
    tokio::spawn(async move {
        let render_duration_secs = 1.0 / fps as f64;
        let tick_duration_secs = 1.0 / tps as f64;

        let mut tick_interval = interval(Duration::from_secs_f64(tick_duration_secs));
        let mut render_interval = interval(Duration::from_secs_f64(render_duration_secs));

        let mut event_stream = EventStream::new();

        loop {
            select! {
                _ = tick_interval.tick() => {
                    let _ = event_tx.send(CustomEvent::Tick);
                }
                _ = render_interval.tick() => {
                    let _ = event_tx.send(CustomEvent::Render);
                }
                maybe_event = event_stream.next().fuse() => {
                    let custom_event = match maybe_event {
                        Some(Ok(e)) => {
                            match e {
                                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => CustomEvent::Key(key_event),
                                Event::FocusGained => CustomEvent::FocusGained,
                                Event::FocusLost => CustomEvent::FocusLost,
                                _ => continue,
                            }
                        }
                        Some(Err(_)) => continue,
                        None => break,
                    };

                    if event_tx.send(custom_event).is_err() {
                        break;
                    }
                }
            }
        }
    });
}
