use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "Multiplayer typing test tui.")]
pub struct Args {
    /// Whether the user must correct errors before moving on to the next word
    #[arg(short, long, default_value_t = false)]
    pub no_error: bool,

    /// Don't get quotes from the backend and only use the built-in quotes
    #[arg(short, long)]
    pub offline: bool,

    /// Custom path to database of words. See
    /// https://github.com/mingli202/typing-test-tui/blob/main/assets/english.json for shape of
    /// json
    #[arg(short, long)]
    pub words_path: Option<String>,

    /// Custom path to database of quotes. See
    /// https://github.com/mingli202/typing-test-tui/blob/main/assets/quotes.json for shape of json
    #[arg(short, long)]
    pub quotes_path: Option<String>,

    /// How many times per second the tui is drawn. (lower fps might have better performance on lower end devices)
    #[arg(short, long, default_value_t = 30)]
    pub fps: usize,

    /// The tick per second. (lower tps might have better performance on lower end devices)
    #[arg(short, long, default_value_t = 120)]
    pub tps: usize,
}
