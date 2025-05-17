# Rust TUI Implementation

PostDevAI features a high-performance Terminal User Interface (TUI) built with Rust and Ratatui. This document outlines the design principles, implementation details, and usage patterns of the TUI component.

## Design Philosophy

The TUI follows several core principles:

1. **Minimal Resource Usage**
   - Near-zero impact on system performance
   - Negligible memory footprint
   - Efficient rendering even with frequent updates

2. **Information Density**
   - High information-to-screen-space ratio
   - Focus on actionable metrics and insights
   - Progressive disclosure of details

3. **Keyboard-centric**
   - Fast, efficient keyboard shortcuts for all operations
   - Vim-inspired navigation patterns
   - Minimal reliance on mouse interactions

4. **Adaptive Layout**
   - Responsive design that adapts to terminal dimensions
   - Contextual layout based on current operations
   - Focus shifts based on system activity

## Core Components

### State Management

```rust
// src/tui/state/app_state.rs

pub struct AppState {
    /// Current tab index
    pub tab_index: usize,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Model statuses
    pub model_states: HashMap<String, ModelState>,
    /// RAM-Lake metrics
    pub ramlake_metrics: RamLakeMetrics,
    /// Current events and notifications
    pub events: VecDeque<SystemEvent>,
    /// Runtime performance metrics
    pub performance: PerformanceMetrics,
    /// Current project context
    pub context: ProjectContext,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tab_index: 0,
            memory_stats: MemoryStats::default(),
            model_states: HashMap::new(),
            ramlake_metrics: RamLakeMetrics::default(),
            events: VecDeque::with_capacity(100),
            performance: PerformanceMetrics::default(),
            context: ProjectContext::default(),
        }
    }
    
    pub fn update(&mut self, system_state: &SystemState) {
        // Update internal state from system state
        self.memory_stats = system_state.memory_stats.clone();
        
        // Update model states
        for (name, state) in &system_state.models {
            self.model_states.insert(name.clone(), state.clone());
        }
        
        // Update RAM-Lake metrics
        self.ramlake_metrics = system_state.ramlake_metrics.clone();
        
        // Add new events
        for event in &system_state.new_events {
            self.events.push_back(event.clone());
        }
        
        // Maintain maximum events
        while self.events.len() > 100 {
            self.events.pop_front();
        }
        
        // Update performance metrics
        self.performance = system_state.performance.clone();
        
        // Update project context
        self.context = system_state.context.clone();
    }
}
```

### UI Components

```rust
// src/tui/views/dashboard.rs

pub fn render_dashboard<B: Backend>(frame: &mut Frame<B>, app_state: &AppState) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(1),  // Footer
        ])
        .split(frame.size());
    
    // Render header
    render_header(frame, chunks[0], app_state);
    
    // Main content layout
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),  // Left panel
            Constraint::Percentage(30),  // Right panel
        ])
        .split(chunks[1]);
    
    // Left panel layout
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),  // Context
            Constraint::Percentage(60),  // Events
        ])
        .split(main_chunks[0]);
    
    // Render context panel
    render_context_panel(frame, left_chunks[0], app_state);
    
    // Render events panel
    render_events_panel(frame, left_chunks[1], app_state);
    
    // Render system stats panel
    render_system_stats(frame, main_chunks[1], app_state);
    
    // Render footer
    render_footer(frame, chunks[2]);
}

fn render_header<B: Backend>(frame: &mut Frame<B>, area: Rect, app_state: &AppState) {
    let title = format!(
        "PostDevAI - RAM: {}/{} GB - Models: {} loaded",
        app_state.memory_stats.used_gb,
        app_state.memory_stats.total_gb,
        app_state.model_states.len()
    );
    
    let header = Paragraph::new(Text::from(title))
        .style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    
    frame.render_widget(header, area);
}

fn render_context_panel<B: Backend>(frame: &mut Frame<B>, area: Rect, app_state: &AppState) {
    let context = &app_state.context;
    
    let items = vec![
        format!("Project: {}", context.project_name),
        format!("Files: {} indexed, {} modified", context.indexed_files, context.modified_files),
        format!("Current file: {}", context.current_file),
        format!("Last action: {}", context.last_action),
        format!("Analysis: {}", context.analysis_status),
    ];
    
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|i| ListItem::new(Text::from(i.as_str())))
        .collect();
    
    let list = List::new(list_items)
        .block(Block::default()
            .title("Current Context")
            .borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::White));
    
    frame.render_widget(list, area);
}

// Additional rendering functions...
```

