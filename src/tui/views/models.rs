use ratatui::Frame;
use ratatui::layout::{Layout, Direction, Constraint, Rect};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Table, Row, Cell, List, ListItem};
use ratatui::style::{Style, Color, Modifier};
use ratatui::text::{Text, Span, Spans};
use std::time::{Instant, Duration};

use crate::tui::state::app_state::{AppState, ModelInfo};

/// Render the models view
pub fn render_models<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState) {
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
        "PostDevAI MLX Models",
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL).title("Models Dashboard"));
    
    frame.render_widget(header, chunks[0]);
    
    // Split content area into two parts horizontally
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),  // Model list
            Constraint::Percentage(40),  // Model details
        ].as_ref())
        .split(chunks[1]);
    
    // Render model list
    render_model_list(frame, state, content_chunks[0]);
    
    // Render model details
    render_model_details(frame, state, content_chunks[1]);
}

/// Render the list of models
fn render_model_list<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    // Create table headers
    let header_cells = ["Name", "Type", "Status", "Memory (GB)", "Priority"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1);
        
    // Create rows from model data
    let rows = state.loaded_models.iter().map(|model| {
        let status_color = match model.status.as_str() {
            "loaded" => Color::Green,
            "unloaded" => Color::Gray,
            "loading" => Color::Yellow,
            "error" => Color::Red,
            _ => Color::White,
        };
        
        let priority_color = match model.priority {
            p if p >= 9 => Color::Green,
            p if p >= 5 => Color::Yellow,
            _ => Color::Gray,
        };
        
        Row::new(vec![
            Cell::from(model.name.as_str()),
            Cell::from(model.model_type.as_str()),
            Cell::from(model.status.as_str()).style(Style::default().fg(status_color)),
            Cell::from(format!("{:.2}", model.memory_gb)),
            Cell::from(model.priority.to_string()).style(Style::default().fg(priority_color)),
        ])
    });
    
    let models_table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Available Models"))
        .widths(&[
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(15), 
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .column_spacing(1)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));
    
    frame.render_widget(models_table, area);
}

/// Render detailed information about the selected model
fn render_model_details<B: ratatui::backend::Backend>(frame: &mut Frame<B>, state: &AppState, area: Rect) {
    let content = if let Some(model) = state.loaded_models.first() {
        // Format the last used time
        let last_used_str = if let Some(last_used) = model.last_used {
            let last_used_ago = Instant::now().duration_since(last_used);
            format_duration(&last_used_ago)
        } else {
            "Unknown".to_string()
        };
        
        vec![
            Spans::from(vec![Span::styled(format!("Model: {}", model.name), 
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![
                Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(model.model_type.as_str()),
            ]),
            Spans::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(model.status.as_str()),
            ]),
            Spans::from(vec![
                Span::styled("Memory Usage: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:.2} GB", model.memory_gb)),
            ]),
            Spans::from(vec![
                Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(model.priority.to_string()),
            ]),
            Spans::from(vec![
                Span::styled("Last Used: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(last_used_str),
            ]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::styled("Controls:", Style::default().add_modifier(Modifier::BOLD))]),
            Spans::from(vec![Span::raw("M - Toggle load/unload model")]),
            Spans::from(vec![Span::raw("↑/↓ - Select model")]),
            Spans::from(vec![Span::raw("R - Refresh model status")]),
        ]
    } else {
        vec![
            Spans::from(vec![Span::styled("No Model Selected", 
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw("No models are currently available.")]),
            Spans::from(vec![Span::raw("Check the MLX configuration and try again.")]),
        ]
    };
    
    let model_details = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Model Details"))
        .wrap(Wrap { trim: true });
    
    frame.render_widget(model_details, area);
}

/// Format a duration to a human readable string
fn format_duration(duration: &Duration) -> String {
    let seconds = duration.as_secs();
    
    if seconds < 60 {
        format!("{} seconds ago", seconds)
    } else if seconds < 3600 {
        format!("{} minutes ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours ago", seconds / 3600)
    } else {
        format!("{} days ago", seconds / 86400)
    }
}