use typing_test_tui::App;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut app = App::new().await;
    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
