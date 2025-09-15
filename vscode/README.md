# Anchora - VSCode Extension

> [Русская версия / Russian Version](doc/README_RU_VSCODE.md)
## What is it?

Anchora is a VSCode extension that allows you to manage tasks directly in your code. Instead of external task trackers, you write tasks in code comments, and the extension automatically tracks and visualizes them.

## Core Features

### Creating Tasks in Comments
Write tasks directly in code using special syntax:
```
// section:task_id: task description
// dev:auth_system: Implement OAuth2 authentication
```

### Status Management
- **todo** (○) — tasks to be done
- **in_progress** (◐) — tasks in progress  
- **done** (●) — completed tasks
- **blocked** (◯) — blocked tasks

### Task Visualization
- Color indicators directly in the editor
- Task panel in the sidebar
- Statistics in the status bar
- Dashboard with overview of all tasks

### Search and Navigation
- Quick search for tasks by title and description
- Jump to task definition
- Find all mentions of a task in the project

## User Interface

### 1. Activity Bar Panel
**Icon**: Anchora in VSCode sidebar
- Tree view of all tasks by sections
- Group by status
- Quick filtering and search

### 2. Explorer Integration
- Tasks are displayed in the standard Explorer panel
- Context menu for working with tasks
- Status indicators next to files

### 3. Code Editor
- **Color indicators**: each task is highlighted by status color
- **Hover tooltips**: task details on hover
- **Quick actions**: change status via context menu

### 4. Status Bar
- Task counters by status: "5 todo, 3 in progress, 12 done"
- Current project context indicator
- Backend service connection status

### 5. Command Palette
All commands are available through `Ctrl+Shift+P`

## Hotkeys

| Action | Keys | Description |
|--------|------|-------------|
| **Create Task** | `Ctrl+Shift+T` | New task creation wizard |
| **Go to Definition** | `F12` | Jump to task creation location |
| **Find All References** | `Shift+F12` | Search for all task references |
| **View All Tasks** | `Ctrl+Shift+A` | Dashboard with statistics |
| **Search Tasks** | `Ctrl+Shift+F` | Advanced search |
| **Control Panel** | `Ctrl+Shift+D` | Quick actions |

## Main Commands

### Project Management
- **Scan Project** - find all tasks in files
- **Refresh Tasks** - reload data
- **Initialize Project** - setup Anchora in new project

### Working with Tasks
- **Create Task** - interactive creation wizard
- **Change Status** - switch between todo/in_progress/done/blocked
- **Find References** - search for all task mentions
- **Go to Task** - navigate to definition

### Viewing and Filtering
- **View by Status** - group tasks
- **Task Dashboard** - overall statistics and charts
- **Search** - search by title, description, ID

## Task Syntax

### Basic Format
```
// section:id: task description
// section:id:status: description
// section:id                  // simple reference
```

### Examples
```rust
// dev:auth_system: Implement OAuth2 system
// bugfix:login_error:todo: Fix login error
// dev:auth_system:in_progress: Working on tokens
// dev:auth_system             // reference to this task
```

## Installation

### From VSIX Package
```bash
code --install-extension anchora-0.1.0.vsix
```

### Project Initialization
Happens automatically when opening a project. Creates `.anchora/` folder with `tasks.json` file.

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

## Configuration
You can configure file patterns, ignored folders, and status colors in VSCode settings.

## License
Apache License 2.0 - See [LICENSE](../LICENSE) for details.

**Attribution Requirements**: When using or redistributing this code, you must:
- Include the original copyright notice
- Include the NOTICE file with attribution details
- Clearly mark any modifications made to the original code
- Cannot use "Anchora Task Manager" name for endorsement without permission