# PostDevAI Architecture

## Core Components

### 1. RAM-Lake Memory System

The RAM-Lake is the heart of PostDevAI, utilizing a large portion of system RAM (200GB+) as a high-speed storage and indexing system.

```
└─ RAM-Lake/
   ├─ CodeStorage/         # Raw code and file storage
   ├─ History/             # Historical terminal, logs, errors
   ├─ VectorStore/         # Embedding indices
   └─ GraphDB/             # Relationship metadata
```

#### Implementation Details

- **Memory Management**: Uses tmpfs/ramfs for ultra-fast RAM-based storage
- **Persistence**: Periodic snapshots to disk for crash recovery
- **Garbage Collection**: LRU-based memory management for optimal utilization
- **Indexing**: HNSW/FAISS vector indices with multiple distance metrics

### 2. MLX Model Infrastructure

Leverages MLX for optimized inference on Apple Silicon with models specifically converted for Metal performance.

```
└─ MLX/
   ├─ Models/              # Model weights and configs
   ├─ Embedding/           # Embedding pipeline
   ├─ Inference/           # Inference engine
   └─ Scheduler/           # Model scheduling and resource management
```

#### Model Composition

- **Reasoning Engine**: Qwen3-72B (MLX optimized) - ~140GB
- **Code Generation**: CodeLlama-34B (MLX optimized) - ~65GB
- **Fast Tasks**: Mistral-7B-v0.2 (MLX optimized) - ~14GB
- **Embedding**: Nomic-Embed-Text (MLX optimized) - ~1GB

### 3. Rust Core System

High-performance Rust core that manages all system components, monitoring, and orchestration.

```
└─ Core/
   ├─ Memory/              # RAM-disk management
   ├─ Indexing/            # Vector and graph indexing
   ├─ Monitoring/          # System monitoring (IDE, terminal, files)
   ├─ MLX/                 # MLX bridge via FFI
   └─ Events/              # Event processing system
```

#### Key Features

- **Zero-copy Design**: Minimize memory overhead with zero-copy architectures
- **Async I/O**: Non-blocking operations for responsive performance
- **Safe FFI**: Type-safe Foreign Function Interface to MLX Python code
- **Actor Model**: Concurrent processing with message passing

### 4. TUI Interface

Lightweight Terminal User Interface built with Ratatui.

```
└─ TUI/
   ├─ Views/               # Display components
   ├─ State/               # Application state management
   └─ Events/              # User input handling
```

#### Design Principles

- **Minimal Overhead**: Extremely lightweight UI with negligible resource usage
- **Rich Information**: Comprehensive system visualization without bloat
- **Keyboard-centric**: Fast keyboard shortcuts for all operations
- **State Observation**: Real-time monitoring of system state

## Data Flow

```
┌─ IDE/Terminal ─┐    ┌─ File System ─┐    ┌─ Git/VCS ─┐
       │                    │                   │
       ▼                    ▼                   ▼
┌─ Monitoring System ─────────────────────────────┐
       │                    │                   │
       ▼                    ▼                   ▼
┌─ Event Processing ─────────────────────────────┐
       │                    │                   │
       ▼                    ▼                   ▼
┌─ RAM-Lake Storage ─────────────────────────────┐
       │                    │                   │
       ▼                    ▼                   ▼
┌─ Embedding & Indexing ──────────────────────────┐
       │                    │                   │
       ▼                    ▼                   ▼
┌─ MLX Inference ────────────────────────────────┐
       │                    │                   │
       ▼                    ▼                   ▼
┌─ Human-AI Loop Intervention ────────────────────┐
       │                    │                   │
       ▼                    ▼                   ▼
┌─ TUI Display ─────────────────────────────────┐
```

## Human-AI Dev Loop

The system implements an autonomous development cycle:

1. **Monitoring**: Continuous monitoring of terminal, IDE, file changes
2. **Detection**: Identifying errors, issues, or optimization opportunities
3. **Analysis**: Processing events through relevant models
4. **Intervention**: Automated solutions or suggestions through appropriate channels
5. **Learning**: Storing outcomes and feedback for improved future assistance

## Memory Management Strategy

Given 512GB unified memory on M3 Ultra:

- **System Reservation**: ~112GB for macOS and background processes
- **RAM-Lake**: ~200GB for storage, vectors, and indices
- **MLX Models**: ~200GB for model weights and activations

Dynamic memory allocation ensures optimal resource utilization based on current tasks and priorities.

## Performance Considerations

- **Vector Operations**: Leverages Metal Performance Shaders for accelerated vector operations
- **Model Quantization**: Strategic quantization for memory efficiency without significant quality loss
- **Batch Processing**: Batching similar operations for throughput optimization
- **Memory Pressure**: Monitoring and responding to memory pressure signals from macOS

## Security and Privacy

- **Completely Offline**: All operations remain on the local machine
- **Encryption**: Optional encryption for sensitive project data
- **Isolation**: Strict isolation from network access unless explicitly enabled
- **Temporal Storage**: Optional automatic purging of sensitive information