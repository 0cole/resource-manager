use sysinfo::{Cpu, System};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

fn get_global_cpu_usage(sys: &System) -> f32 {
    sys.global_cpu_usage()
}

fn get_individual_cpus(sys: &System) -> Vec<&Cpu> {
    let mut individual_cpus: Vec<&Cpu> = Vec::new();
    for cpu in sys.cpus() {
        individual_cpus.push(cpu);
    }
    individual_cpus
}

fn color_severity_percentage(num: f32) -> Span<'static> {
    if num > 75.5 {
        return Span::styled(format!("{num:.2}%\n"), Style::default().fg(Color::LightRed));
    } else if num > 50.0 {
        return Span::styled(
            format!("{num:.2}%\n"),
            Style::default().fg(Color::LightYellow),
        );
    }
    return Span::styled(
        format!("{num:.2}%\n"),
        Style::default().fg(Color::LightGreen),
    );
}

#[allow(clippy::cast_precision_loss)]
pub fn create_stats_chunk<B: Backend>(f: &mut Frame<B>, sys: &System, chunk: Rect) -> Vec<Rect> {
    let outer_block = Block::default().title("Stats").borders(Borders::ALL);

    let sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunk);

    // Calculate the length to extend the constraints by
    let cpus = get_individual_cpus(sys);
    let num_cpus = cpus.len();
    let mut constraints = vec![Constraint::Length(1), Constraint::Length(1)];
    constraints.extend(vec![Constraint::Length(1); num_cpus]);

    let cpu_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(sub_chunks[0]);

    let global_usage: f32 = get_global_cpu_usage(sys);
    let prefix = Span::styled("Global CPU Usage: ".to_string(), Style::default());
    let percentage = color_severity_percentage(global_usage);
    let global_percentage = Spans::from(vec![prefix, percentage]);

    let global_percentage_paragraph = Paragraph::new(global_percentage)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);

    for (i, cpu) in get_individual_cpus(sys).iter().enumerate() {
        let prefix = Span::styled(format!("CPU {i}: "), Style::default());
        let percentage = color_severity_percentage(cpu.cpu_usage());
        let individual_cpu_percentage = Spans::from(vec![prefix, percentage]);

        let individual_cpu_paragraph = Paragraph::new(individual_cpu_percentage)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left);
        f.render_widget(individual_cpu_paragraph, cpu_chunk[i + 2]);
    }

    let used_mem = sys.used_memory() as f64;
    let total_mem = sys.total_memory() as f64;
    let global_numeric = format!(
        "Memory Used: {:.2} GB / {:.2} GB",
        used_mem / 1_000_000_000.0,
        total_mem / 1_000_000_000.0
    );
    let global_numeric_paragraph = Paragraph::new(global_numeric)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);

    f.render_widget(outer_block, chunk); // connect stat block to main chunk vec
    f.render_widget(global_percentage_paragraph, cpu_chunk[0]);
    f.render_widget(global_numeric_paragraph, sub_chunks[1]);

    sub_chunks
}
