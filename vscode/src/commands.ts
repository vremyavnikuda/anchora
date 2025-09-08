/**
 * Command handlers for Anchora VSCode extension
 * Implements all user-facing commands with proper error handling
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { JsonRpcClient } from './client';
import { TaskTreeProvider } from './taskProvider';
import {
    TaskStatus,
    CreateTaskParams,
    ScanProjectParams,
    createTaskId,
    createSectionName,
    AnchoraError
} from './types';

export class CommandHandler {
    constructor(
        private readonly client: JsonRpcClient,
        private readonly taskProvider: TaskTreeProvider,
        private readonly context: vscode.ExtensionContext
    ) { }

    /**
     * Register all commands
     */
    registerCommands(): void {
        const commands = [
            vscode.commands.registerCommand('anchora.createTask', () => this.createTask()),
            vscode.commands.registerCommand('anchora.refreshTasks', () => this.refreshTasks()),
            vscode.commands.registerCommand('anchora.scanProject', () => this.scanProject()),
            vscode.commands.registerCommand('anchora.goToTaskDefinition', () => this.goToTaskDefinition()),
            vscode.commands.registerCommand('anchora.findTaskReferences', (section?: string, taskId?: string) =>
                this.findTaskReferences(section, taskId)),
            vscode.commands.registerCommand('anchora.updateTaskStatus', (section?: string, taskId?: string) =>
                this.updateTaskStatus(section, taskId)),
            vscode.commands.registerCommand('anchora.showTaskReferences', (section: string, taskId: string) =>
                this.showTaskReferences(section, taskId)),
            vscode.commands.registerCommand('anchora.searchTasks', () => this.searchTasks()),
            vscode.commands.registerCommand('anchora.viewAllTaskLists', () => this.viewAllTaskLists()),
            vscode.commands.registerCommand('anchora.viewTasksByStatus', () => this.viewTasksByStatus()),
            vscode.commands.registerCommand('anchora.initializeProject', () => this.initializeProject())
        ];
        commands.forEach(command => this.context.subscriptions.push(command));
    }

    /**
     * Create a new task interactively
     */
    private async createTask(): Promise<void> {
        try {
            const section = await vscode.window.showInputBox({
                prompt: 'Enter section name (e.g., dev, ref, bug)',
                placeHolder: 'dev',
                validateInput: (value) => {
                    if (!value.trim()) {
                        return 'Section name cannot be empty';
                    }
                    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(value)) {
                        return 'Section name must start with letter or underscore and contain only alphanumeric characters and underscores';
                    }
                    return undefined;
                }
            });

            if (!section) return;
            const taskId = await vscode.window.showInputBox({
                prompt: 'Enter task ID',
                placeHolder: 'task_1',
                validateInput: (value) => {
                    if (!value.trim()) {
                        return 'Task ID cannot be empty';
                    }
                    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(value)) {
                        return 'Task ID must start with letter or underscore and contain only alphanumeric characters and underscores';
                    }
                    return undefined;
                }
            });
            if (!taskId) return;
            const title = await vscode.window.showInputBox({
                prompt: 'Enter task title/description',
                placeHolder: 'Add new feature for user authentication',
                validateInput: (value) => {
                    if (!value.trim()) {
                        return 'Task title cannot be empty';
                    }
                    return undefined;
                }
            });
            if (!title) return;
            const description = await vscode.window.showInputBox({
                prompt: 'Enter detailed description (optional)',
                placeHolder: 'Detailed implementation notes...'
            });
            const validSection = createSectionName(section);
            const validTaskId = createTaskId(taskId);
            if (!validSection || !validTaskId) {
                throw new AnchoraError('Invalid section name or task ID');
            }
            const params: CreateTaskParams = {
                section: validSection,
                task_id: validTaskId,
                title: title.trim(),
                ...(description?.trim() && { description: description.trim() })
            };
            const result = await this.client.createTask(params);
            if (result.success) {
                vscode.window.showInformationMessage(`Task created: ${section}:${taskId}`);
                await this.taskProvider.loadTasks();
                const insertReference = await vscode.window.showQuickPick(
                    ['Yes', 'No'],
                    { placeHolder: 'Insert task reference at current cursor position?' }
                );
                if (insertReference === 'Yes') {
                    await this.insertTaskReference(section, taskId);
                }
            } else {
                vscode.window.showErrorMessage(`Failed to create task: ${result.message}`);
            }
        } catch (error) {
            this.handleError('create task', error);
        }
    }

    /**
     * Refresh tasks from backend
     */
    private async refreshTasks(): Promise<void> {
        try {
            await this.taskProvider.loadTasks();
            vscode.window.showInformationMessage('Tasks refreshed');
        } catch (error) {
            this.handleError('refresh tasks', error);
        }
    }

    /**
     * Scan project for tasks
     */
    private async scanProject(): Promise<void> {
        try {
            const config = vscode.workspace.getConfiguration('anchora');
            const filePatterns = config.get<string[]>('filePatterns') || [];
            if (!vscode.workspace.workspaceFolders?.[0]) {
                vscode.window.showErrorMessage('No workspace folder open');
                return;
            }
            const workspacePath = vscode.workspace.workspaceFolders[0].uri.fsPath;
            const params: ScanProjectParams = {
                workspace_path: workspacePath,
                file_patterns: filePatterns
            };
            vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Scanning project for tasks...',
                cancellable: false
            }, async () => {
                const result = await this.client.scanProject(params);
                await this.taskProvider.loadTasks();
                const message = `Scan complete: ${result.files_scanned} files scanned, ${result.tasks_found} tasks found`;
                if (result.errors.length > 0) {
                    vscode.window.showWarningMessage(`${message}. ${result.errors.length} errors occurred.`);
                    console.warn('Scan errors:', result.errors);
                } else {
                    vscode.window.showInformationMessage(message);
                }
            });
        } catch (error) {
            this.handleError('scan project', error);
        }
    }

    /**
     * Go to task definition (first occurrence)
     */
    private async goToTaskDefinition(): Promise<void> {
        try {
            const taskRef = await this.getCurrentTaskReference();
            if (!taskRef) return;
            const references = await this.client.findTaskReferences({
                section: taskRef.section,
                task_id: taskRef.taskId
            });
            if (references.length === 0) {
                vscode.window.showInformationMessage(`No references found for task ${taskRef.section}:${taskRef.taskId}`);
                return;
            }
            const firstRef = references[0];
            if (firstRef) {
                await this.openFileAtLine(firstRef.file_path, firstRef.line);
            }
        } catch (error) {
            this.handleError('go to task definition', error);
        }
    }

    /**
     * Find all task references
     */
    private async findTaskReferences(section?: string, taskId?: string): Promise<void> {
        try {
            let taskRef: { section: string; taskId: string } | null = null;
            if (section && taskId) {
                taskRef = { section, taskId };
            } else {
                taskRef = await this.getCurrentTaskReference();
            }
            if (!taskRef) return;
            await this.showTaskReferences(taskRef.section, taskRef.taskId);
        } catch (error) {
            this.handleError('find task references', error);
        }
    }

    /**
     * Show task references in a dedicated view
     */
    private async showTaskReferences(section: string, taskId: string): Promise<void> {
        try {
            const references = await this.client.findTaskReferences({
                section,
                task_id: taskId
            });
            if (references.length === 0) {
                vscode.window.showInformationMessage(`No references found for task ${section}:${taskId}`);
                return;
            }
            const items = references.map(ref => ({
                label: `${this.getFileName(ref.file_path)}:${ref.line}`,
                description: ref.note || '',
                detail: ref.file_path,
                reference: ref
            }));
            const selected = await vscode.window.showQuickPick(items, {
                placeHolder: `References for ${section}:${taskId} (${references.length} found)`,
                matchOnDescription: true,
                matchOnDetail: true
            });
            if (selected) {
                await this.openFileAtLine(selected.reference.file_path, selected.reference.line);
            }
        } catch (error) {
            this.handleError('show task references', error);
        }
    }

    /**
     * Update task status interactively
     */
    private async updateTaskStatus(section?: string, taskId?: string): Promise<void> {
        try {
            let taskRef: { section: string; taskId: string } | null = null;
            if (section && taskId) {
                taskRef = { section, taskId };
            } else {
                taskRef = await this.getCurrentTaskReference();
            }
            if (!taskRef) return;
            const statusOptions: { label: string; description: string; status: TaskStatus }[] = [
                { label: '○ Todo', description: 'Mark as todo', status: 'todo' },
                { label: '◐ In Progress', description: 'Mark as in progress', status: 'in_progress' },
                { label: '● Done', description: 'Mark as completed', status: 'done' },
                { label: '◯ Blocked', description: 'Mark as blocked', status: 'blocked' }
            ];
            const selected = await vscode.window.showQuickPick(statusOptions, {
                placeHolder: `Update status for ${taskRef.section}:${taskRef.taskId}`
            });
            if (selected) {
                await this.taskProvider.updateTaskStatus(taskRef.section, taskRef.taskId, selected.status);
            }
        } catch (error) {
            this.handleError('update task status', error);
        }
    }

    /**
     * Search tasks interactively
     */
    private async searchTasks(): Promise<void> {
        try {
            const query = await vscode.window.showInputBox({
                prompt: 'Search tasks by title, description, or ID',
                placeHolder: 'Enter search query...'
            });
            if (!query) return;
            const results = this.taskProvider.searchTasks(query);
            if (results.length === 0) {
                vscode.window.showInformationMessage(`No tasks found matching "${query}"`);
                return;
            }
            const items = results.map(task => ({
                label: task.label,
                description: task.status ? `Status: ${task.status}` : '',
                detail: task.description || '',
                task
            }));
            const selected = await vscode.window.showQuickPick(items, {
                placeHolder: `Search results for "${query}" (${results.length} found)`,
                matchOnDescription: true,
                matchOnDetail: true
            });
            if (selected && selected.task.section && selected.task.taskId) {
                await this.showTaskReferences(selected.task.section, selected.task.taskId);
            }
        } catch (error) {
            this.handleError('search tasks', error);
        }
    }

    /**
     * Get current task reference from cursor position
     */
    private async getCurrentTaskReference(): Promise<{ section: string; taskId: string } | null> {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor');
            return null;
        }
        const line = editor.document.lineAt(editor.selection.active.line);
        const taskMatch = line.text.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*)/);
        if (!taskMatch) {
            vscode.window.showErrorMessage('No task reference found at current line');
            return null;
        }
        return {
            section: taskMatch[1] || '',
            taskId: taskMatch[2] || ''
        };
    }

    /**
     * Insert task reference at current cursor position
     */
    private async insertTaskReference(section: string, taskId: string): Promise<void> {
        const editor = vscode.window.activeTextEditor;
        if (!editor) return;
        const reference = `// ${section}:${taskId}`;
        await editor.edit(editBuilder => {
            editBuilder.insert(editor.selection.active, reference);
        });
    }

    /**
     * Open file at specific line
     */
    private async openFileAtLine(filePath: string, line: number): Promise<void> {
        const uri = vscode.Uri.file(filePath);
        const document = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(document);
        const position = new vscode.Position(Math.max(0, line - 1), 0);
        editor.selection = new vscode.Selection(position, position);
        editor.revealRange(new vscode.Range(position, position));
    }

    /**
     * Extract filename from path
     */
    private getFileName(filePath: string): string {
        return filePath.split(/[/\\]/).pop() || filePath;
    }

    /**
     * Handle errors with consistent messaging
     */
    private handleError(operation: string, error: unknown): void {
        const message = error instanceof Error ? error.message : String(error);
        vscode.window.showErrorMessage(`Failed to ${operation}: ${message}`);
        console.error(`Failed to ${operation}:`, error);
    }

    /**
     * View all task lists with comprehensive overview
     */
    private async viewAllTaskLists(): Promise<void> {
        try {
            if (!this.client.isConnected()) {
                await this.client.connect();
            }
            const projectData = await this.client.getTasks();
            if (!projectData || Object.keys(projectData).length === 0) {
                vscode.window.showInformationMessage('No tasks found in the project.');
                return;
            }
            const taskOverview = this.createTaskOverview(projectData);
            await this.showTaskOverviewPanel(taskOverview);
        } catch (error) {
            this.handleError('view all task lists', error);
        }
    }

    /**
     * View tasks grouped by status
     */
    private async viewTasksByStatus(): Promise<void> {
        try {
            if (!this.client.isConnected()) {
                await this.client.connect();
            }
            const projectData = await this.client.getTasks();
            if (!projectData || Object.keys(projectData).length === 0) {
                vscode.window.showInformationMessage('No tasks found in the project.');
                return;
            }
            const statusGroups = this.groupTasksByStatus(projectData);
            const statusOptions = Object.entries(statusGroups).map(([status, tasks]) => ({
                label: `${this.getStatusIcon(status as TaskStatus)} ${status.toUpperCase()}`,
                description: `${tasks.length} tasks`,
                tasks: tasks
            }));
            const selectedStatus = await vscode.window.showQuickPick(statusOptions, {
                placeHolder: 'Select status to view tasks',
                matchOnDescription: true
            });
            if (selectedStatus) {
                await this.showTasksForStatus(selectedStatus.tasks);
            }
        } catch (error) {
            this.handleError('view tasks by status', error);
        }
    }

    /**
     * Create comprehensive task overview
     */
    private createTaskOverview(projectData: any): any {
        const overview = {
            totalSections: 0,
            totalTasks: 0,
            statusCounts: { todo: 0, in_progress: 0, done: 0, blocked: 0 },
            sections: [] as any[]
        };
        for (const [sectionName, section] of Object.entries(projectData)) {
            overview.totalSections++;
            const sectionData = {
                name: sectionName,
                taskCount: 0,
                tasks: [] as any[]
            };
            for (const [taskId, task] of Object.entries(section as any)) {
                overview.totalTasks++;
                sectionData.taskCount++;
                const taskData = task as any;
                if (taskData.status in overview.statusCounts) {
                    (overview.statusCounts as any)[taskData.status]++;
                }
                sectionData.tasks.push({
                    id: taskId,
                    title: taskData.title,
                    status: taskData.status,
                    description: taskData.description,
                    fileCount: Object.keys(taskData.files || {}).length,
                    created: taskData.created,
                    updated: taskData.updated
                });
            }
            overview.sections.push(sectionData);
        }
        return overview;
    }

    /**
     * Group tasks by status
     */
    private groupTasksByStatus(projectData: any): Record<TaskStatus, any[]> {
        const groups: Record<TaskStatus, any[]> = {
            todo: [],
            in_progress: [],
            done: [],
            blocked: []
        };
        for (const [sectionName, section] of Object.entries(projectData)) {
            for (const [taskId, task] of Object.entries(section as any)) {
                const taskData = task as any;
                const status = taskData.status as TaskStatus;
                if (status in groups) {
                    groups[status].push({
                        section: sectionName,
                        id: taskId,
                        title: taskData.title,
                        description: taskData.description,
                        status: status
                    });
                }
            }
        }
        return groups;
    }

    /**
     * Show task overview in a webview panel
     */
    private async showTaskOverviewPanel(overview: any): Promise<void> {
        const panel = vscode.window.createWebviewPanel(
            'anchoraTaskOverview',
            'Anchora Task Overview',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );
        panel.webview.html = this.getTaskOverviewHtml(overview);
    }

    /**
     * Generate HTML for task overview
     */
    private getTaskOverviewHtml(overview: any): string {
        return `<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Anchora Task Overview</title>
            <style>
                body {
                    font-family: var(--vscode-font-family);
                    color: var(--vscode-foreground);
                    background-color: var(--vscode-editor-background);
                    padding: 20px;
                }
                .header {
                    border-bottom: 2px solid var(--vscode-panel-border);
                    padding-bottom: 15px;
                    margin-bottom: 20px;
                }
                .stats {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 15px;
                    margin-bottom: 30px;
                }
                .stat-card {
                    background: var(--vscode-editor-widget-background);
                    border: 1px solid var(--vscode-panel-border);
                    border-radius: 5px;
                    padding: 15px;
                    text-align: center;
                }
                .stat-number {
                    font-size: 2em;
                    font-weight: bold;
                    margin-bottom: 5px;
                }
                .section {
                    background: var(--vscode-editor-widget-background);
                    border: 1px solid var(--vscode-panel-border);
                    border-radius: 5px;
                    margin-bottom: 20px;
                    padding: 15px;
                }
                .section-header {
                    font-size: 1.2em;
                    font-weight: bold;
                    margin-bottom: 10px;
                    color: var(--vscode-textLink-foreground);
                }
                .task {
                    background: var(--vscode-input-background);
                    border: 1px solid var(--vscode-input-border);
                    border-radius: 3px;
                    padding: 10px;
                    margin-bottom: 8px;
                }
                .task-title {
                    font-weight: bold;
                    margin-bottom: 5px;
                }
                .task-meta {
                    font-size: 0.9em;
                    color: var(--vscode-descriptionForeground);
                }
                .status-todo { color: #ff6b6b; }
                .status-in_progress { color: #4ecdc4; }
                .status-done { color: #45b7d1; }
                .status-blocked { color: #f9ca24; }
            </style>
        </head>
        <body>
            <div class="header">
                <h1>📋 Anchora Task Overview</h1>
                <p>Complete overview of all tasks in your project</p>
            </div>
            
            <div class="stats">
                <div class="stat-card">
                    <div class="stat-number">${overview.totalSections}</div>
                    <div>Sections</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number">${overview.totalTasks}</div>
                    <div>Total Tasks</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number status-todo">${overview.statusCounts.todo}</div>
                    <div>Todo</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number status-in_progress">${overview.statusCounts.in_progress}</div>
                    <div>In Progress</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number status-done">${overview.statusCounts.done}</div>
                    <div>Done</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number status-blocked">${overview.statusCounts.blocked}</div>
                    <div>Blocked</div>
                </div>
            </div>
            
            <h2>📁 Sections & Tasks</h2>
            ${overview.sections.map((section: any) => `
                <div class="section">
                    <div class="section-header">${section.name} (${section.taskCount} tasks)</div>
                    ${section.tasks.map((task: any) => `
                        <div class="task">
                            <div class="task-title">
                                <span class="status-${task.status}">${this.getStatusIcon(task.status)}</span>
                                ${task.id}: ${task.title}
                            </div>
                            ${task.description ? `<div style="margin: 5px 0; font-style: italic;">${task.description}</div>` : ''}
                            <div class="task-meta">
                                Status: <span class="status-${task.status}">${task.status}</span> |
                                Files: ${task.fileCount} |
                                Created: ${new Date(task.created).toLocaleDateString()}
                                ${task.updated !== task.created ? ` | Updated: ${new Date(task.updated).toLocaleDateString()}` : ''}
                            </div>
                        </div>
                    `).join('')}
                </div>
            `).join('')}
        </body>
        </html>`;
    }

    /**
     * Show tasks for a specific status
     */
    private async showTasksForStatus(tasks: any[]): Promise<void> {
        const taskItems = tasks.map(task => ({
            label: `${task.section}:${task.id}`,
            description: task.title,
            detail: task.description || '',
            task: task
        }));
        const selectedTask = await vscode.window.showQuickPick(taskItems, {
            placeHolder: `Select a task to view references (${tasks.length} tasks)`,
            matchOnDescription: true,
            matchOnDetail: true
        });
        if (selectedTask) {
            await this.showTaskReferences(selectedTask.task.section, selectedTask.task.id);
        }
    }

    /**
     * Get status icon
     */
    private getStatusIcon(status: TaskStatus): string {
        const icons: Record<TaskStatus, string> = {
            'todo': '○',
            'in_progress': '◐',
            'done': '●',
            'blocked': '◯'
        };
        return icons[status] || '○';
    }

    /**
     * Initialize Anchora project in current workspace
     */
    private async initializeProject(): Promise<void> {
        try {
            if (!vscode.workspace.workspaceFolders?.[0]) {
                vscode.window.showErrorMessage('No workspace folder open');
                return;
            }
            const workspacePath = vscode.workspace.workspaceFolders[0].uri.fsPath;
            const anchoraDir = vscode.Uri.file(path.join(workspacePath, '.anchora'));
            try {
                await vscode.workspace.fs.stat(anchoraDir);
                vscode.window.showInformationMessage('Anchora project is already initialized!');
                return;
            } catch { }
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Initializing Anchora project...',
                cancellable: false
            }, async (progress) => {
                progress.report({ increment: 20, message: 'Creating .anchora directory...' });
                await vscode.workspace.fs.createDirectory(anchoraDir);
                progress.report({ increment: 40, message: 'Creating tasks.json...' });
                const tasksFile = vscode.Uri.file(path.join(workspacePath, '.anchora', 'tasks.json'));
                const initialTasksData = {
                    meta: {
                        version: "1.0.0",
                        created: new Date().toISOString(),
                        last_updated: new Date().toISOString(),
                        project_name: path.basename(workspacePath)
                    },
                    sections: {},
                    index: {
                        files: {},
                        tasks_by_status: {
                            todo: [],
                            in_progress: [],
                            done: [],
                            blocked: []
                        }
                    }
                };
                const tasksContent = JSON.stringify(initialTasksData, null, 2);
                await vscode.workspace.fs.writeFile(tasksFile, Buffer.from(tasksContent, 'utf8'));
                progress.report({ increment: 40, message: 'Refreshing views...' });
                await this.taskProvider.loadTasks();
                await vscode.commands.executeCommand('setContext', 'workspaceHasAnchoraProject', true);
            });
            vscode.window.showInformationMessage(
                'Anchora project initialized successfully! You can now start managing tasks.',
                'Create First Task',
                'Open Task Dashboard'
            ).then(selection => {
                if (selection === 'Create First Task') {
                    vscode.commands.executeCommand('anchora.createTask');
                } else if (selection === 'Open Task Dashboard') {
                    vscode.commands.executeCommand('anchora.openTaskDashboard');
                }
            });

        } catch (error) {
            this.handleError('initialize project', error);
        }
    }
}