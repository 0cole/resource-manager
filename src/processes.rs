use sysinfo::{Process, System};
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

fn add_process(_index: usize, process: &Process, rows: &mut Vec<Row>) {
    let pid = process.pid().to_string();
    let name = process.name().to_string_lossy().to_string();
    let mem = process.memory().to_string();

    let cells = vec![pid, name, mem].into_iter().map(Cell::from);

    let row = Row::new(cells);
    rows.push(row);
}

pub fn create_processes_chunk<B: Backend>(f: &mut Frame<B>, sys: &System, chunk: Rect) {
    let min_memory_usage = 50_000_000; // 50 MB
    let mut processes: Vec<_> = sys
        .processes()
        .values()
        .filter(|process| process.memory() > min_memory_usage)
        .collect();
    // sort by memory size in descending order
    processes.sort_by_key(|b| std::cmp::Reverse(b.memory()));

    let header_cells = ["PID", "Name", "Memory (Bytes)"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(tui::style::Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells);
    let mut process_rows: Vec<Row> = vec![];

    for (index, process) in processes.iter().enumerate() {
        add_process(index, process, &mut process_rows);
    }

    let table = Table::new(process_rows)
        .header(header)
        .block(Block::default().title("Processes").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ]);

    f.render_widget(table, chunk);
}
