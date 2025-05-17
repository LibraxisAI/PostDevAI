use ratatui::Frame;
use ratatui::layout::{Layout, Direction, Constraint, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Gauge, Sparkline, Table, Row, Cell};
use ratatui::style::{Style, Color, Modifier};
use ratatui::text::{Text, Span, Spans};
use ratatui::symbols;
use chrono::Local;

use crate::tui::state::app_state::AppState;

const GB: u64 = 1024 * 1024 * 1024;

/// Render the dashboard view
pub fn render_dashboard<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState) {
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
        ].as_ref())
        .split(frame.size());
    
    // Render header
    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let header_text = vec![
        Spans::from(vec![
            Span::styled("PostDevAI Distributed System Dashboard", 
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(vec![
            Span::raw(format!("Uptime: {}  |  Node: {}  |  Current Time: {}", 
                format_duration(&state.uptime),
                state.system_state.hostname,
                current_time,
            )),
        ]),
    ];
    
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("PostDevAI"))
        .alignment(ratatui::layout::Alignment::Center);
    
    frame.render_widget(header, chunks[0]);
    
    // Create the main dashboard layout
    let dashboard_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),    // System status gauges
            Constraint::Length(9),    // Node connections and model info
            Constraint::Min(5),       // Recent events
            Constraint::Length(5),    // Quick stats
        ].as_ref())
        .split(chunks[1]);
    
    // Render system status gauges
    render_system_gauges(frame, state, dashboard_chunks[0]);
    
    // Render node connections and model info
    render_nodes_and_models(frame, state, dashboard_chunks[1]);
    
    // Render recent events
    render_recent_events(frame, state, dashboard_chunks[2]);
    
    // Render quick stats
    render_quick_stats(frame, state, dashboard_chunks[3]);
}

/// Render system resource gauges
fn render_system_gauges<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    // Split the area into three columns for different gauges
    let gauge_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),  // CPU usage
            Constraint::Percentage(33),  // Memory usage
            Constraint::Percentage(34),  // RAM-Lake usage
        ].as_ref())
        .split(area);
    
    // CPU usage gauge
    let cpu_usage = state.system_state.cpu_usage as u16;
    let cpu_color = match cpu_usage {
        0..=50 => Color::Green,
        51..=80 => Color::Yellow,
        _ => Color::Red,
    };
    
    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("CPU Usage"))
        .gauge_style(Style::default().fg(cpu_color).bg(Color::Black))
        .percent(cpu_usage)
        .label(format!("{}%", cpu_usage));
        
    frame.render_widget(cpu_gauge, gauge_chunks[0]);
    
    // Memory usage gauge
    let memory_used_pct = if state.system_state.memory_usage.total > 0 {
        (state.system_state.memory_usage.used as f64 / state.system_state.memory_usage.total as f64 * 100.0) as u16
    } else {
        0
    };
    
    let memory_color = match memory_used_pct {
        0..=50 => Color::Green,
        51..=80 => Color::Yellow,
        _ => Color::Red,
    };
    
    let memory_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("System Memory"))
        .gauge_style(Style::default().fg(memory_color).bg(Color::Black))
        .percent(memory_used_pct)
        .label(format!("{}/{} GB ({}%)", 
            state.system_state.memory_usage.used / GB, 
            state.system_state.memory_usage.total / GB,
            memory_used_pct));
            
    frame.render_widget(memory_gauge, gauge_chunks[1]);
    
    // RAM-Lake usage gauge
    let ramlake_used_pct = if state.ramlake_metrics.total_size > 0 {
        (state.ramlake_metrics.used_size as f64 / state.ramlake_metrics.total_size as f64 * 100.0) as u16
    } else {
        0
    };
    
    let ramlake_color = match ramlake_used_pct {
        0..=50 => Color::Green,
        51..=80 => Color::Yellow,
        _ => Color::Red,
    };
    
    let ramlake_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("RAM-Lake"))
        .gauge_style(Style::default().fg(ramlake_color).bg(Color::Black))
        .percent(ramlake_used_pct)
        .label(format!("{}/{} GB ({}%)", 
            state.ramlake_metrics.used_size / GB, 
            state.ramlake_metrics.total_size / GB,
            ramlake_used_pct));
            
    frame.render_widget(ramlake_gauge, gauge_chunks[2]);
}

