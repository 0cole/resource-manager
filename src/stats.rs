use sysinfo::{Cpu, Disks, System};
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

fn render_label_value<B: Backend>(
    f: &mut Frame<B>,
    label: &str,
    value: String,
    label_chunk: Rect,
    value_chunk: Rect,
) {
    let label_paragraph = Paragraph::new(Span::styled(label, Style::default()))
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    let value_paragraph = Paragraph::new(Span::styled(value, Style::default()))
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Right);
    f.render_widget(label_paragraph, label_chunk);
    f.render_widget(value_paragraph, value_chunk);
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
        .alignment(Alignment::Right);
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
        .alignment(Alignment::Right);
    f.render_widget(bar_paragraph, mem_num_chunks[0]);

    // render total memory
    render_label_value(
        f,
        "Total Memory: ",
        format!("{:.2} GB", (total_mem as f64) / 1_000_000_000.0),
        mem_label_chunks[2],
        mem_num_chunks[2],
    );
    // render available memory
    render_label_value(
        f,
        "Avail Memory: ",
        format!("{:.2} GB", (avail_mem as f64) / 1_000_000_000.0),
        mem_label_chunks[3],
        mem_num_chunks[3],
    );
    // render used memory
    render_label_value(
        f,
        "Used Memory: ",
        format!("{:.2} GB", (used_mem as f64) / 1_000_000_000.0),
        mem_label_chunks[4],
        mem_num_chunks[4],
    );
    // render free memeory
    render_label_value(
        f,
        "Free Memory: ",
        format!("{:.2} MB", (free_mem as f64) / 1_000_000.0),
        mem_label_chunks[5],
        mem_num_chunks[5],
    );
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn render_swp_stats<B: Backend>(f: &mut Frame<B>, sys: &System, chunk: Rect) {
    let total_swp = sys.total_swap();
    let used_swp = sys.used_swap();
    let free_swp = sys.free_swap();

    let swp_sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        //                           swp: XX.XX%    [|||       ]
        //                            Total swp:    XX.XX GB
        //                         ...
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunk);

    let swp_label_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // percentage swp
            Constraint::Length(1), // spacing
            Constraint::Length(1), // total swp
            Constraint::Length(1), // used swp
            Constraint::Length(1), // free swp
        ])
        .split(swp_sub_chunks[0]);

    let swp_num_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // percentage swp
            Constraint::Length(1), // spacing
            Constraint::Length(1), // total swp
            Constraint::Length(1), // used swp
            Constraint::Length(1), // free swp
        ])
        .split(swp_sub_chunks[1]);

    // render swp percentage
    let prefix = Span::styled("swp: ".to_string(), Style::default());
    let percent = ((used_swp as f64 / total_swp as f64) * 100.0) as f32;
    let percent_color = color_severity(format!("{percent:.2}%"), percent);
    let formatted_percent = Spans::from(vec![prefix, percent_color]);
    let percent_paragraph = Paragraph::new(formatted_percent)
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Left);
    f.render_widget(percent_paragraph, swp_label_chunks[0]);

    // render swp bar
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
        .alignment(Alignment::Right);
    f.render_widget(bar_paragraph, swp_num_chunks[0]);

    // render total swp
    render_label_value(
        f,
        "Total swp: ",
        format!("{:.2} GB", (total_swp as f64) / 1_000_000_000.0),
        swp_label_chunks[2],
        swp_num_chunks[2],
    );
    // render used swp
    render_label_value(
        f,
        "Used swp: ",
        format!("{:.2} GB", (used_swp as f64) / 1_000_000_000.0),
        swp_label_chunks[3],
        swp_num_chunks[3],
    );
    // render free swp
    render_label_value(
        f,
        "Free swp: ",
        format!("{:.2} GB", (free_swp as f64) / 1_000_000_000.0),
        swp_label_chunks[4],
        swp_num_chunks[4],
    );
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn render_disk_stats<B: Backend>(f: &mut Frame<B>, disks: &Disks, chunk: Rect) {
    let num_disks = disks.len();

    let constraints =
        vec![Constraint::Percentage(100 / (u16::try_from(num_disks).unwrap())); num_disks];

    let disk_sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(chunk);

    for (i, disk) in disks.list().iter().enumerate() {
        let total_space = disk.total_space();
        let available_space = disk.available_space();
        let used_space = total_space - available_space;
        let percentage_used = ((used_space as f64 / total_space as f64) * 100.0) as f32;
        let name = disk.name();
        let kind = disk.kind();
        let fs = disk.file_system();
        let mount_point = disk.mount_point();

        let title = format!("Disk {i}");
        let outer_block = Block::default().title(title).borders(Borders::ALL);
        f.render_widget(outer_block, disk_sub_chunks[i]);

        let disk_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(disk_sub_chunks[i]);

        let disk_label_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(disk_chunk[0]);

        let disk_value_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(disk_chunk[1]);

        // render mount point
        render_label_value(
            f,
            "Mount Point:",
            mount_point.to_str().unwrap().to_string(),
            disk_label_chunks[0],
            disk_value_chunks[0],
        );

        // render disk name
        render_label_value(
            f,
            "Name: ",
            name.to_str().unwrap().to_string(),
            disk_label_chunks[1],
            disk_value_chunks[1],
        );

        // render disk usage
        let percent_label_paragraph = Paragraph::new("Usage: ")
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left);
        f.render_widget(percent_label_paragraph, disk_label_chunks[2]);
        let percent_color = color_severity(format!("{percentage_used:.2}%"), percentage_used);
        let percent_paragraph = Paragraph::new(percent_color)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Right);
        f.render_widget(percent_paragraph, disk_value_chunks[2]);

        // render disk fs
        render_label_value(
            f,
            "Filesystem: ",
            fs.to_str().unwrap().to_string(),
            disk_label_chunks[3],
            disk_value_chunks[3],
        );

        // render disk kind
        render_label_value(
            f,
            "Kind: ",
            kind.to_string(),
            disk_label_chunks[4],
            disk_value_chunks[4],
        );
    }
}

