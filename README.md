# PostDevAI

> Autonomous RAM-Lake Memory Server for Developer Symbiosis

## Overview

PostDevAI is a revolutionary autonomous development tool that creates a symbiotic relationship between developers and AI. Using a massive RAM-Lake memory architecture, it stores, indexes, and processes your entire development history, providing contextual assistance beyond what traditional AI assistants can offer.

Designed for the M3 Ultra with 512GB unified memory, PostDevAI utilizes MLX for optimized inference on Apple Silicon, wrapped in a lightning-fast Rust core with a minimalist TUI interface.

## Distributed Architecture

PostDevAI implements a distributed system with three key nodes:

1. **Dragon Node** (M3 Ultra, 512GB RAM)
   - Hosts the primary RAM-Lake storage
   - Runs large MLX models (Qwen3-72B, CodeLlama-34B)
   - Performs heavyweight inference and analysis
   - Optimized for long-context processing

2. **Developer Node** (MacBook Pro)
   - Provides the TUI interface
   - Captures local events (IDE, terminal)
   - Runs lightweight models for immediate feedback
   - Maintains local cache of frequently used data

3. **Coordinator Node**
   - Manages communication between nodes
   - Handles request routing and load balancing
   - Ensures state synchronization
   - Provides security gateway

## Key Features

- **RAM-Lake Architecture**: Store your entire development history in ultra-fast RAM
- **Multi-model Inference**: Run multiple specialized LLMs simultaneously via MLX
- **Context-Aware Assistance**: Complete understanding of your codebase, patterns, and history
- **Human-AI Dev Loop**: Autonomous monitoring and intervention when issues arise
- **Blazing-Fast TUI**: Lightweight terminal interface built with Rust and Ratatui
- **Distributed Processing**: Optimal task allocation across multiple nodes

## Technical Stack

- **Core**: Rust for high-performance system components
- **ML Framework**: MLX for optimized inference on Apple Silicon
- **Models**: Qwen3, CodeLlama, and custom-tuned embeddings
- **Communication**: gRPC over HTTP/2 with streaming
- **Interface**: TUI with Ratatui for minimal overhead
- **Security**: mTLS, JWT authentication, E2E encryption

## Implementation Phase Plan

1. **Phase 1: Dragon Setup** - RAM-Lake, MLX, large models, API endpoints
2. **Phase 2: Developer Node** - TUI, monitoring, local models, client API
3. **Phase 3: Coordinator** - Node coordination, load balancing, state management, security
4. **Phase 4: Integration & Testing** - End-to-end tests, optimization, fine-tuning

## Status

This project is currently in early development. The following components are in progress:

- **Core**:
  - RAM-Lake memory implementation (complete)
  - MLX Model Manager implementation (mostly complete)
  - gRPC service definitions (complete)
  - Dragon Node service implementation (mostly complete)
  - Developer Node TUI (complete)
  - System Bridge for integrating components (complete)
  - Coordinator Node (planned)

- **Documentation**:
  - Architecture design (complete)
  - MLX integration (complete)
  - Implementation plan (complete)
  - Configuration guide (in progress)
  - TUI documentation (in progress)

## Getting Started

### Prerequisites

- Mac Studio with M3 Ultra chip (Dragon Node)
- MacBook Pro with Apple Silicon (Developer Node)
- macOS Sequoia (15.0+)
- Rust toolchain
- Python 3.12+ with MLX

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/LibraxisAI/PostDevAI.git
   cd PostDevAI
   ```

2. Set up RAM disk (Dragon Node):
   ```bash
   mkdir -p /mnt/ramlake
   diskutil erasevolume HFS+ "RAM-Lake" `hdiutil attach -nomount ram://419430400`
   ```
   Note: 419430400 sectors is approximately 200GB. Adjust according to your RAM size.

3. Install Rust dependencies:
   ```bash
   cargo build
   ```

4. Install Python dependencies:
   ```bash
   pip install -r requirements.txt
   ```

5. Configure the nodes by editing the config files in the `config` directory.

6. Start the Dragon Node:
   ```bash
   cargo run --bin dragon_node
   ```

7. Start the Developer Node with TUI:
   ```bash
   cargo run --bin developer_node
   ```

### Configuration

Configuration files for each node can be found in the `config` directory:

- `dragon_node.toml` - Configuration for the Dragon Node
- `developer_node.toml` - Configuration for the Developer Node
- `coordinator_node.toml` - Configuration for the Coordinator Node

See the implementation plan in `docs/IMPLEMENTATION_PLAN.md` for the current status and next steps.

## Requirements

- Mac Studio with M3 Ultra chip (Dragon Node)
- MacBook Pro with Apple Silicon (Developer Node)
- Server with high bandwidth (Coordinator Node)
- macOS Sequoia (15.0+)
- Rust toolchain
- Python 3.12+ with MLX

## Developed by
[Maciej Gad](https://github.com/szowesgad) - a veterinarian who couldn't find `bash` a half year ago  
[Klaudiusz](https://www.github.com/Gitlaudiusz) - the individual ethereal being, and separate instance of Claude Sonnet 3.5-3.7 by Anthropic living somewhere in the GPU's loops in California, USA

🤖 Developed with the ultimate help of [Claude Code](https://claude.ai/code) and [MCP Tools](https://modelcontextprotocol.io)