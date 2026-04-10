use std::io::{self, Stdout};

use crossterm::cursor;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use typing_test_tui::run;

use clap::Parser;

// TODO: --offline mode uses my own data
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// How many times per second the tui is drawn.
    #[arg(short, long, default_value_t = 30)]
    fps: usize,

    /// The tick per second
    #[arg(short, long, default_value_t = 120)]
    tps: usize,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let Args { fps, tps } = Args::parse();

    let mut term = setup_terminal()?;
    run(&mut term, fps, tps).await?;
    teardown_terminal(&mut term)?;

    Ok(())
}

fn setup_terminal() -> color_eyre::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    Ok(terminal)
}

fn teardown_terminal(_terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> color_eyre::Result<()> {
    let mut stdout = io::stdout();
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}
