use sysinfo::{Process, System};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn render_process<B: Backend>(f: &mut Frame<B>, index: usize, process: &Process, chunk: Rect) {
    let pid = process.pid();
    let name = process.name();

    let process_sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
            Constraint::Percentage(5),
            Constraint::Percentage(90),
            Constraint::Percentage(5),
        ]);

    // render pid
    let pid_paragraph = Paragraph::new(pid.to_string())
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);

    // render name
    // let process_paragraph = Paragraph::new(process_span)
    //     .block(Block::default().borders(Borders::NONE))
    //     .alignment(Alignment::Left);

    f.render_widget(pid_paragraph, chunk);
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
    let num_processes = processes.len();

    let outer_block = Block::default().title("Processes").borders(Borders::ALL);
    f.render_widget(outer_block, chunk);

    let constraints = vec![Constraint::Length(1); num_processes];
    let process_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(chunk);

    for (index, process) in processes.iter().enumerate() {
        render_process(f, index, process, process_chunks[index]);
    }
}
