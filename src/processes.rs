use sysinfo::{Process, System};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

#[allow(clippy::cast_precision_loss)]
fn add_process(_index: usize, process: &Process, rows: &mut Vec<Row>) {
    let pid = process.pid().to_string();
    // name should be truncated after 21 chars
    let name = process
        .name()
        .to_string_lossy()
        .to_string()
        .chars()
        .take(21)
        .collect();
    let mem = (process.memory() as f64) / (1_000_000.0);
    let mem_fmt = format!("{mem:.2}");
    let cpu_usage = format!("{:.2}%", process.cpu_usage());
    let uptime = format!("{}", process.run_time());
    let euid_egid_fmt = format!(
        "{}/{}",
        **process.effective_user_id().unwrap(),
        *process.effective_group_id().unwrap()
    );

    let cells = vec![pid, name, mem_fmt, cpu_usage, uptime, euid_egid_fmt]
        .into_iter()
        .map(Cell::from);

    let row = Row::new(cells);
    rows.push(row);
}

pub fn create_processes_chunk<B: Backend>(f: &mut Frame<B>, sys: &System, chunk: Rect) {
    let outer_chunk = Block::default()
        .borders(Borders::ALL)
        .title("Processes")
        .border_style(Style::default().fg(tui::style::Color::White));
    f.render_widget(outer_chunk, chunk);

    // table goes here
    let inner_chunk = Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .horizontal_margin(3)
        .vertical_margin(2)
        .constraints([Constraint::Min(1)].as_ref())
        .split(chunk);

    let min_memory_usage = 50_000_000; // ignore any processes <50 MB
    let mut processes: Vec<_> = sys
        .processes()
        .values()
        .filter(|process| process.memory() > min_memory_usage)
        .collect();
    // sort by memory size in descending order
    processes.sort_by_key(|b| std::cmp::Reverse(b.memory()));

    let header_cells = ["PID", "Name", "Mem (MB)", "CPU", "Uptime (s)", "EUID/EGID"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(tui::style::Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells);
    let mut process_rows: Vec<Row> = vec![];
    process_rows.push(Row::new(vec![Cell::from("")]));

    for (index, process) in processes.iter().enumerate() {
        add_process(index, process, &mut process_rows);
    }

    let table = Table::new(process_rows)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .widths(&[
            Constraint::Percentage(10), // pid
            Constraint::Percentage(32), // name
            Constraint::Percentage(13), // memory
            Constraint::Percentage(10), // cpu
            Constraint::Percentage(15), // uptime
            Constraint::Percentage(18), // euid/egid
        ]);

    f.render_widget(table, inner_chunk[0]);
}