### Event Handling

```rust
// src/tui/events/handler.rs

pub enum Event {
    Input(KeyEvent),
    Tick,
    SystemUpdate(SystemState),
}

pub struct EventHandler {
    rx: mpsc::Receiver<Event>,
    _tx: mpsc::Sender<Event>,
    _tick_handle: Option<JoinHandle<()>>,
    _system_handle: Option<JoinHandle<()>>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let tx_clone = tx.clone();
        
        // Set up ticker for UI updates
        let tick_handle = spawn_ticker(tick_rate, tx_clone);
        
        // Set up system state poller
        let tx_clone = tx.clone();
        let system_handle = spawn_system_poller(Duration::from_millis(500), tx_clone);
        
        Self {
            rx,
            _tx: tx,
            _tick_handle: Some(tick_handle),
            _system_handle: Some(system_handle),
        }
    }
    
    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}

fn spawn_ticker(tick_rate: Duration, tx: mpsc::Sender<Event>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            
            if timeout.as_secs() == 0 {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                last_tick = Instant::now();
            } else {
                thread::sleep(timeout);
            }
        }
    })
}

fn spawn_system_poller(poll_rate: Duration, tx: mpsc::Sender<Event>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(poll_rate);
            
            // Get current system state
            let system_state = match get_system_state() {
                Ok(state) => state,
                Err(_) => continue,
            };
            
            // Send system update event
            if tx.send(Event::SystemUpdate(system_state)).is_err() {
                break;
            }
        }
    })
}
```

## UI Layout

The TUI is organized into several key areas:

```
┌─ PostDevAI ─ RAM: 289/512 GB ─ Models: 3 loaded ─────────────────────────────┐
│                                                                               │
│ ┌─ Current Context ───────────────────┐  ┌─ System Stats ─────────────────┐  │
│ │ Project: ramlake-core               │  │ ┌─ Memory ─────────────────┐   │  │
│ │ Files: 126 indexed, 14 modified     │  │ │ RAM-Lake:   196.4 GB     │   │  │
│ │ Current file: src/core/indexing.rs  │  │ │ MLX Models: 183.7 GB     │   │  │
│ │ Last action: Terminal error         │  │ │ System:      89.3 GB     │   │  │
│ │ Analysis: Running semantic indexing │  │ │ Available:   42.6 GB     │   │  │
│ └───────────────────────────────────┬─┘  │ └─────────────────────────┬─┘   │  │
│                                     │    │                           │     │  │
│ ┌─ Recent Events ──────────────────┐│    │ ┌─ Models ──────────────┐ │     │  │
│ │ [12:42:03] Error: Memory overflow││    │ │ Qwen3-72B:    Running │ │     │  │
│ │ [12:41:58] Build: Compilation fai││    │ │ CodeLlama-34B: Standby│ │     │  │
│ │ [12:41:30] Terminal: Command comp││    │ │ Mistral-7B:   Active │ │     │  │
│ │ [12:40:22] Analysis: Similar issu││    │ └─────────────────────┬─┘ │     │  │
│ │ [12:39:55] Terminal: npm install ││    │                       │   │     │  │
│ │ [12:39:40] File: src/core/indexin││    │ ┌─ RAM-Lake ─────────┐│   │     │  │
│ │                                   ││    │ │ Vectors:  94.2 GB  ││   │     │  │
│ │                                   ││    │ │ Code:     68.7 GB  ││   │     │  │
│ │                                   ││    │ │ History:  25.1 GB  ││   │     │  │
│ │                                   ││    │ │ Metadata:  8.4 GB  ││   │     │  │
│ └───────────────────────────────────┴┘    │ └─────────────────────┴───┴─────┘  │
│                                                                               │
│ F1:Help  F2:Models  F3:RAM-Lake  F4:History  F5:Context  F10:Quit            │
└───────────────────────────────────────────────────────────────────────────────┘
```

