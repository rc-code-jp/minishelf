mod app;
mod git_status;
mod input;
mod preview;
mod tree;
mod ui;

use std::io;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::{App, REFRESH_INTERVAL};
use crate::input::map_event;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Startup directory. Defaults to current directory.
    path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let startup_root = resolve_startup_root(args.path)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let _cleanup = TerminalCleanup;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run_result = run(&mut terminal, startup_root);
    let cursor_result = terminal.show_cursor();

    match (run_result, cursor_result) {
        (Err(err), _) => Err(err),
        (Ok(_), Err(err)) => Err(err.into()),
        (Ok(_), Ok(_)) => Ok(()),
    }
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, startup_root: PathBuf) -> Result<()> {
    let mut app = App::new(startup_root)?;
    let mut last_tick = Instant::now();

    while !app.should_quit {
        app.poll_background_tasks();
        terminal.draw(|f| ui::render(f, &app))?;

        let timeout = REFRESH_INTERVAL.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                if let Some(command) = map_event(key_event) {
                    app.handle_command(command);
                }
            }
        }

        if let Some(path) = app.take_external_markdown_preview_request() {
            match run_glow_preview(terminal, &path) {
                Ok(()) => {}
                Err(err) => {
                    app.status_message = format!("glow failed: {err}");
                }
            }
            last_tick = Instant::now();
            continue;
        }

        if last_tick.elapsed() >= REFRESH_INTERVAL {
            app.periodic_refresh();
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn run_glow_preview(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    path: &std::path::Path,
) -> Result<()> {
    let glow_ready = ProcessCommand::new("glow")
        .arg("--version")
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if !glow_ready {
        anyhow::bail!("glow not found (install: brew install glow)");
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    let glow_result = ProcessCommand::new("glow").arg("-p").arg(path).status();

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    *terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    match glow_result {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => anyhow::bail!("glow exited with status: {status}"),
        Err(err) => anyhow::bail!("{err}"),
    }
}

fn resolve_startup_root(path: Option<PathBuf>) -> Result<PathBuf> {
    let candidate = match path {
        Some(p) => p,
        None => std::env::current_dir()?,
    };

    let canonical = std::fs::canonicalize(candidate)?;
    if !canonical.is_dir() {
        anyhow::bail!("startup path must be a directory");
    }

    Ok(canonical)
}

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}
