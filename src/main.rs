use tokio::sync::mpsc;
use typing_test_tui::App;
use typing_test_tui::config::Config;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let (config_tx, config_rx) = mpsc::unbounded_channel();

    Config::init(config_rx);
    let mut app = App::new(config_tx).await;
    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
