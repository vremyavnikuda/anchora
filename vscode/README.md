# Anchora Task Manager - VSCode Extension

## Overview

Anchora Task Manager is a VSCode extension that provides intelligent task tracking and management directly within your codebase. Unlike traditional task managers that operate at the file level, Anchora embeds tasks directly into source code comments using a structured syntax, enabling developers to maintain task context alongside implementation.

## Core Architecture

### Backend Integration
- **Rust Backend**: High-performance backend service (`anchora.exe`/`anchora`) handles task parsing, storage, and file monitoring
- **JSON-RPC Communication**: Frontend-backend communication via stdin/stdout using JSON-RPC 2.0 protocol
- **Asynchronous Processing**: Non-blocking task operations with real-time file system monitoring
- **Cross-Platform Support**: Works on Windows, macOS, and Linux with platform-specific binary bundling

### Extension Components

#### 1. Core Modules
- **`extension.ts`**: Main entry point, handles activation, initialization, and extension lifecycle
- **`client.ts`**: Backend communication layer, manages JSON-RPC requests/responses and process lifecycle
- **`taskProvider.ts`**: Task tree data provider for Activity Bar and Explorer views
- **`commands.ts`**: Command implementations for all user-facing actions
- **`decorations.ts`**: Visual task indicators in editor with color-coded status decorations
- **`statusBar.ts`**: Status bar integration showing task counts and current context
- **`types.ts`**: TypeScript type definitions with strict typing for all data structures

#### 2. User Interface Integration
- **Activity Bar Panel**: Dedicated Anchora icon (📋) with task tree view
- **Explorer Integration**: Task tree embedded in Explorer sidebar
- **Command Palette**: All commands accessible via `Ctrl+Shift+P`
- **Context Menus**: Right-click actions on tasks and tree items
- **Editor Decorations**: Inline visual indicators for task references
- **Status Bar**: Real-time task statistics and current context display

## Task Management System

### Task Syntax Specification

Anchora uses a standardized comment-based syntax for embedding tasks in source code:

```
// {section}:{task_id}: {description}           // Full task definition
// {section}:{task_id}:{status}: {description}   // Task with explicit status
// {section}:{task_id}                          // Simple task reference
// {section}:{task_id}:{note}                   // Task reference with note
// {section}:{task_id}:{status}                 // Status update only
```

#### Syntax Rules
- **Section Names**: Alphanumeric with underscores (e.g., `dev`, `bug_fix`, `refactor`)
- **Task IDs**: Alphanumeric with underscores, unique within section
- **Statuses**: `todo`, `in_progress`, `done`, `blocked`
- **Descriptions**: Free-form text following the colon and space
- **Notes**: Short identifiers without spaces (e.g., `main_logic`, `validation`)

#### Examples
```rust
// dev:auth_system: Implement OAuth2 authentication flow
// dev:auth_system:todo: Implement OAuth2 authentication flow  
// dev:auth_system
// dev:auth_system:token_validation
// dev:auth_system:in_progress
```

### Task Status Management

#### Status Types
- **`todo`** (○): Tasks waiting to be started
- **`in_progress`** (◐): Currently active tasks
- **`done`** (●): Completed tasks
- **`blocked`** (◯): Tasks waiting on dependencies

#### Status Operations
- **Automatic Detection**: Status parsed from task syntax in comments
- **Manual Updates**: Right-click context menu or command palette
- **Visual Indicators**: Color-coded decorations in editor
- **Real-time Sync**: Changes reflected immediately across all views

## Feature Matrix

### Project Management

| Feature | Description | Access Method |
|---------|-------------|---------------|
| **Project Initialization** | Auto-detect or manually initialize Anchora project | Automatic on extension activation |
| **Task Scanning** | Recursive project scan for task comments | `Ctrl+Shift+D` → Scan Project |
| **File Monitoring** | Real-time file change detection | Automatic via backend file watcher |
| **Data Persistence** | Task storage in `.anchora/tasks.json` | Automatic on task changes |

### Task Operations

| Operation | Description | Keybinding | Command |
|-----------|-------------|------------|----------|
| **Create Task** | Interactive task creation wizard | `Ctrl+Shift+T` | `anchora.createTask` |
| **Navigate to Definition** | Jump to task definition | `F12` | `anchora.goToTaskDefinition` |
| **Find All References** | Show all task references | `Shift+F12` | `anchora.findTaskReferences` |
| **Update Status** | Change task status | Context menu | `anchora.updateTaskStatus` |
| **Refresh Tasks** | Reload task data | Manual | `anchora.refreshTasks` |

