use ratatui::Frame;
use ratatui::layout::{Layout, Direction, Constraint};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::style::{Style, Color};
use ratatui::text::{Text, Span, Spans};

use crate::tui::state::app_state::AppState;

/// Render the help view
pub fn render_help<B: ratatui::backend::Backend>(frame: &mut Frame<B>, _state: &AppState) {
    // This is a placeholder implementation
    // In a real implementation, we would render a proper help screen
    
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
        "PostDevAI Help",
        Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL).title("Help"));
    
    frame.render_widget(header, chunks[0]);
    
    // Render content
    let content = Paragraph::new(vec![
        Spans::from(vec![Span::styled("Key Bindings:", Style::default().add_modifier(ratatui::style::Modifier::BOLD))]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("F1 or ? - Show this help")]),
        Spans::from(vec![Span::raw("F2 - Models view")]),
        Spans::from(vec![Span::raw("F3 - RAM-Lake view")]),
        Spans::from(vec![Span::raw("F4 - History view")]),
        Spans::from(vec![Span::raw("F5 - Context view")]),
        Spans::from(vec![Span::raw("Home - Dashboard view")]),
        Spans::from(vec![Span::raw("Tab - Next view")]),
        Spans::from(vec![Span::raw("Shift+Tab - Previous view")]),
        Spans::from(vec![Span::raw("q or Q - Quit")]),
        Spans::from(vec![Span::raw("r - Refresh")]),
        Spans::from(vec![Span::raw("c - Clear events (in History view)")]),
        Spans::from(vec![Span::raw("Ctrl+s - Save snapshot")])
    ])
    .block(Block::default().borders(Borders::ALL).title("Key Bindings"))
    .wrap(Wrap { trim: true });
    
    frame.render_widget(content, chunks[1]);
}