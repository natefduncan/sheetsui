use std::{path::PathBuf, process::ExitCode, time::{Duration, Instant}};

use clap::Parser;
use crossterm::event;
use serde_json::to_writer;
use std::io::Write;

use ui::Workspace;

mod book;
mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg()]
    workbook: Option<PathBuf>,
    #[arg(default_value_t=String::from("en"), short, long)]
    locale_name: String,
    #[arg(default_value_t=String::from("America/New_York"), short, long)]
    timezone_name: String,
    #[arg(long)]
    log_input: Option<PathBuf>,
    /// A theme name or a path to a theme file.
    #[arg(long)]
    theme: Option<String>,
    /// A keys file of `nnoremap` and `nunmap` lines, instead of the one in the
    /// config directory.
    #[arg(long)]
    keys: Option<PathBuf>,
}

/// Reads the next event. With a timeout, returns `None` if it ran out first.
type ReadFn = Box<dyn FnMut(Option<Duration>) -> anyhow::Result<Option<event::Event>>>;

fn read_event(timeout: Option<Duration>) -> anyhow::Result<Option<event::Event>> {
    if let Some(timeout) = timeout {
        if !event::poll(timeout)? {
            return Ok(None);
        }
    }
    Ok(Some(event::read()?))
}

fn run(terminal: &mut ratatui::DefaultTerminal, args: Args) -> anyhow::Result<ExitCode> {
    let mut ws = if let Some(workbook_path) = &args.workbook {
        Workspace::load(workbook_path, &args.locale_name, &args.timezone_name)?
    } else {
        Workspace::new_empty(&args.locale_name, &args.timezone_name)?
    };
    if let Some(path) = ui::CommandHistory::default_path() {
        ws.state.command_history.use_file(&path);
    }
    if let Some(path) = ui::CommandHistory::search_path() {
        ws.state.search_history.use_file(&path);
    }
    ws.init_config();
    ws.init_theme(args.theme.as_deref());
    ws.init_keys(args.keys.as_deref());
    let mut read_func: ReadFn = if let Some(log_path) = args.log_input {
        {
            let log_file = std::fs::File::create(log_path)?;
            Box::new(move |timeout| {
                let Some(evt) = read_event(timeout)? else {
                    return Ok(None);
                };
                to_writer(&log_file, &evt)?;
                writeln!(&log_file)?;
                Ok(Some(evt))
            })
        }
    } else {
        Box::new(read_event)
    };
    loop {
        terminal.draw(|frame| ui::render::draw(frame, &mut ws))?;
        let timeout = ws
            .key_deadline()
            .map(|deadline| deadline.saturating_duration_since(Instant::now()));
        let code = match read_func(timeout)? {
            Some(evt) => ws.handle_input(evt)?,
            None => ws.handle_key_timeout()?,
        };
        if let Some(code) = code {
            return Ok(code);
        }
    }
}

fn main() -> anyhow::Result<ExitCode> {
    let args = Args::parse();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(&mut terminal, args);
    ratatui::restore();
    app_result
}
