use std::io;
use std::time::{Duration, Instant};
use std::sync::Arc;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use parking_lot::RwLock;

use crate::core::memory::ramlake::{RamLake, RamLakeMetrics};
use crate::system::SystemState;
use crate::tui::state::app_state::{AppState, ModelInfo};
use crate::tui::views::dashboard::render_dashboard;
use crate::tui::views::help::render_help;
use crate::tui::views::models::render_models;
use crate::tui::views::ramlake::render_ramlake;
use crate::tui::views::history::render_history;
use crate::tui::views::context::render_context;
use crate::tui::bridge::system_bridge::SystemBridge;

/// Main TUI application for PostDevAI
pub struct App {
    /// Application state
    state: Arc<RwLock<AppState>>,
    
    /// Current view being displayed
    current_view: View,
    
    /// Whether help is being shown
    show_help: bool,
    
    /// Last frame update time
    last_update: Instant,
    
    /// Update frequency
    update_freq: Duration,
    
    /// System bridge for connecting to underlying components
    system_bridge: Arc<RwLock<SystemBridge>>,
    
    /// RAM-Lake instance
    ramlake: Option<Arc<RwLock<RamLake>>>,
}

/// Available views in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Models,
    RamLake,
    History,
    Context,
}

impl App {
    /// Create a new application
    pub fn new(update_freq: Duration) -> Self {
        let system_bridge = Arc::new(RwLock::new(SystemBridge::new()));
        
        Self {
            state: Arc::new(RwLock::new(AppState::new())),
            current_view: View::Dashboard,
            show_help: false,
            last_update: Instant::now(),
            update_freq,
            system_bridge,
            ramlake: None,
        }
    }
    
    /// Set RAM-Lake instance
    pub fn set_ramlake(&mut self, ramlake: Arc<RwLock<RamLake>>) {
        self.ramlake = Some(ramlake.clone());
        self.system_bridge.write().set_ramlake(ramlake);
    }
    
    /// Run the application
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            // Update application state if needed
            let now = Instant::now();
            if now.duration_since(self.last_update) >= self.update_freq {
                self.update_state()?;
                self.last_update = now;
            }
            
            // Draw the UI
            terminal.draw(|f| {
                if self.show_help {
                    render_help(f, &self.state.read());
                } else {
                    match self.current_view {
                        View::Dashboard => render_dashboard(f, &self.state.read()),
                        View::Models => render_models(f, &self.state.read()),
                        View::RamLake => render_ramlake(f, &self.state.read()),
                        View::History => render_history(f, &self.state.read()),
                        View::Context => render_context(f, &self.state.read()),
                    }
                }
            })?;
            
