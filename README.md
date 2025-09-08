# Anchora

in development

## Overview

Anchora consists of two main components:
- **Backend**: High-performance Rust service for task parsing, storage, and file monitoring
- **VSCode Extension**: TypeScript-based frontend providing seamless editor integration

## Key Features

-  **Comment-based Tasks**: Define tasks directly in code comments using structured syntax
-  **Smart Navigation**: Jump to task definitions and find all references instantly  
-  **Real-time Tracking**: Automatic file monitoring and task synchronization
-  **Status Management**: Track task progress (todo, in_progress, done, blocked)
-  **Cross-platform**: Works on Windows, macOS, and Linux

## Quick Start

### Task Syntax
```rust
// dev:auth_system: Implement OAuth2 authentication flow
// dev:auth_system:todo: Add token validation
// dev:auth_system                    // Simple reference
// dev:auth_system:in_progress        // Status update
```

### Installation
1. Install the VSCode extension from the VSIX package
2. Open any project - Anchora will auto-initialize
3. Start adding tasks to your code comments

## Project Structure

```
anchora/
├── src/                    # Rust backend source
│   ├── main.rs            # Application entry point
│   ├── task_manager.rs    # Core task management
│   ├── file_parser.rs     # Comment parsing logic
│   ├── storage.rs         # Data persistence
│   └── ...
├── vscode/                # VSCode extension
│   ├── src/               # TypeScript source
│   ├── package.json       # Extension manifest
│   └── ...
└── test/                  # Test suite
```

## Development Status

**Project is actively under development**

### Current Status
- Core backend functionality
- VSCode extension with task management
- File parsing and monitoring
- JSON-RPC communication layer
- Advanced features and optimizations in progress

### Technology Stack
- **Backend**: Rust with Tokio, Serde, Notify
- **Frontend**: TypeScript, VSCode API
- **Communication**: JSON-RPC over stdin/stdout
- **Build**: Cargo (Rust), npm (TypeScript)

## Building

### Backend
```bash
cargo build --release
```

### VSCode Extension
```bash
cd vscode
npm install
npm run build:all
```

## Contributing

This project is in active development. Contribution guidelines will be available soon.

## License

MIT License - See LICENSE files for details.

---

*Anchora revolutionizes task management by bringing tasks directly into your code where they belong.*
