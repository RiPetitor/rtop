use std::error::Error;
use std::io;
use std::time::{Duration, Instant};

use crossterm::cursor::Show;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;

use rtop::app::{App, Config};
use rtop::events::{AppEvent, handle_event};
use rtop::ui;

type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<(), Box<dyn Error>> {
    let config = match Config::from_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };
    let tick_rate = config.tick_rate;
    let mut terminal = setup_terminal()?;
    install_panic_hook();
    let mut app = App::new(config);

    let result = run_app(&mut terminal, &mut app, tick_rate);
    restore_terminal(&mut terminal)?;

    if let Err(err) = result {
        eprintln!("rtop error: {err}");
    }

    Ok(())
}

fn setup_terminal() -> io::Result<AppTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut AppTerminal) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal_raw();
        default_hook(info);
    }));
}

fn restore_terminal_raw() {
    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen, Show);
}

fn run_app(terminal: &mut AppTerminal, app: &mut App, tick_rate: Duration) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        app.tick();
        terminal.draw(|frame| ui::render(frame, app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            let event = match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => AppEvent::Key(key),
                Event::Resize(w, h) => AppEvent::Resize(w, h),
                _ => continue,
            };

            if handle_event(app, event).should_exit() {
                return Ok(());
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if handle_event(app, AppEvent::Tick).should_exit() {
                return Ok(());
            }
            last_tick = Instant::now();
        }
    }
}
