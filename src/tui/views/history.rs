use ratatui::Frame;
use ratatui::layout::{Layout, Direction, Constraint, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Table, Row, Cell, List, ListItem};
use ratatui::style::{Style, Color, Modifier};
use ratatui::text::{Text, Span, Spans};
use std::collections::VecDeque;
use chrono::{DateTime, Local};

use crate::tui::state::app_state::{AppState, EventInfo};

/// Render the history view
pub fn render_history<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState) {
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
    let header = Paragraph::new(Text::styled(
        "PostDevAI Event History",
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL).title("Development History Tracking"));
    
    frame.render_widget(header, chunks[0]);
    
    // Split content area into two sections
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),  // Event list
            Constraint::Percentage(30),  // Event details/stats
        ].as_ref())
        .split(chunks[1]);
    
    // Render the event list
    render_event_list(frame, state, content_chunks[0]);
    
    // Render event summary statistics
    render_event_stats(frame, state, content_chunks[1]);
}

/// Render the list of events
fn render_event_list<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    // Create table headers
    let header_cells = ["Time", "Type", "Source", "Summary"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1);
    
    // Create rows from event data
    let rows = state.recent_events.iter().map(|event| {
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
            Cell::from("No events recorded yet. Terminal activity will appear here."),
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
        .column_spacing(1)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));
    
    frame.render_widget(events_table, area);
}

/// Render event statistics
fn render_event_stats<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    // Split the stats area vertically
    let stats_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Summary stats
            Constraint::Percentage(50),  // Type breakdown
        ].as_ref())
        .split(area);
    
    // Count events by type
    let mut event_counts = std::collections::HashMap::new();
    let mut source_counts = std::collections::HashMap::new();
    
    for event in &state.recent_events {
        *event_counts.entry(event.event_type.as_str()).or_insert(0) += 1;
        if let Some(source) = &event.source {
            *source_counts.entry(source.as_str()).or_insert(0) += 1;
        }
    }
    
    // Create summary stats content
    let summary_content = vec![
        Spans::from(vec![Span::styled("Event Statistics:", Style::default().add_modifier(Modifier::BOLD))]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled("Total Events: ", Style::default().add_modifier(Modifier::BOLD)), 
            Span::raw(format!("{}", state.recent_events.len()))]),
        Spans::from(vec![Span::styled("Newest Event: ", Style::default().add_modifier(Modifier::BOLD)), 
            Span::raw(match state.recent_events.front() {
                Some(event) => format!("{}", event.timestamp.format("%H:%M:%S")),
                None => "N/A".to_string(),
            })]),
        Spans::from(vec![Span::styled("Oldest Event: ", Style::default().add_modifier(Modifier::BOLD)), 
            Span::raw(match state.recent_events.back() {
                Some(event) => format!("{}", event.timestamp.format("%H:%M:%S")),
                None => "N/A".to_string(),
            })]),
    ];
    
    // Create type breakdown content
    let mut type_content = vec![
        Spans::from(vec![Span::styled("Event Types:", Style::default().add_modifier(Modifier::BOLD))]),
        Spans::from(vec![Span::raw("")]),
    ];
    
    // Add event counts by type
    for (event_type, count) in event_counts.iter() {
        type_content.push(Spans::from(vec![
            Span::styled(format!("{}: ", event_type), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", count)),
        ]));
    }
    
    // If there are no events, show placeholder content
    if state.recent_events.is_empty() {
        type_content.push(Spans::from(vec![Span::raw("No events recorded")]));
    }
    
    // Create paragraphs for each section
    let summary_stats = Paragraph::new(summary_content)
        .block(Block::default().borders(Borders::ALL).title("Summary"))
        .wrap(Wrap { trim: true });
    
    let type_breakdown = Paragraph::new(type_content)
        .block(Block::default().borders(Borders::ALL).title("Type Breakdown"))
        .wrap(Wrap { trim: true });
    
    // Render the paragraphs
    frame.render_widget(summary_stats, stats_chunks[0]);
    frame.render_widget(type_breakdown, stats_chunks[1]);
}

/// Truncate a string to a maximum length and add ellipsis if needed
pub fn truncate_summary(s: &str, max_len: usize) -> String {
    // We can just use the implementation from dashboard.rs
    super::dashboard::truncate_summary(s, max_len)
}