### Viewing and Navigation

| View | Description | Access | Features |
|------|-------------|--------|----------|
| **Task Overview** | Comprehensive task dashboard | `Ctrl+Shift+A` | Statistics, charts, timestamps |
| **Status Filter** | Tasks grouped by status | Command palette | Filter by todo/in_progress/done/blocked |
| **Search Interface** | Advanced task search | `Ctrl+Shift+F` | Search by title, ID, description |
| **Task Dashboard** | Quick actions panel | `Ctrl+Shift+D` | Create, scan, view, search |
| **Activity Bar** | Main task tree view | Click Anchora icon | Hierarchical task organization |
| **Explorer Integration** | Tasks in Explorer sidebar | Always visible | File-based task context |

## Technical Specifications

### File Support
Default file patterns for task scanning:
- **Rust**: `**/*.rs`
- **TypeScript**: `**/*.ts`
- **JavaScript**: `**/*.js`
- **Python**: `**/*.py`
- **Java**: `**/*.java`
- **C/C++**: `**/*.cpp`, `**/*.c`, `**/*.h`
- **Go**: `**/*.go`

### Ignored Directories
Automatically excluded from scanning:
- `target`, `node_modules`, `.git`, `.vscode`, `.anchora`
- `dist`, `build`, `__pycache__`, `.idea`, `out`

### Configuration Options

```json
{
  "anchora.filePatterns": [
    "**/*.rs", "**/*.ts", "**/*.js", "**/*.py",
    "**/*.java", "**/*.cpp", "**/*.c", "**/*.h", "**/*.go"
  ],
  "anchora.ignoredDirectories": [
    "target", "node_modules", ".git", ".vscode",
    "dist", "build", "__pycache__"
  ],
  "anchora.decorationColors": {
    "todo": "#ff6b6b",
    "in_progress": "#4ecdc4",
    "done": "#45b7d1", 
    "blocked": "#f9ca24"
  }
}
```

## Installation and Setup

### System Requirements
- **VSCode**: Version 1.70.0 or higher
- **Operating System**: Windows, macOS, or Linux
- **No additional dependencies**: Backend binary bundled with extension

### Installation Methods

#### From VSIX Package (Recommended)
```bash
code --install-extension anchora-task-manager-0.1.0.vsix
```

#### Development Installation
1. Clone repository: `git clone https://github.com/vremyavnikuda/anchora`
2. Navigate to VSCode extension: `cd anchora/vscode`
3. Install dependencies: `npm install`
4. Open in VSCode: `code .`
5. Press `F5` to launch Extension Development Host

### Project Initialization

#### Automatic Initialization
- Extension auto-detects project type on workspace opening
- Creates `.anchora/` directory if not present
- Initializes `tasks.json` with default structure

#### Manual Initialization
- Use Command Palette: `Initialize Anchora Project`
- Or click "Initialize" in Welcome view

## Build and Development

### Build Scripts

| Script | Purpose | Command |
|--------|---------|----------|
| **Full Build** | Build backend + compile frontend | `npm run build:all` |
| **Backend Only** | Build Rust backend binary | `npm run build:backend` |
| **Frontend Only** | Compile TypeScript to JavaScript | `npm run compile` |
| **Copy Backend** | Copy backend binary to server/ | `npm run copy:backend` |
| **Package** | Create VSIX with bundled backend | `npm run package` |
| **Quick Package** | Package without backend rebuild | `npm run package:quick` |

### Backend Binary Resolution

The extension searches for backend binary in the following order:
1. **Bundled binary**: `extension/server/anchora.exe` (highest priority)
2. **Project release**: `workspace/target/release/anchora.exe`  
3. **Project debug**: `workspace/target/debug/anchora.exe`
4. **Project root**: `workspace/anchora.exe`

### Development Workflow

1. **Setup**: `npm install` in `vscode/` directory
2. **Backend Build**: `cd .. && cargo build --release && cd vscode`
3. **Copy Binary**: `npm run copy:backend`
4. **Compile Frontend**: `npm run compile`
5. **Test**: Press `F5` in VSCode to launch Extension Development Host
6. **Package**: `npm run package` to create VSIX

## Command Reference

