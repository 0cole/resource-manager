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

fn color_severity(s: String, num: f32) -> Span<'static> {
    if num > 75.5 {
        return Span::styled(s, Style::default().fg(Color::LightRed));
    } else if num > 50.0 {
        return Span::styled(s, Style::default().fg(Color::LightYellow));
    }
    return Span::styled(s, Style::default().fg(Color::LightGreen));
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
    let constraints = vec![Constraint::Length(1); num_cpus];
    let constraints_clone = constraints.clone();

    let cpu_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(2), Constraint::Length(10)])
        .split(sub_chunks[0]);

    let individual_cpu_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(cpu_chunk[1]);

    let individual_cpu_percents_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(constraints)
        .split(individual_cpu_chunks[0]);

    let individual_cpu_bars_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(constraints_clone)
        .split(individual_cpu_chunks[1]);

    let global_usage: f32 = get_global_cpu_usage(sys);
    let prefix = Span::styled("Global CPU Usage: ".to_string(), Style::default());
    let percentage = color_severity(format!("{global_usage:.2}%"), global_usage);
    let global_percentage = Spans::from(vec![prefix, percentage]);

    let global_percentage_paragraph = Paragraph::new(global_percentage)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);

    // add cpu percentages
    for (i, cpu) in get_individual_cpus(sys).iter().enumerate() {
        let prefix = Span::styled(format!("CPU {i}: "), Style::default());
        let percentage = color_severity(format!("{:.2}%", cpu.cpu_usage()), cpu.cpu_usage());
        let individual_cpu_percentage = Spans::from(vec![prefix, percentage]);

        let rounded = cpu.cpu_usage().round() as u32;
        let num_bars: usize = rounded.div_ceil(10) as usize;
        let severity_bar = color_severity("|".repeat(num_bars), cpu.cpu_usage());
        let severity_span = Spans::from(vec![
            Span::styled("[ ".to_string(), Style::default()),
            severity_bar,
            Span::styled(" ]".to_string(), Style::default()),
        ]);

        let individual_percentage_paragraph = Paragraph::new(individual_cpu_percentage)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left);
        let individual_bar_paragraph = Paragraph::new(severity_span)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left);

        f.render_widget(
            individual_percentage_paragraph,
            individual_cpu_percents_chunks[i],
        );
        f.render_widget(individual_bar_paragraph, individual_cpu_bars_chunks[i]);
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
