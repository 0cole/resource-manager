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
    // change the color of s based on which category the percentge is in
    // >75% -> Red
    // >50% -> Yellow
    // rest -> Green
    if num > 75.5 {
        return Span::styled(s, Style::default().fg(Color::LightRed));
    } else if num > 50.0 {
        return Span::styled(s, Style::default().fg(Color::LightYellow));
    }
    return Span::styled(s, Style::default().fg(Color::LightGreen));
}

#[allow(clippy::cast_possible_truncation)]
fn render_individual_cpu<B: Backend>(
    f: &mut Frame<B>,
    cpu: &Cpu,
    percent_chunk: Rect,
    bar_chunk: Rect,
) {
    // render percentage chunk (CPU #: XX.XX%)
    let prefix = Span::styled(format!("CPU {}: ", cpu.name()), Style::default());
    let percent = color_severity(format!("{:.2}%", cpu.cpu_usage()), cpu.cpu_usage());
    let formatted_percent = Spans::from(vec![prefix, percent]);
    let percent_paragraph = Paragraph::new(formatted_percent)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(percent_paragraph, percent_chunk);

    // render a percentage bar to provide another representation of the cpu usage
    let rounded = cpu.cpu_usage().round() as u32;
    let num_bars: usize = rounded.div_ceil(10) as usize;
    let mut severity_string = "|".repeat(num_bars);
    while severity_string.len() < 10 {
        severity_string.push(' ');
    }
    let severity_bar = color_severity(severity_string, cpu.cpu_usage());
    let severity_span = Spans::from(vec![
        Span::styled("[ ".to_string(), Style::default()),
        severity_bar,
        Span::styled(" ]".to_string(), Style::default()),
    ]);
    let bar_paragraph = Paragraph::new(severity_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(bar_paragraph, bar_chunk);
}

fn render_cpu_stats<B: Backend>(f: &mut Frame<B>, sys: &System, cpus: &[&Cpu], chunk: Rect) {
    let cpu_chunk = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(2), Constraint::Length(10)])
        .split(chunk);

    // render global cpu stats
    let global_usage: f32 = get_global_cpu_usage(sys);
    let prefix = Span::styled("Global CPU Usage: ".to_string(), Style::default());
    let percentage = color_severity(format!("{global_usage:.2}%"), global_usage);
    let global_percentage = Spans::from(vec![prefix, percentage]);
    let global_percentage_paragraph = Paragraph::new(global_percentage)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(global_percentage_paragraph, cpu_chunk[0]);

    // render individual cpu stats
    let num_cpus = cpus.len();
    let constraints = vec![Constraint::Length(1); num_cpus];

    let individual_cpu_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        //                         CPU #: XX.XX%    [|||       ]
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(cpu_chunk[1]);

    let individual_cpu_percents_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(constraints.clone())
        .split(individual_cpu_chunks[0]);

    let individual_cpu_bars_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(constraints)
        .split(individual_cpu_chunks[1]);

    // add cpu percentages
    for (i, cpu) in get_individual_cpus(sys).iter().enumerate() {
        render_individual_cpu(
            f,
            cpu,
            individual_cpu_percents_chunks[i],
            individual_cpu_bars_chunks[i],
        );
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
#[allow(clippy::too_many_lines)] // for now
fn render_mem_stats<B: Backend>(f: &mut Frame<B>, sys: &System, chunk: Rect) {
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let avail_mem = sys.available_memory();
    let free_mem = sys.free_memory();

    let mem_sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        //                         Memory: XX.XX%    [|||       ]
        //                         Total Memory:     XX.XX GB
        //                         ...
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunk);

    let mem_label_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // percentage mem
            Constraint::Length(1), // spacing
            Constraint::Length(1), // total mem
            Constraint::Length(1), // avail mem
            Constraint::Length(1), // used mem
            Constraint::Length(1), // free mem
        ])
        .split(mem_sub_chunks[0]);

    let mem_num_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // percentage mem
            Constraint::Length(1), // spacing
            Constraint::Length(1), // total mem
            Constraint::Length(1), // avail mem
            Constraint::Length(1), // used mem
            Constraint::Length(1), // free mem
        ])
        .split(mem_sub_chunks[1]);

    // render global mem percentage
    let prefix = Span::styled("Memory: ".to_string(), Style::default());
    let percent = ((used_mem as f64 / total_mem as f64) * 100.0) as f32;
    let percent_color = color_severity(format!("{percent:.2}%"), percent);
    let formatted_percent = Spans::from(vec![prefix, percent_color]);
    let percent_paragraph = Paragraph::new(formatted_percent)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(percent_paragraph, mem_label_chunks[0]);

    // render global mem bar
    let num_bars: usize = (percent / 10.0) as usize;
    let mut severity_string = "|".repeat(num_bars);
    while severity_string.len() < 10 {
        severity_string.push(' ');
    }
    let severity_bar = color_severity(severity_string, percent);
    let severity_span = Spans::from(vec![
        Span::styled("[ ".to_string(), Style::default()),
        severity_bar,
        Span::styled(" ]".to_string(), Style::default()),
    ]);
    let bar_paragraph = Paragraph::new(severity_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(bar_paragraph, mem_num_chunks[0]);

    // render total memory
    let total_mem_label_span = Span::styled("Total Memory: ".to_string(), Style::default());
    let total_mem_num_span = Span::styled(
        format!("{:.2} GB", (total_mem as f64) / 1_000_000_000.0),
        Style::default(),
    );
    let total_mem_label_paragraph = Paragraph::new(total_mem_label_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    let total_mem_num_paragraph = Paragraph::new(total_mem_num_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(total_mem_label_paragraph, mem_label_chunks[2]);
    f.render_widget(total_mem_num_paragraph, mem_num_chunks[2]);

    // render available memory
    let avail_mem_label_span = Span::styled("Avail Memory: ".to_string(), Style::default());
    let avail_mem_num_span = Span::styled(
        format!("{:.2} GB", (avail_mem as f64) / 1_000_000_000.0),
        Style::default(),
    );
    let avail_mem_label_paragraph = Paragraph::new(avail_mem_label_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    let avail_mem_num_paragraph = Paragraph::new(avail_mem_num_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(avail_mem_label_paragraph, mem_label_chunks[3]);
    f.render_widget(avail_mem_num_paragraph, mem_num_chunks[3]);

    // render used memory
    let used_mem_label_span = Span::styled("Used Memory: ".to_string(), Style::default());
    let used_mem_num_span = Span::styled(
        format!("{:.2} GB", (used_mem as f64) / 1_000_000_000.0),
        Style::default(),
    );
    let used_mem_label_paragraph = Paragraph::new(used_mem_label_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    let used_mem_num_paragraph = Paragraph::new(used_mem_num_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(used_mem_label_paragraph, mem_label_chunks[4]);
    f.render_widget(used_mem_num_paragraph, mem_num_chunks[4]);

    // render free memeory
    let free_mem_label_span = Span::styled("Free Memory: ".to_string(), Style::default());
    let free_mem_num_span = Span::styled(
        format!("{:.2} MB", (free_mem as f64) / 1_000_000.0),
        Style::default(),
    );
    let free_mem_label_paragraph = Paragraph::new(free_mem_label_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    let free_mem_num_paragraph = Paragraph::new(free_mem_num_span)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(free_mem_label_paragraph, mem_label_chunks[5]);
    f.render_widget(free_mem_num_paragraph, mem_num_chunks[5]);
}

pub fn create_stats_chunk<B: Backend>(f: &mut Frame<B>, sys: &System, chunk: Rect) -> Vec<Rect> {
    // draw outer block for stats
    let outer_block = Block::default().title("Stats").borders(Borders::ALL);
    f.render_widget(outer_block, chunk);

    // splits the stats chunk into three chunks
    // 1. CPU
    // 2. Memory
    // 3. Something else
    let sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(25), // cpu
                Constraint::Percentage(25), // mem
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(chunk);

    // render cpu stats
    let cpus = get_individual_cpus(sys);
    render_cpu_stats(f, sys, &cpus, sub_chunks[0]);

    // render mem stats
    render_mem_stats(f, sys, sub_chunks[1]);

    sub_chunks
}