fn render_system_stats<B: Backend>(f: &mut Frame<B>, chunk: Rect) {
    let host_name = System::host_name();
    let version = System::os_version();
    let uptime = System::uptime();
    let arch = System::cpu_arch();
    let os = System::name();

    let padding_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(1)
        .constraints([Constraint::Min(1)].as_ref())
        .split(chunk);

    let outer_block = Block::default().title("System").borders(Borders::ALL);
    f.render_widget(outer_block, padding_chunk[0]);

    let system_sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)].as_ref())
        .split(padding_chunk[0]);

    let system_label_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(system_sub_chunks[0]);

    let system_value_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(system_sub_chunks[1]);

    // render host name
    render_label_value(
        f,
        "Hostname: ",
        host_name.unwrap(),
        system_label_chunks[0],
        system_value_chunks[0],
    );

    // render version
    render_label_value(
        f,
        "Version: ",
        version.unwrap(),
        system_label_chunks[1],
        system_value_chunks[1],
    );

    // render uptime
    render_label_value(
        f,
        "Up-time: ",
        uptime.to_string(),
        system_label_chunks[2],
        system_value_chunks[2],
    );

    // render architecture
    render_label_value(
        f,
        "CPU Arch: ",
        arch,
        system_label_chunks[3],
        system_value_chunks[3],
    );

    // render operating system
    render_label_value(
        f,
        "OS: ",
        os.unwrap(),
        system_label_chunks[4],
        system_value_chunks[4],
    );
}

pub fn create_stats_chunk<B: Backend>(
    f: &mut Frame<B>,
    sys: &System,
    disks: &Disks,
    chunk: Rect,
) -> Vec<Rect> {
    // draw outer block for stats
    let outer_block = Block::default().title("Stats").borders(Borders::ALL);
    f.render_widget(outer_block, chunk);

    let cpus = get_individual_cpus(sys);
    let num_cpus = u16::try_from(sys.cpus().len()).unwrap();

    // splits the stats chunk into four chunks
    // 1. CPU
    // 2. Memory
    // 3. swp
    // 4. disk usage
    // 5. system metadata
    let sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .vertical_margin(1)
        .constraints(
            [
                Constraint::Percentage(7 + (num_cpus * 2)), // cpu
                Constraint::Percentage(16),                 // mem
                Constraint::Percentage(14),                 // swp
                Constraint::Percentage(31),                 // TBD
                Constraint::Percentage(20),                 // metadata
            ]
            .as_ref(),
        )
        .split(chunk);

    // render cpu stats
    render_cpu_stats(f, sys, &cpus, sub_chunks[0]);

    // render mem stats
    render_mem_stats(f, sys, sub_chunks[1]);

    // render swp stats
    render_swp_stats(f, sys, sub_chunks[2]);

    // render disk stats
    render_disk_stats(f, disks, sub_chunks[3]);

    // render sys metadata stats
    render_system_stats(f, sub_chunks[4]);

    sub_chunks
}
