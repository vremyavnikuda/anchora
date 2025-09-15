# Anchora - Developer Task Management
> [Русская версия / Russian Version](doc/README_RU.md)
Anchora is a VSCode extension that transforms code comments into a full-featured task management system. Instead of external task trackers, all tasks live directly in your code where they belong.

## Core Functionality

### Tasks in Comments
- Create tasks directly in code comments
- Track status (todo, in_progress, done, blocked)
- Automatic file scanning and synchronization
- Visual indicators in the editor

### Note System
- Create notes for future tasks
- Generate links to convert notes into tasks
- Automatic conversion of notes to tasks when they appear in code

### Navigation and Search
- Jump to task definition
- Find all task references
- Tree view of tasks and notes
- Search across tasks

<details>
<summary><strong>Supported Programming Languages</strong> (click to expand)</summary>

### Anchora supports a wide range of programming languages:

- Rust (.rs), C (.c), C++ (.cpp, .cc, .cxx), C# (.cs), Go (.go)
- JavaScript (.js), TypeScript (.ts), JSX (.jsx), TSX (.tsx)
- HTML (.html), CSS (.css), SCSS (.scss), SASS (.sass), LESS (.less)
- Vue (.vue), Svelte (.svelte)
- Python (.py), Java (.java), PHP (.php), Ruby (.rb)
- Shell (.sh), PowerShell (.ps1), Batch (.bat, .cmd)
- Swift (.swift), Kotlin (.kt), Dart (.dart)
- Objective-C (.m, .mm)
- Haskell (.hs), F# (.fs), OCaml (.ml), Clojure (.clj), Elm (.elm)
- Java (.java), Kotlin (.kt), Scala (.scala), Clojure (.clj)
- Julia (.jl), R (.r), Lua (.lua), Perl (.pl, .pm)
- Erlang (.erl), Elixir (.ex, .exs)
- Docker (.dockerfile), Terraform (.tf), HCL (.hcl)
- YAML (.yaml, .yml), TOML (.toml), JSON (.json), XML (.xml)
- INI (.ini), CFG (.cfg), CONF (.conf)
- Markdown (.md), reStructuredText (.rst), LaTeX (.tex)
- SQL (.sql)
- Visual Basic (.vb)

</details>

## How to Use

### Task Syntax

```rust
// dev:auth_system: Implement OAuth2 authentication
// dev:auth_system:todo: Add token validation
// dev:auth_system                    // Simple reference
// dev:auth_system:in_progress        // Status update
```

### Working with Tasks

1. **Create Task**: Add a comment in format `// section:identifier: description`
2. **Update Status**: Use format `// section:identifier:status`
3. **Navigation**: F12 to go to definition, Shift+F12 to find references

### VSCode Panels

**Task Panel:**
- View all tasks by sections
- Update status via context menu
- Navigate to task code

**Notes Panel:**
- Create notes for future tasks
- Generate links for code insertion
- Automatic conversion to tasks

### Status Bar
- Task counters by status: ○3 ◐1 ●5
- Current task context when editing
- Quick access to search

### Commands

| Command | Hotkeys | Description |
|---------|---------|-------------|
| Create Note | Ctrl+Shift+T | Create a note |
| Go to Task Definition | F12 | Jump to task definition |
| Find Task References | Shift+F12 | Find all task references |
| View All Tasks | Ctrl+Shift+A | Show all tasks |
| Search Tasks | Ctrl+Shift+F | Search across tasks |
| Task Dashboard | Ctrl+Shift+D | Open task dashboard |

### Color Indicators

Tasks are highlighted in the editor with different colors:
- **Todo** (○): Red (#ff6b6b) - new tasks
- **In Progress** (◐): Teal (#4ecdc4) - in progress
- **Done** (●): Blue (#45b7d1) - completed
- **Blocked** (◯): Yellow (#f9ca24) - blocked

### Workflow with Notes

1. Create a note via command or panel
2. Fill in title, description, section and suggested ID
3. Generate link via context menu
4. Insert link in code - note automatically becomes a task

### Project Monitoring

The system automatically:
- Scans files on changes
- Updates task index
- Synchronizes data between VSCode and backend
- Saves state in `.anchora/tasks.json`

### Configuration

In settings.json you can configure:
- File patterns for scanning
- Ignored directories
- Colors for task statuses
- Debug mode

```json
{
  "anchora.filePatterns": ["**/*.rs", "**/*.ts", "**/*.py"],
  "anchora.ignoredDirectories": ["target", "node_modules", ".git"],
  "anchora.decorationColors": {
    "todo": "#ff6b6b",
    "in_progress": "#4ecdc4"
  }
}
```

## Installation

1. Install the VSCode extension from the VSIX package
2. Open any project - Anchora will auto-initialize
3. Start adding tasks to your code comments

## Architecture

Anchora consists of two main components:
- **Backend**: High-performance Rust service for task parsing, storage, and file monitoring
- **VSCode Extension**: TypeScript-based frontend providing seamless editor integration
- **Communication**: JSON-RPC over stdin/stdout for real-time synchronization

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

## Contributing

This project is in active development. Contribution guidelines will be available soon.

## License

Apache License 2.0 - See [LICENSE](LICENSE) file for details.

**Attribution Requirements**: When using or redistributing this code, you must:
- Include the original copyright notice
- Include the NOTICE file with attribution details
- Clearly mark any modifications made to the original code
- Cannot use "Anchora Task Manager" name for endorsement without permission

---

*Anchora revolutionizes task management by bringing tasks directly into your code where they belong.*