## Keyboard Shortcuts

| Key       | Action                           |
|-----------|----------------------------------|
| F1        | Help screen                      |
| F2        | Models management view           |
| F3        | RAM-Lake management view         |
| F4        | History view                     |
| F5        | Context view                     |
| F10       | Quit                             |
| Tab       | Cycle between panels             |
| Arrow keys| Navigate within panels           |
| Enter     | Select/expand item               |
| Esc       | Back/collapse                    |
| /         | Search                           |
| :         | Command mode                     |
| m         | Toggle model state               |
| r         | Refresh data                     |
| c         | Clear events                     |
| s         | Save snapshot                    |
| h/j/k/l   | Vim-style navigation             |

## FFI to Core System

The TUI communicates with the core Rust system via direct function calls, and with the MLX components via FFI:

```rust
// src/tui/bridge/system_bridge.rs

use std::ffi::{c_void, CString};
use std::os::raw::c_char;

#[repr(C)]
pub struct SystemStateFfi {
    // FFI-safe representation of SystemState
    memory_used: u64,
    memory_total: u64,
    models_count: u32,
    // etc.
}

extern "C" {
    fn get_system_state_ffi(out: *mut SystemStateFfi) -> bool;
    fn update_model_state_ffi(model_name: *const c_char, new_state: u32) -> bool;
    // etc.
}

pub fn get_system_state() -> Result<SystemState, String> {
    let mut ffi_state = SystemStateFfi {
        memory_used: 0,
        memory_total: 0,
        models_count: 0,
        // etc.
    };
    
    let success = unsafe {
        get_system_state_ffi(&mut ffi_state)
    };
    
    if !success {
        return Err("Failed to get system state from FFI".to_string());
    }
    
    // Convert FFI representation to Rust representation
    Ok(SystemState {
        memory_stats: MemoryStats {
            used_gb: ffi_state.memory_used as f64 / 1024.0 / 1024.0 / 1024.0,
            total_gb: ffi_state.memory_total as f64 / 1024.0 / 1024.0 / 1024.0,
        },
        // etc.
    })
}

pub fn update_model_state(model_name: &str, new_state: ModelState) -> Result<(), String> {
    let model_name_c = match CString::new(model_name) {
        Ok(s) => s,
        Err(_) => return Err("Invalid model name for FFI".to_string()),
    };
    
    let state_val = match new_state {
        ModelState::Running => 1,
        ModelState::Standby => 2,
        ModelState::Loading => 3,
        ModelState::Unloaded => 4,
        ModelState::Error => 5,
    };
    
    let success = unsafe {
        update_model_state_ffi(model_name_c.as_ptr(), state_val)
    };
    
    if !success {
        return Err("Failed to update model state via FFI".to_string());
    }
    
    Ok(())
}
```

## Performance Considerations

The TUI is designed with extreme performance in mind:

1. **Render Optimization**
   - Partial screen updates when possible
   - Frame rate limiting to 30fps max
   - Double-buffering to eliminate flicker

2. **Thread Management**
   - UI rendering on dedicated thread
   - Event handling on separate thread
   - Non-blocking I/O for system state updates

3. **Memory Footprint**
   - Static allocation where possible
   - Bounded event history
   - Zero-copy data sharing with core system

## Future TUI Enhancements

Planned improvements for the TUI include:

1. **Rich Text Support**
   - Syntax highlighting for code snippets
   - Markdown rendering for documentation
   - ANSI escape sequence parsing for terminal output

2. **Interactive Visualizations**
   - Memory usage heat maps
   - RAM-Lake utilization graphs
   - Model performance metrics

3. **Advanced Interaction**
   - Split-pane view for side-by-side comparison
   - Searchable command palette
   - Customizable keybindings