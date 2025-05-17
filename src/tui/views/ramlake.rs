use ratatui::Frame;
use ratatui::layout::{Layout, Direction, Constraint, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Gauge, BarChart, Tabs, Chart, Dataset, GraphType, Axis};
use ratatui::style::{Style, Color, Modifier};
use ratatui::text::{Text, Span, Spans};
use ratatui::symbols;

use crate::tui::state::app_state::AppState;

const GB: u64 = 1024 * 1024 * 1024;

/// Render the RAM-Lake view
pub fn render_ramlake<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState) {
    // Create main layout with header and content
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
        "PostDevAI RAM-Lake Storage System",
        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL).title("RAM-Lake Dashboard"));
    
    frame.render_widget(header, chunks[0]);
    
    // Split content area into two main sections: metrics and charts
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),  // Usage metrics
            Constraint::Percentage(60),  // Detail charts
        ].as_ref())
        .split(chunks[1]);
        
    // Render the usage metrics section
    render_usage_metrics(frame, state, content_chunks[0]);
    
    // Render store details and charts
    render_store_details(frame, state, content_chunks[1]);
}

/// Render RAM-Lake usage metrics
fn render_usage_metrics<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    // Split the metrics area into multiple sections
    let metrics_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Overall usage gauge
            Constraint::Min(0),     // Store usage breakdown
        ].as_ref())
        .split(area);
        
    // Render the overall usage gauge
    render_overall_gauge(frame, state, metrics_chunks[0]);
    
    // Render the stores usage breakdown
    render_stores_breakdown(frame, state, metrics_chunks[1]);
}

/// Render the overall usage gauge
fn render_overall_gauge<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    let metrics = &state.ramlake_metrics;
    
    // Calculate the usage percentage
    let percentage = if metrics.total_size > 0 {
        (metrics.used_size as f64 / metrics.total_size as f64 * 100.0) as u16
    } else {
        0
    };
    
    // Determine gauge color based on usage
    let gauge_color = match percentage {
        0..=50 => Color::Green,
        51..=75 => Color::Yellow,
        _ => Color::Red,
    };
    
    // Create the gauge
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("RAM-Lake Usage"))
        .gauge_style(Style::default().fg(gauge_color).bg(Color::Black))
        .percent(percentage)
        .label(format!("{}/{} GB ({:.1}%)", 
            metrics.used_size / GB, 
            metrics.total_size / GB,
            percentage));
        
    frame.render_widget(gauge, area);
}

/// Render the stores usage breakdown
fn render_stores_breakdown<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    let metrics = &state.ramlake_metrics;
    
    // Split the area into two columns
    let column_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Left column
            Constraint::Percentage(50),  // Right column
        ].as_ref())
        .split(area);
        
    // Create store size metrics for the left column
    let store_metrics = vec![
        Spans::from(vec![
            Span::styled("Vector Store: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.2} GB", metrics.vector_store_size as f64 / GB as f64)),
            Span::styled(format!(" ({:.1}%)", 
                if metrics.total_size > 0 { metrics.vector_store_size as f64 / metrics.total_size as f64 * 100.0 } else { 0.0 }
            ), Style::default().fg(Color::Blue)),
        ]),
        Spans::from(vec![
            Span::styled("Code Store: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.2} GB", metrics.code_store_size as f64 / GB as f64)),
            Span::styled(format!(" ({:.1}%)", 
                if metrics.total_size > 0 { metrics.code_store_size as f64 / metrics.total_size as f64 * 100.0 } else { 0.0 }
            ), Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![
            Span::styled("History Store: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.2} GB", metrics.history_store_size as f64 / GB as f64)),
            Span::styled(format!(" ({:.1}%)", 
                if metrics.total_size > 0 { metrics.history_store_size as f64 / metrics.total_size as f64 * 100.0 } else { 0.0 }
            ), Style::default().fg(Color::Yellow)),
        ]),
        Spans::from(vec![
            Span::styled("Metadata Store: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.2} GB", metrics.metadata_store_size as f64 / GB as f64)),
            Span::styled(format!(" ({:.1}%)", 
                if metrics.total_size > 0 { metrics.metadata_store_size as f64 / metrics.total_size as f64 * 100.0 } else { 0.0 }
            ), Style::default().fg(Color::Magenta)),
        ]),
    ];
    
    // Create counters for the right column
    let counter_metrics = vec![
        Spans::from(vec![
            Span::styled("Total Files: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", metrics.indexed_files)),
        ]),
        Spans::from(vec![
            Span::styled("Vector Entries: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", metrics.vector_entries)),
        ]),
        Spans::from(vec![
            Span::styled("History Events: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", metrics.history_events)),
        ]),
        Spans::from(vec![
            Span::styled("Free Space: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.2} GB", (metrics.total_size - metrics.used_size) as f64 / GB as f64)),
        ]),
    ];
    
    // Create paragraphs for each column
    let store_metrics_paragraph = Paragraph::new(store_metrics)
        .block(Block::default().borders(Borders::ALL).title("Store Sizes"))
        .alignment(ratatui::layout::Alignment::Left)
        .wrap(Wrap { trim: true });
        
    let counter_metrics_paragraph = Paragraph::new(counter_metrics)
        .block(Block::default().borders(Borders::ALL).title("Entry Counts"))
        .alignment(ratatui::layout::Alignment::Left)
        .wrap(Wrap { trim: true });
        
    // Render the paragraphs
    frame.render_widget(store_metrics_paragraph, column_chunks[0]);
    frame.render_widget(counter_metrics_paragraph, column_chunks[1]);
}

/// Render the store details and charts
fn render_store_details<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    let metrics = &state.ramlake_metrics;
    
    // Create data for the barchart
    let data = [
        ("Vector", (metrics.vector_store_size / GB) as u64),
        ("Code", (metrics.code_store_size / GB) as u64),
        ("History", (metrics.history_store_size / GB) as u64),
        ("Metadata", (metrics.metadata_store_size / GB) as u64),
    ];
    
    // Find the maximum value for scaling
    let max_value = data.iter()
        .map(|(_, v)| *v)
        .max()
        .unwrap_or(1);
    
    // Create the bar chart
    let barchart = BarChart::default()
        .block(Block::default().borders(Borders::ALL).title("Store Size Distribution (GB)"))
        .data(&data)
        .bar_width(10)
        .bar_gap(6)
        .bar_style(Style::default().fg(Color::Blue))
        .value_style(Style::default().fg(Color::Black).bg(Color::Blue))
        .max(max_value);
        
    // Render the bar chart
    frame.render_widget(barchart, area);
}