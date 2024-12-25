use sysinfo::System;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

pub fn create_graph_chunk<B: Backend>(f: &mut Frame<B>, _sys: &System, chunk: Rect) -> Vec<Rect> {
    let outer_block = Block::default().title("Stats").borders(Borders::ALL);

    let sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunk);

    f.render_widget(outer_block, chunk);

    sub_chunks
}