            // Handle input
            if let Ok(true) = self.handle_input() {
                return Ok(());
            }
        }
    }
    
    /// Update application state
    fn update_state(&mut self) -> io::Result<()> {
        let mut bridge = self.system_bridge.write();
        
        // Update AppState with system information
        match bridge.get_system_state() {
            Ok(system_state) => {
                let mut app_state = self.state.write();
                app_state.update(&system_state);
                
                // Update RAM-Lake metrics
                let metrics = bridge.get_ramlake_metrics();
                app_state.update_ramlake_metrics(metrics);
                
                // Update loaded models
                let models = bridge.get_loaded_models();
                app_state.update_loaded_models(models);
                
                // Update recent events
                let events = bridge.get_recent_events(100);
                for event in events {
                    app_state.add_event(event);
                }
                
                // Update recent code
                let code_files = bridge.get_recent_code(100);
                for code in code_files {
                    app_state.add_code(code);
                }
                
                // Update node connections
                let connections = bridge.get_node_connections();
                app_state.update_node_connections(connections);
            }
            Err(e) => {
                // Log error but don't crash
                eprintln!("Failed to get system state: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Handle user input
    fn handle_input(&mut self) -> io::Result<bool> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Quit
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        return Ok(true);
                    }
                    KeyCode::F(10) => {
                        return Ok(true);
                    }
                    
                    // Help
                    KeyCode::F(1) | KeyCode::Char('?') => {
                        self.show_help = !self.show_help;
                    }
                    KeyCode::Esc => {
                        if self.show_help {
                            self.show_help = false;
                        }
                    }
                    
                    // Views
                    KeyCode::F(2) => {
                        self.current_view = View::Models;
                        self.show_help = false;
                    }
                    KeyCode::F(3) => {
                        self.current_view = View::RamLake;
                        self.show_help = false;
                    }
                    KeyCode::F(4) => {
                        self.current_view = View::History;
                        self.show_help = false;
                    }
                    KeyCode::F(5) => {
                        self.current_view = View::Context;
                        self.show_help = false;
                    }
                    KeyCode::Home => {
                        self.current_view = View::Dashboard;
                        self.show_help = false;
                    }
                    
                    // Navigation
                    KeyCode::Tab => {
                        // Cycle through views
                        self.current_view = match self.current_view {
                            View::Dashboard => View::Models,
                            View::Models => View::RamLake,
                            View::RamLake => View::History,
                            View::History => View::Context,
                            View::Context => View::Dashboard,
                        };
                        self.show_help = false;
                    }
                    KeyCode::BackTab => {
                        // Reverse cycle through views
                        self.current_view = match self.current_view {
                            View::Dashboard => View::Context,
                            View::Models => View::Dashboard,
                            View::RamLake => View::Models,
                            View::History => View::RamLake,
                            View::Context => View::History,
                        };
                        self.show_help = false;
                    }
                    
                    // Vim-style navigation
                    KeyCode::Char('h') => {
                        // Left/previous
                        self.current_view = match self.current_view {
                            View::Dashboard => View::Context,
                            View::Models => View::Dashboard,
                            View::RamLake => View::Models,
                            View::History => View::RamLake,
                            View::Context => View::History,
                        };
                        self.show_help = false;
                    }
                    KeyCode::Char('l') => {
                        // Right/next
                        self.current_view = match self.current_view {
                            View::Dashboard => View::Models,
                            View::Models => View::RamLake,
                            View::RamLake => View::History,
                            View::History => View::Context,
                            View::Context => View::Dashboard,
                        };
                        self.show_help = false;
                    }
                    
                    // Model management
                    KeyCode::Char('m') => {
                        if self.current_view == View::Models {
                            // Get the selected model name
                            let model_name = {
                                let state = self.state.read();
                                state.loaded_models.first().map(|m| m.name.clone())
                            };
                            
                            // Use the model name after releasing the lock
                            if let Some(name) = model_name {
                                // For now, just log the action
                                println!("Toggle model: {}", name);
                            }
                        }
                    }
                    
                    // Refresh
                    KeyCode::Char('r') => {
                        // Force refresh
                        self.update_state()?;
                    }
                    
                    // Clear events
                    KeyCode::Char('c') => {
                        if self.current_view == View::History {
                            // Clear events
                            self.state.write().clear_events();
                        }
                    }
                    
                    // Save snapshot
                    KeyCode::Char('s') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            if let Some(_ramlake) = &self.ramlake {
                                // Trigger backup on RAM-Lake
                                // This would actually force a backup
                                println!("Triggered RAM-Lake backup");
                            }
                        }
                    }
                    
                    _ => {}
                }
            }
        }
        
        Ok(false)
    }
}

/// Setup terminal for TUI
pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal state
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Run the TUI application
pub fn run_app() -> io::Result<()> {
    // Setup terminal
    let mut terminal = setup_terminal()?;
    
    // Create app
    let mut app = App::new(Duration::from_millis(250));
    
    // Run app
    let result = app.run(&mut terminal);
    
    // Restore terminal
    restore_terminal(&mut terminal)?;
    
    // Return any error from app
    result
}