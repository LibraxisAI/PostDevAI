use ratatui::Frame;
use ratatui::layout::{Layout, Direction, Constraint};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::style::{Style, Color};
use ratatui::text::{Text, Span, Spans};

use crate::tui::state::app_state::AppState;

/// Render the context view
pub fn render_context<B: ratatui::backend::Backend>(frame: &mut Frame<B>, _state: &AppState) {
    // This is a placeholder implementation
    // In a real implementation, we would render a proper context view
    
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
        "PostDevAI Context",
        Style::default().fg(Color::Red).add_modifier(ratatui::style::Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL).title("Context"));
    
    frame.render_widget(header, chunks[0]);
    
    // Render content
    let content = Paragraph::new(vec![
        Spans::from(vec![Span::styled("Current Context:", Style::default().add_modifier(ratatui::style::Modifier::BOLD))]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("No active context.")])
    ])
    .block(Block::default().borders(Borders::ALL).title("Active Context"))
    .wrap(Wrap { trim: true });
    
    frame.render_widget(content, chunks[1]);
}