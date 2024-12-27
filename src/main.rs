mod graphs;
mod stats;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, Result},
    time,
};
use std::{thread, time::Duration};
use sysinfo::{Disks, System};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    Terminal,
};

fn refresh_system(sys: &mut System) {
    sys.refresh_all();
}

fn ui<B: Backend>(terminal: &mut Terminal<B>, sys: &System, disks: &Disks) -> Result<()> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Length(40), Constraint::Length(50)].as_ref())
            .split(f.size());

        stats::create_stats_chunk(f, sys, disks, chunks[0]);
        graphs::create_graph_chunk(f, sys, chunks[1]);
    })?;
    Ok(())
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut sys = System::new_all();
    let disks = Disks::new_with_refreshed_list();

    let mut tick = 0;

    loop {
        // every 10 ticks (1 sec) redraw tui
        if tick % 10 == 0 {
            refresh_system(&mut sys);
            ui(&mut terminal, &sys, &disks)?;
        }

        // exit if q is pressed
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        // each tick = 10 ms + computation amt
        let sleep_time = time::Duration::from_millis(10);
        thread::sleep(sleep_time);
        tick += 1;
    }

    // tear down terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    Ok(())
}