/// Render node connections and model info
fn render_nodes_and_models<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    // Split the area into two columns
    let column_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Node connections
            Constraint::Percentage(50),  // Model information
        ].as_ref())
        .split(area);
    
    // Create node connections table
    let node_header_cells = ["Node Type", "Hostname", "Status", "Last Heartbeat"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let node_header = Row::new(node_header_cells)
        .style(Style::default())
        .height(1);
    
    // Create rows from node data
    let node_rows = state.node_connections.iter().map(|node| {
        let status_color = match node.status.as_str() {
            "connected" => Color::Green,
            "disconnected" => Color::Red,
            "connecting" => Color::Yellow,
            _ => Color::White,
        };
        
        // Format the last heartbeat time
        let heartbeat_time = node.last_heartbeat.format("%H:%M:%S").to_string();
        
        Row::new(vec![
            Cell::from(node.node_type.as_str()),
            Cell::from(node.hostname.as_str()),
            Cell::from(node.status.as_str()).style(Style::default().fg(status_color)),
            Cell::from(heartbeat_time),
        ])
    });
    
    let node_table = Table::new(node_rows)
        .header(node_header)
        .block(Block::default().borders(Borders::ALL).title("Node Connections"))
        .widths(&[
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .column_spacing(1);
    
    frame.render_widget(node_table, column_chunks[0]);
    
    // Create model information table
    let model_header_cells = ["Model", "Status", "Memory (GB)"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let model_header = Row::new(model_header_cells)
        .style(Style::default())
        .height(1);
    
    // Create rows from model data
    let model_rows = state.loaded_models.iter().map(|model| {
        let status_color = match model.status.as_str() {
            "loaded" => Color::Green,
            "unloaded" => Color::Gray,
            "loading" => Color::Yellow,
            "error" => Color::Red,
            _ => Color::White,
        };
        
        Row::new(vec![
            Cell::from(model.name.as_str()),
            Cell::from(model.status.as_str()).style(Style::default().fg(status_color)),
            Cell::from(format!("{:.2}", model.memory_gb)),
        ])
    });
    
    // If no models, show placeholder row
    let model_rows = if state.loaded_models.is_empty() {
        vec![Row::new(vec![
            Cell::from("No models loaded"),
            Cell::from("-"),
            Cell::from("-"),
        ])]
    } else {
        model_rows.collect()
    };
    
    let model_table = Table::new(model_rows)
        .header(model_header)
        .block(Block::default().borders(Borders::ALL).title("MLX Models"))
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .column_spacing(1);
    
    frame.render_widget(model_table, column_chunks[1]);
}

/// Render recent events
fn render_recent_events<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    // Create table headers
    let header_cells = ["Time", "Type", "Source", "Summary"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1);
    
    // Create rows from event data (limited to most recent 5)
    let rows = state.recent_events.iter().take(5).map(|event| {
        let event_color = match event.severity.as_deref() {
            Some("Error") => Color::Red,
            Some("Warning") => Color::Yellow,
            Some("Info") => Color::Green,
            _ => Color::White,
        };
        
        // Format the time as HH:MM:SS
        let time = event.timestamp.format("%H:%M:%S").to_string();
        
        Row::new(vec![
            Cell::from(time),
            Cell::from(event.event_type.as_str()).style(Style::default().fg(event_color)),
            Cell::from(event.source.as_deref().unwrap_or("-")),
            Cell::from(truncate_summary(&event.summary, 50)),
        ])
    });
    
    // If no events, show placeholder row
    let rows = if state.recent_events.is_empty() {
        vec![Row::new(vec![
            Cell::from("-"),
            Cell::from("-"),
            Cell::from("-"),
            Cell::from("No events recorded yet. Activities will appear here."),
        ])]
    } else {
        rows.collect()
    };
    
    let events_table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Recent Events"))
        .widths(&[
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(55),
        ])
        .column_spacing(1);
    
    frame.render_widget(events_table, area);
}

/// Render quick stats at the bottom of the dashboard
fn render_quick_stats<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    // Split the area into three columns
    let stat_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),  // RAM-Lake stats
            Constraint::Percentage(33),  // System stats
            Constraint::Percentage(34),  // Help text
        ].as_ref())
        .split(area);
    
    // RAM-Lake quick stats
    let ramlake_stats = vec![
        Spans::from(vec![
            Span::styled("Indexed Files: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", state.ramlake_metrics.indexed_files)),
        ]),
        Spans::from(vec![
            Span::styled("Vector Entries: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", state.ramlake_metrics.vector_entries)),
        ]),
        Spans::from(vec![
            Span::styled("History Events: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", state.ramlake_metrics.history_events)),
        ]),
    ];
    
    let ramlake_stats_widget = Paragraph::new(ramlake_stats)
        .block(Block::default().borders(Borders::ALL).title("RAM-Lake Stats"))
        .wrap(Wrap { trim: true });
    
    frame.render_widget(ramlake_stats_widget, stat_chunks[0]);
    
    // System stats
    let system_stats = vec![
        Spans::from(vec![
            Span::styled("Node Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:?}", state.system_state.node_type)),
        ]),
        Spans::from(vec![
            Span::styled("Hostname: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&state.system_state.hostname),
        ]),
        Spans::from(vec![
            Span::styled("System Uptime: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format_duration(&state.system_state.uptime)),
        ]),
    ];
    
    let system_stats_widget = Paragraph::new(system_stats)
        .block(Block::default().borders(Borders::ALL).title("System Info"))
        .wrap(Wrap { trim: true });
    
    frame.render_widget(system_stats_widget, stat_chunks[1]);
    
    // Help text
    let help_text = vec![
        Spans::from(vec![Span::styled("Press F1 or ? for help", Style::default().fg(Color::Cyan))]),
        Spans::from(vec![Span::raw("F2-F5: Switch views")]),
        Spans::from(vec![Span::raw("q: Quit")]),
    ];
    
    let help_widget = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Navigation"))
        .wrap(Wrap { trim: true });
    
    frame.render_widget(help_widget, stat_chunks[2]);
}

/// Format a duration to a human readable string (e.g. "2h 15m 30s")
pub fn format_duration(duration: &std::time::Duration) -> String {
    let seconds = duration.as_secs();
    
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Truncate a string to a maximum length and add ellipsis if needed
pub fn truncate_summary(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[0..max_len - 3])
    }
}