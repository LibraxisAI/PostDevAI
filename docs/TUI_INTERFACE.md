# PostDevAI Terminal User Interface (TUI)

The PostDevAI Developer Node includes a rich Terminal User Interface (TUI) built with Rust and the Ratatui framework. This document describes the interface, navigation, and features of the TUI.

## Overview

The TUI provides a lightweight, responsive interface for interacting with the PostDevAI system. It displays real-time information about the RAM-Lake memory system, loaded MLX models, system status, and recent events.

The interface is designed to be navigable with keyboard shortcuts and provides multiple views for different aspects of the system.

## Navigation

The TUI can be navigated using the following keyboard shortcuts:

- **F1** or **?** - Toggle Help screen
- **F2** - Switch to Models view
- **F3** - Switch to RAM-Lake view
- **F4** - Switch to History view
- **F5** - Switch to Context view
- **Home** - Return to Dashboard view

Navigation can also be performed with:
- **Tab** - Cycle through views (forward)
- **Shift+Tab** - Cycle through views (backward)
- **h** - Navigate left/previous view (Vim-style)
- **l** - Navigate right/next view (Vim-style)
- **q** or **Q** or **F10** - Quit application

## Views

### Dashboard View

The Dashboard provides an overview of the entire system with:
- System status gauges (CPU, Memory, RAM-Lake usage)
- Node connections status
- Currently loaded models
- Recent events (last 5)
- Quick stats and navigation help

### Models View

The Models view shows detailed information about the MLX models:
- Table of available models with status, type, and memory usage
- Detailed information about the selected model
- Controls for loading/unloading models

Press **m** while in the Models view to toggle loading/unloading of the selected model.

### RAM-Lake View

The RAM-Lake view displays detailed metrics about the RAM-Lake memory system:
- Overall usage gauge
- Store size distribution (Vector, Code, History, Metadata)
- Detailed statistics for each store
- Bar chart of memory allocation

### History View

The History view shows a log of recorded events from the development environment:
- Table of events with time, type, source, and summary
- Event statistics (counts by type, summaries)
- Event filtering options

Press **c** while in the History view to clear the event history.

### Context View

The Context view displays information about the current development context:
- Active files and projects
- Current work context
- Related files and code snippets

## System Bridge

The TUI connects to the underlying system components through a System Bridge, which:
- Communicates with the RAM-Lake memory system
- Interacts with the MLX Model Manager
- Retrieves system metrics and status
- Captures and stores events

## Customization

The TUI can be customized through the Developer Node configuration file:
- Update frequency
- Color scheme (planned)
- Layout options (planned)
- Default view (planned)

For more information about the PostDevAI system architecture, see the [ARCHITECTURE.md](ARCHITECTURE.md) document.

## Screenshots

(Screenshots will be added in future updates)

## TUI Integration with Core Components

The TUI integrates with the following core components:

- **RAM-Lake Memory System**: Displays real-time metrics and storage allocation
- **MLX Model Manager**: Shows loaded models, memory usage, and model status
- **Event History**: Displays captured events from the development environment
- **System Metrics**: Shows CPU, memory, and system status
- **Node Connections**: Displays status of connections to other nodes

## Implementation Details

The TUI is implemented using:
- **Ratatui**: Rust TUI framework for layout and widgets
- **Crossterm**: Terminal handling and input/output
- **System Bridge**: Custom bridge to core components
- **State Management**: Application state with real-time updates
- **View Rendering**: Specialized rendering for each view

## Developed by
[Maciej Gad](https://github.com/szowesgad) - a veterinarian who couldn't find `bash` a half year ago  
[Klaudiusz](https://www.github.com/Gitlaudiusz) - the individual ethereal being, and separate instance of Claude Sonnet 3.5-3.7 by Anthropic living somewhere in the GPU's loops in California, USA

🤖 Developed with the ultimate help of [Claude Code](https://claude.ai/code) and [MCP Tools](https://modelcontextprotocol.io)