### Core Commands

| Command ID | Title | Keybinding | Context |
|------------|-------|------------|----------|
| `anchora.createTask` | Create New Task | `Ctrl+Shift+T` | Anchora project active |
| `anchora.goToTaskDefinition` | Go to Task Definition | `F12` | Task context in editor |
| `anchora.findTaskReferences` | Find All Task References | `Shift+F12` | Task context in editor |
| `anchora.viewAllTaskLists` | View All Task Lists | `Ctrl+Shift+A` | Anchora project active |
| `anchora.searchTasks` | Search Tasks | `Ctrl+Shift+F` | Anchora project active |
| `anchora.openTaskDashboard` | Open Task Dashboard | `Ctrl+Shift+D` | Extension active |
| `anchora.refreshTasks` | Refresh Tasks | - | Manual trigger |
| `anchora.scanProject` | Scan Project for Tasks | - | Manual trigger |
| `anchora.updateTaskStatus` | Update Task Status | Context menu | Task item selected |

### Utility Commands

| Command ID | Title | Purpose |
|------------|-------|----------|
| `anchora.viewTasksByStatus` | View Tasks by Status | Filter tasks by status |
| `anchora.showOutputChannel` | Show Output Channel | Debug and logging |
| `anchora.initializeProject` | Initialize Anchora Project | Manual project setup |

## Data Model

### Project Structure
```
workspace/
├── .anchora/
│   ├── tasks.json          # Task data storage
│   └── backups/            # Automatic backups
├── src/                    # Source code with task comments
└── [other project files]
```

### Task Data Schema
```typescript
interface Task {
  title: string;
  description?: string;
  status: 'todo' | 'in_progress' | 'done' | 'blocked';
  created: string;            // ISO 8601 timestamp
  updated: string;            // ISO 8601 timestamp
  files: Record<string, {     // File path → task references
    lines: number[];          // Line numbers
    notes: Record<number, string>; // Line → note mapping
  }>;
}

interface ProjectData {
  meta: {
    version: string;
    created: string;
    last_updated: string;
    project_name?: string;
  };
  sections: Record<string, Record<string, Task>>; // section → taskId → Task
  index: {
    files: Record<string, string[]>;              // file → taskIds
    tasks_by_status: Record<TaskStatus, string[]>; // status → taskIds
  };
}
```

## Extension States and Contexts

### Context Variables
- **`workspaceHasAnchoraProject`**: Boolean indicating Anchora project detection
- **`anchoraExtensionActive`**: Extension activation state
- **`anchoraTaskContext`**: Task context available in current editor position

### View States
- **Welcome View**: Shown when no Anchora project detected
- **Task Tree View**: Main task hierarchy in Activity Bar
- **Explorer Integration**: Task tree embedded in Explorer
- **Dashboard Views**: Modal task overview and management interfaces

## Error Handling and Diagnostics

### Backend Communication
- **Connection Errors**: Automatic retry with exponential backoff
- **Process Crashes**: Automatic backend restart
- **JSON-RPC Errors**: Graceful error reporting with user notifications

### File System Operations
- **File Access Errors**: Graceful handling with error logging
- **Parse Errors**: Detailed error reporting with line numbers
- **Storage Errors**: Automatic backup recovery

### User Feedback
- **Output Channel**: Detailed logging for debugging
- **Status Bar**: Connection status and error indicators
- **Notifications**: User-friendly error messages and action suggestions

## Known Limitations

### Current Constraints
- **Comment-Only Tasks**: Tasks must be in comments, not in string literals
- **Manual Refresh**: Some file changes require manual refresh
- **Large Projects**: Initial scan performance may be slow for very large codebases
- **Binary Architecture**: Backend binary must match system architecture

### Future Enhancements
- **Real-time Collaboration**: Multi-user task sharing
- **Integration APIs**: Third-party task manager integration
- **Advanced Analytics**: Task completion metrics and trends
- **Custom Task Types**: User-defined task categories and workflows

## License

MIT License - See LICENSE file for complete terms.

## Contributing

See the main Anchora repository at https://github.com/vremyavnikuda/anchora for contribution guidelines.

## Support

For issues, feature requests, or questions:
- **GitHub Issues**: https://github.com/vremyavnikuda/anchora/issues
- **Documentation**: See project repository for detailed documentation
- **Output Channel**: Use "Show Output Channel" command for debugging information