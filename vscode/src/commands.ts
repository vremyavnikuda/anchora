/**
 * Command handlers for Anchora VSCode extension
 * Implements all user-facing commands with proper error handling
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { JsonRpcClient, setDebugMode, isDebugMode } from './client';
import { TaskTreeProvider } from './taskProvider';
import { NoteTreeProvider } from './noteProvider';
import {
    TaskStatus,
    ScanProjectParams,
    DeleteTaskParams,
    Note
} from './types';

let commandOutputChannel: vscode.OutputChannel | null = null;

function getCommandOutputChannel(): vscode.OutputChannel {
    if (!commandOutputChannel) {
        commandOutputChannel = vscode.window.createOutputChannel('Anchora Commands');
    }
    return commandOutputChannel;
}

function logCommandInfo(message: string, data?: any): void {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] COMMAND: ${message}`;
    console.log(logMessage);
    getCommandOutputChannel().appendLine(logMessage);
    if (data !== undefined && isDebugMode()) {
        const dataStr = typeof data === 'object' ? JSON.stringify(data, null, 2) : String(data);
        getCommandOutputChannel().appendLine(`Data: ${dataStr}`);
        console.log('Command data:', data);
    }
}

function logCommandError(message: string, error?: any, context?: any): void {
    const timestamp = new Date().toISOString();
    const errorDetails = error ? ` - ${error instanceof Error ? error.message : String(error)}` : '';
    const errorStack = error instanceof Error ? error.stack : '';
    const logMessage = `[${timestamp}] COMMAND ERROR: ${message}${errorDetails}`;
    console.error(logMessage);
    getCommandOutputChannel().appendLine(logMessage);
    if (context && isDebugMode()) {
        const contextStr = typeof context === 'object' ? JSON.stringify(context, null, 2) : String(context);
        getCommandOutputChannel().appendLine(`Context: ${contextStr}`);
        console.error('Error context:', context);
    }
    if (errorStack && isDebugMode()) {
        getCommandOutputChannel().appendLine(`Stack trace: ${errorStack}`);
        console.error('Full error object:', error);
    }
    if (isDebugMode()) {
        getCommandOutputChannel().show(true);
    }
}

function logCommandDebug(message: string, data?: any): void {
    if (!isDebugMode()) return;
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] COMMAND DEBUG: ${message}`;
    console.debug(logMessage);
    getCommandOutputChannel().appendLine(logMessage);
    if (data !== undefined) {
        const dataStr = typeof data === 'object' ? JSON.stringify(data, null, 2) : String(data);
        getCommandOutputChannel().appendLine(`Debug data: ${dataStr}`);
        console.debug('Debug data:', data);
    }
}

export class CommandHandler {
    constructor(
        private readonly client: JsonRpcClient,
        private readonly taskProvider: TaskTreeProvider,
        private readonly noteProvider: NoteTreeProvider,
        private readonly context: vscode.ExtensionContext
    ) {
        logCommandInfo('CommandHandler initialized');
        const config = vscode.workspace.getConfiguration('anchora');
        const debugEnabled = config.get<boolean>('debugMode', false);
        if (debugEnabled) {
            setDebugMode(true);
            logCommandInfo('Debug mode enabled from configuration');
        }
    }

    /**
     * Register all commands
     */
    registerCommands(): void {
        logCommandInfo('Registering commands...');
        const commands = [
            vscode.commands.registerCommand('anchora.createTask', () => this.createNote()),
            vscode.commands.registerCommand('anchora.refreshTasks', () => this.refreshTasks()),
            vscode.commands.registerCommand('anchora.scanProject', () => this.scanProject()),
            vscode.commands.registerCommand('anchora.goToTaskDefinition', () => this.goToTaskDefinition()),
            vscode.commands.registerCommand('anchora.findTaskReferences', (section?: string, taskId?: string) =>
                this.findTaskReferences(section, taskId)),
            vscode.commands.registerCommand('anchora.updateTaskStatus', (section?: string, taskId?: string) =>
                this.updateTaskStatus(section, taskId)),
            vscode.commands.registerCommand('anchora.deleteTask', (section?: string, taskId?: string) =>
                this.deleteTask(section, taskId)),
            vscode.commands.registerCommand('anchora.showTaskReferences', (section: string, taskId: string) =>
                this.showTaskReferences(section, taskId)),
            vscode.commands.registerCommand('anchora.searchTasks', () => this.searchTasks()),
            vscode.commands.registerCommand('anchora.viewAllTaskLists', () => this.viewAllTaskLists()),
            vscode.commands.registerCommand('anchora.viewTasksByStatus', () => this.viewTasksByStatus()),
            vscode.commands.registerCommand('anchora.initializeProject', () => this.initializeProject()),
            vscode.commands.registerCommand('anchora.toggleDebugMode', () => this.toggleDebugMode()),
            vscode.commands.registerCommand('anchora.showOutputChannel', () => this.showOutputChannel()),
            vscode.commands.registerCommand('anchora.testErrorHandling', () => this.testErrorHandling()),
            vscode.commands.registerCommand('anchora.createNote', () => this.createNote()),
            vscode.commands.registerCommand('anchora.refreshNotes', () => this.refreshNotes()),
            vscode.commands.registerCommand('anchora.generateTaskLink', (noteIdOrItem: string | any) => this.generateTaskLink(noteIdOrItem)),
            vscode.commands.registerCommand('anchora.viewNote', (noteIdOrItem: string | any) => this.viewNote(noteIdOrItem)),
            vscode.commands.registerCommand('anchora.deleteNote', (noteIdOrItem: string | any) => this.deleteNote(noteIdOrItem))
        ];
        commands.forEach(command => this.context.subscriptions.push(command));
        logCommandInfo(`Registered ${commands.length} commands successfully`);
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
     * Delete task interactively with confirmation
     * Also removes all task anchors from source code files
     */
    private async deleteTask(section?: string, taskId?: string): Promise<void> {
        try {
            let taskRef: { section: string; taskId: string } | null = null;
            if (section && taskId) {
                taskRef = { section, taskId };
            } else {
                taskRef = await this.getCurrentTaskReference();
                if (!taskRef) {
                    taskRef = await this.selectTaskInteractively('Select task to delete');
                }
            }
            if (!taskRef) return;
            const confirmation = await vscode.window.showWarningMessage(
                `Are you sure you want to delete task ${taskRef.section}:${taskRef.taskId}?\n\nThis will remove the task from the list AND delete all task anchors from your code files.`,
                { modal: true },
                'Delete Task & Anchors',
                'Cancel'
            );
            if (confirmation !== 'Delete Task & Anchors') {
                return;
            }
            logCommandInfo(`Starting deletion of task ${taskRef.section}:${taskRef.taskId} including anchors`);
            let taskReferences: ReadonlyArray<any> = [];
            try {
                taskReferences = await this.client.findTaskReferences({
                    section: taskRef.section,
                    task_id: taskRef.taskId
                });
                logCommandDebug(`Found ${taskReferences.length} task references to remove`, taskReferences);
            } catch (error) {
                logCommandError('Failed to find task references', error);
            }
            if (taskReferences.length > 0) {
                const removedAnchors = await this.removeTaskAnchors(taskRef.section, taskRef.taskId, taskReferences);
                logCommandInfo(`Removed ${removedAnchors} task anchors from source files`);
            }
            const params: DeleteTaskParams = {
                section: taskRef.section,
                task_id: taskRef.taskId
            };
            const result = await this.client.deleteTask(params);
            if (result.success) {
                vscode.window.showInformationMessage(
                    `Task deleted: ${taskRef.section}:${taskRef.taskId}${taskReferences.length > 0 ? ` (${taskReferences.length} anchors removed)` : ''}`
                );
                await this.taskProvider.loadTasks();
            } else {
                vscode.window.showErrorMessage(`Failed to delete task: ${result.message}`);
            }
        } catch (error) {
            this.handleError('delete task', error);
        }
    }

    /**
     * Remove task anchors from source code files
     */
    private async removeTaskAnchors(section: string, taskId: string, taskReferences: ReadonlyArray<any>): Promise<number> {
        let removedCount = 0;
        const fileGroups = new Map<string, number[]>();
        for (const ref of taskReferences) {
            if (!fileGroups.has(ref.file_path)) {
                fileGroups.set(ref.file_path, []);
            }
            fileGroups.get(ref.file_path)!.push(ref.line);
        }
        logCommandDebug(`Processing ${fileGroups.size} files for anchor removal`);
        for (const [filePath, lines] of fileGroups) {
            try {
                const removed = await this.removeTaskAnchorsFromFile(section, taskId, filePath, lines);
                removedCount += removed;
                logCommandDebug(`Removed ${removed} anchors from ${filePath}`);
            } catch (error) {
                logCommandError(`Failed to remove anchors from ${filePath}`, error, {
                    filePath,
                    lines,
                    section,
                    taskId
                });
            }
        }

        return removedCount;
    }

    /**
     * Remove task anchors from a specific file
     */
    private async removeTaskAnchorsFromFile(section: string, taskId: string, filePath: string, lineNumbers: number[]): Promise<number> {
        try {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                throw new Error('No workspace folder available');
            }
            let absolutePath: string;
            if (path.isAbsolute(filePath)) {
                absolutePath = filePath;
            } else {
                absolutePath = path.join(workspaceFolder.uri.fsPath, filePath);
            }
            const uri = vscode.Uri.file(absolutePath);
            const document = await vscode.workspace.openTextDocument(uri);
            const taskPatterns = [
                new RegExp(`^\\s*//\\s*${section}:${taskId}:[a-zA-Z_][a-zA-Z0-9_]*:\\s+.+$`),
                new RegExp(`^\\s*//\\s*${section}:${taskId}:\\s+.+$`),
                new RegExp(`^\\s*//\\s*${section}:${taskId}:(todo|in_progress|inprogress|progress|done|completed|complete|blocked|block)\\s*$`, 'i'),
                new RegExp(`^\\s*//\\s*${section}:${taskId}:[a-zA-Z0-9_]+\\s*$`),
                new RegExp(`^\\s*//\\s*${section}:${taskId}\\s*$`)
            ];
            const linesToRemove = lineNumbers
                .map(line => line - 1)
                .filter(line => line >= 0 && line < document.lineCount)
                .sort((a, b) => b - a);
            if (linesToRemove.length === 0) {
                logCommandDebug(`No valid lines to remove in ${filePath}`);
                return 0;
            }
            const validLinesToRemove: number[] = [];
            for (const lineIndex of linesToRemove) {
                const lineText = document.lineAt(lineIndex).text;
                const isTaskAnchor = taskPatterns.some(pattern => pattern.test(lineText.trim()));
                if (isTaskAnchor) {
                    validLinesToRemove.push(lineIndex);
                    logCommandDebug(`Line ${lineIndex + 1} matches task anchor pattern: ${lineText.trim()}`);
                } else {
                    logCommandDebug(`Line ${lineIndex + 1} does not match task anchor pattern, skipping: ${lineText.trim()}`);
                }
            }
            if (validLinesToRemove.length === 0) {
                logCommandDebug(`No valid task anchors found to remove in ${filePath}`);
                return 0;
            }
            const editor = await vscode.window.showTextDocument(document, { preview: false, preserveFocus: true });
            const success = await editor.edit(editBuilder => {
                for (const lineIndex of validLinesToRemove) {
                    const line = document.lineAt(lineIndex);
                    const range = line.rangeIncludingLineBreak;
                    editBuilder.delete(range);
                    logCommandDebug(`Removing line ${lineIndex + 1}: ${line.text.trim()}`);
                }
            });
            if (success) {
                await document.save();
                logCommandInfo(`Successfully removed ${validLinesToRemove.length} task anchors from ${filePath}`);
                return validLinesToRemove.length;
            } else {
                throw new Error('Failed to apply edits to document');
            }
        } catch (error) {
            logCommandError(`Failed to remove task anchors from file ${filePath}`, error);
            throw error;
        }
    }

    /**
     * Search tasks interactively with server-side performance
     */
    private async searchTasks(): Promise<void> {
        try {
            const query = await vscode.window.showInputBox({
                prompt: 'Search tasks by title, description, or ID',
                placeHolder: 'Enter search query...'
            });
            if (!query) return;

            // Use server-side search for better performance
            const results = await this.taskProvider.searchTasks(query);

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
     * Supports all task reference patterns that the backend recognizes
     */
    private async getCurrentTaskReference(): Promise<{ section: string; taskId: string } | null> {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            // Don't show error message here since this method is now used as a fallback
            return null;
        }
        const line = editor.document.lineAt(editor.selection.active.line);
        const lineText = line.text.trim();
        let taskMatch = lineText.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):\s+(.+)/);
        if (taskMatch) {
            return {
                section: taskMatch[1] || '',
                taskId: taskMatch[2] || ''
            };
        }
        taskMatch = lineText.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):\s+(.+)/);
        if (taskMatch) {
            return {
                section: taskMatch[1] || '',
                taskId: taskMatch[2] || ''
            };
        }
        taskMatch = lineText.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):(todo|in_progress|inprogress|progress|done|completed|complete|blocked|block)\s*$/i);
        if (taskMatch) {
            return {
                section: taskMatch[1] || '',
                taskId: taskMatch[2] || ''
            };
        }
        taskMatch = lineText.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z0-9_]+)\s*$/);
        if (taskMatch && taskMatch[3]) {
            const thirdPart = taskMatch[3].toLowerCase();
            const statusKeywords = ['todo', 'in_progress', 'inprogress', 'progress', 'done', 'completed', 'complete', 'blocked', 'block'];
            if (!statusKeywords.includes(thirdPart)) {
                return {
                    section: taskMatch[1] || '',
                    taskId: taskMatch[2] || ''
                };
            }
        }
        taskMatch = lineText.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*)\s*$/);
        if (taskMatch) {
            return {
                section: taskMatch[1] || '',
                taskId: taskMatch[2] || ''
            };
        }
        return null;
    }

    /**
     * Let user select a task interactively from available tasks
     */
    private async selectTaskInteractively(prompt: string): Promise<{ section: string; taskId: string } | null> {
        try {
            if (!this.client.isConnected()) {
                await this.client.connect();
            }
            const projectData = await this.client.getTasks();
            if (!projectData) {
                vscode.window.showInformationMessage('No project data available.');
                return null;
            }
            if (!projectData.sections || Object.keys(projectData.sections).length === 0) {
                vscode.window.showInformationMessage('No tasks found in the project.');
                return null;
            }
            const taskItems: Array<{
                label: string;
                description: string;
                detail: string;
                section: string;
                taskId: string;
            }> = [];
            for (const [sectionName, section] of Object.entries(projectData.sections)) {
                for (const [taskId, task] of Object.entries(section as any)) {
                    const taskData = task as any;
                    taskItems.push({
                        label: `${sectionName}:${taskId}`,
                        description: taskData.title || 'No title',
                        detail: taskData.description || 'No description',
                        section: sectionName,
                        taskId: taskId
                    });
                }
            }
            if (taskItems.length === 0) {
                vscode.window.showInformationMessage('No tasks found in the project.');
                return null;
            }
            const selectedTask = await vscode.window.showQuickPick(taskItems, {
                placeHolder: prompt,
                matchOnDescription: true,
                matchOnDetail: true
            });
            if (selectedTask) {
                return {
                    section: selectedTask.section,
                    taskId: selectedTask.taskId
                };
            }
            return null;
        } catch (error) {
            this.handleError('select task interactively', error);
            return null;
        }
    }

    /**
     * Open file at specific line
     */
    private async openFileAtLine(filePath: string, line: number): Promise<void> {
        try {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                throw new Error('No workspace folder available');
            }
            let absolutePath: string;
            if (path.isAbsolute(filePath)) {
                absolutePath = filePath;
            } else {
                absolutePath = path.join(workspaceFolder.uri.fsPath, filePath);
            }
            const uri = vscode.Uri.file(absolutePath);
            const document = await vscode.workspace.openTextDocument(uri);
            const editor = await vscode.window.showTextDocument(document);
            const position = new vscode.Position(Math.max(0, line - 1), 0);
            editor.selection = new vscode.Selection(position, position);
            editor.revealRange(new vscode.Range(position, position));
        } catch (error) {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            const debugInfo = {
                originalPath: filePath,
                workspaceRoot: workspaceFolder?.uri.fsPath || 'No workspace',
                isAbsolute: path.isAbsolute(filePath),
                line: line
            };
            logCommandError('Failed to open file', error, debugInfo);
            vscode.window.showErrorMessage(
                `Failed to open file: ${path.basename(filePath)}. Check if the file exists.`
            );
        }
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
    private handleError(operation: string, error: unknown, context?: any): void {
        const message = error instanceof Error ? error.message : String(error);
        const fullContext = {
            operation,
            timestamp: new Date().toISOString(),
            errorType: error instanceof Error ? error.constructor.name : typeof error,
            ...context
        };
        logCommandError(`Failed to ${operation}`, error, fullContext);
        vscode.window.showErrorMessage(`Failed to ${operation}: ${message}`);
        console.error(`Failed to ${operation}:`, error);
        if (isDebugMode()) {
            vscode.window.showErrorMessage(
                `Debug: ${operation} failed. Check Anchora Commands output channel for details.`,
                'Show Output'
            ).then(selection => {
                if (selection === 'Show Output') {
                    getCommandOutputChannel().show();
                }
            });
        }
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
            if (!projectData) {
                vscode.window.showInformationMessage('No project data available.');
                return;
            }
            // Allow overview to open even when no tasks exist
            // Users should be able to see the overview and potentially create new tasks
            const taskOverview = await this.getTaskOverview();
            await this.showTaskOverviewPanel(taskOverview);
        } catch (error) {
            this.handleError('view all task lists', error);
        }
    }

    /**
     * View tasks grouped by status using server-side filtering
     */
    private async viewTasksByStatus(): Promise<void> {
        try {
            if (!this.client.isConnected()) {
                await this.client.connect();
            }
            const statusGroups = await this.groupTasksByStatus();
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
     * Group tasks by status using server-side filtering for better performance
     */
    private async groupTasksByStatus(): Promise<Record<TaskStatus, any[]>> {
        const groups: Record<TaskStatus, any[]> = {
            todo: [],
            in_progress: [],
            done: [],
            blocked: []
        };

        try {
            // Use server-side filtered search for each status
            for (const status of ['todo', 'in_progress', 'done', 'blocked'] as TaskStatus[]) {
                const searchResult = await this.client.searchTasks({
                    query: '*', // Search all tasks
                    filters: {
                        statuses: [status],
                        include_descriptions: true
                    },
                    limit: 1000 // High limit to get all tasks
                });

                groups[status] = searchResult.tasks.map(task => ({
                    section: task.section,
                    id: task.task_id,
                    title: task.title,
                    description: task.description,
                    status: task.status
                }));
            }
        } catch (error) {
            logCommandError('Failed to group tasks by status using server-side filtering', error);
            // Return empty groups on error
        }

        return groups;
    }

    /**
     * Show task overview in a webview panel with note creation capability
     */
    private async showTaskOverviewPanel(overview: any, activeTab: string = 'overview'): Promise<void> {
        const panel = vscode.window.createWebviewPanel(
            'anchoraTaskOverview',
            'Anchora Task Overview',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true
            }
        );
        panel.webview.html = this.getTaskOverviewHtml(overview, activeTab);
        panel.webview.onDidReceiveMessage(
            async message => {
                switch (message.command) {
                    case 'createNote':
                        await this.handleWebviewCreateNote(message.data, panel);
                        break;
                    case 'refreshOverview':
                        const newOverview = await this.getTaskOverview();
                        panel.webview.html = this.getTaskOverviewHtml(newOverview, activeTab);
                        break;
                }
            },
            undefined,
            this.context.subscriptions
        );
    }

    /**
     * Generate HTML for task overview with note creation form
     */
    private getTaskOverviewHtml(overview: any, activeTab: string = 'overview'): string {
        return `<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Anchora</title>
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
                .tabs {
                    display: flex;
                    border-bottom: 1px solid var(--vscode-panel-border);
                    margin-bottom: 20px;
                }
                .tab {
                    padding: 10px 20px;
                    cursor: pointer;
                    background: var(--vscode-tab-inactiveBackground);
                    border: none;
                    color: var(--vscode-tab-inactiveForeground);
                    border-top-left-radius: 5px;
                    border-top-right-radius: 5px;
                    margin-right: 2px;
                }
                .tab.active {
                    background: var(--vscode-tab-activeBackground);
                    color: var(--vscode-tab-activeForeground);
                    border-bottom: 2px solid var(--vscode-focusBorder);
                }
                .tab-content {
                    display: none;
                }
                .tab-content.active {
                    display: block;
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
                
                /* Note creation form styles */
                .note-form {
                    background: var(--vscode-editor-widget-background);
                    border: 1px solid var(--vscode-panel-border);
                    border-radius: 5px;
                    padding: 20px;
                    margin-bottom: 20px;
                }
                .form-group {
                    margin-bottom: 15px;
                }
                .form-group label {
                    display: block;
                    margin-bottom: 5px;
                    font-weight: bold;
                    color: var(--vscode-foreground);
                }
                .form-group input,
                .form-group select,
                .form-group textarea {
                    width: 100%;
                    padding: 8px;
                    border: 1px solid var(--vscode-input-border);
                    background-color: var(--vscode-input-background);
                    color: var(--vscode-input-foreground);
                    border-radius: 3px;
                    font-family: inherit;
                }
                .form-group textarea {
                    min-height: 80px;
                    resize: vertical;
                }
                .form-group input:focus,
                .form-group select:focus,
                .form-group textarea:focus {
                    outline: none;
                    border-color: var(--vscode-focusBorder);
                }
                .btn {
                    background: var(--vscode-button-background);
                    color: var(--vscode-button-foreground);
                    border: none;
                    border-radius: 3px;
                    padding: 10px 15px;
                    cursor: pointer;
                    font-family: inherit;
                    margin-right: 10px;
                }
                .btn:hover {
                    background: var(--vscode-button-hoverBackground);
                }
                .btn-secondary {
                    background: var(--vscode-button-secondaryBackground);
                    color: var(--vscode-button-secondaryForeground);
                }
                .btn-secondary:hover {
                    background: var(--vscode-button-secondaryHoverBackground);
                }
                .error-message {
                    color: var(--vscode-errorForeground);
                    background: var(--vscode-inputValidation-errorBackground);
                    border: 1px solid var(--vscode-inputValidation-errorBorder);
                    padding: 8px;
                    border-radius: 3px;
                    margin-top: 10px;
                    display: none;
                }
                .success-message {
                    color: var(--vscode-testing-iconPassed);
                    background: var(--vscode-editor-widget-background);
                    border: 1px solid var(--vscode-testing-iconPassed);
                    padding: 8px;
                    border-radius: 3px;
                    margin-top: 10px;
                    display: none;
                }
            </style>
        </head>
        <body>
            <div class="header">
                <h1>Anchora</h1>
                <p>Complete overview of all tasks and notes in your project</p>
            </div>
            
            <div class="tabs">
                <button class="tab ${activeTab === 'overview' ? 'active' : ''}" onclick="showTab('overview')">📊 Task Overview</button>
                <button class="tab ${activeTab === 'create-note' ? 'active' : ''}" onclick="showTab('create-note')">📝 Create Note</button>
            </div>
            
            <!-- Task Overview Tab -->
            <div id="overview" class="tab-content ${activeTab === 'overview' ? 'active' : ''}">
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
                ${overview.sections.length === 0 ? `
                    <div class="section">
                        <div class="section-header">No tasks found</div>
                        <p style="color: var(--vscode-descriptionForeground); font-style: italic; margin: 10px 0;">
                            No tasks are currently present in this project. You can create notes using the "Create Note" tab above, 
                            or add tasks directly to your code files using Anchora's comment syntax.
                        </p>
                    </div>
                ` : overview.sections.map((section: any) => `
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
            </div>
            
            <!-- Create Note Tab -->
            <div id="create-note" class="tab-content ${activeTab === 'create-note' ? 'active' : ''}">
                <div class="note-form">
                    <h2>📝 Create New Note</h2>
                    <p>Notes are ideas that can be converted into tasks later when you're ready to implement them.</p>
                    
                    <form id="noteForm">
                        <div class="form-group">
                            <label for="noteTitle">Title *</label>
                            <input type="text" id="noteTitle" placeholder="Brief description of the idea" maxlength="100" required>
                        </div>
                        
                        <div class="form-group">
                            <label for="noteSection">Section *</label>
                            <select id="noteSection" required>
                                <option value="">Select section...</option>
                                <option value="dev">dev - Development</option>
                                <option value="bug">bug - Bug fixes</option>
                                <option value="feature">feature - New features</option>
                                <option value="refactor">refactor - Code refactoring</option>
                                <option value="test">test - Testing</option>
                                <option value="doc">doc - Documentation</option>
                            </select>
                        </div>
                        
                        <div class="form-group">
                            <label for="noteTaskId">Future Task ID *</label>
                            <input type="text" id="noteTaskId" placeholder="task_id (letters, numbers, underscores)" pattern="[a-zA-Z_][a-zA-Z0-9_]*" required>
                            <small style="color: var(--vscode-descriptionForeground);">Must start with a letter, contain only letters, numbers and underscores</small>
                        </div>
                        
                        <div class="form-group">
                            <label for="noteStatus">Suggested Status</label>
                            <select id="noteStatus">
                                <option value="todo">Todo - Task awaiting execution</option>
                                <option value="in_progress">In Progress - Task in progress</option>
                                <option value="blocked">Blocked - Task blocked</option>
                            </select>
                        </div>
                        
                        <div class="form-group">
                            <label for="noteContent">Content *</label>
                            <textarea id="noteContent" placeholder="Detailed description of the idea, approaches, requirements..." required></textarea>
                        </div>
                        
                        <div class="form-group">
                            <button type="submit" class="btn">Create Note</button>
                            <button type="button" class="btn btn-secondary" onclick="clearForm()">Clear Form</button>
                        </div>
                        
                        <div id="errorMessage" class="error-message"></div>
                        <div id="successMessage" class="success-message"></div>
                    </form>
                </div>
            </div>
            
            <script>
                const vscode = acquireVsCodeApi();
                
                // Check if we should open a specific tab
                const urlParams = new URLSearchParams(window.location.search);
                const activeTab = urlParams.get('tab');
                if (activeTab === 'create-note') {
                    showTab('create-note');
                }
                
                // Tab switching
                function showTab(tabName) {
                    // Hide all tab contents
                    document.querySelectorAll('.tab-content').forEach(content => {
                        content.classList.remove('active');
                    });
                    
                    // Remove active class from all tabs
                    document.querySelectorAll('.tab').forEach(tab => {
                        tab.classList.remove('active');
                    });
                    
                    // Show selected tab content
                    document.getElementById(tabName).classList.add('active');
                    
                    // Add active class to clicked tab
                    event.target.classList.add('active');
                }
                
                // Form handling
                document.getElementById('noteForm').addEventListener('submit', function(e) {
                    e.preventDefault();
                    
                    const formData = {
                        title: document.getElementById('noteTitle').value.trim(),
                        section: document.getElementById('noteSection').value,
                        suggested_task_id: document.getElementById('noteTaskId').value.trim(),
                        suggested_status: document.getElementById('noteStatus').value,
                        content: document.getElementById('noteContent').value.trim()
                    };
                    
                    // Basic validation
                    if (!formData.title || !formData.section || !formData.suggested_task_id || !formData.content) {
                        showError('All required fields must be filled');
                        return;
                    }
                    
                    // Validate task ID format
                    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(formData.suggested_task_id)) {
                        showError('Task ID must start with a letter and contain only letters, numbers and underscores');
                        return;
                    }
                    
                    // Send to extension
                    vscode.postMessage({
                        command: 'createNote',
                        data: formData
                    });
                    
                    hideMessages();
                });
                
                function clearForm() {
                    document.getElementById('noteForm').reset();
                    hideMessages();
                }
                
                function showError(message) {
                    const errorDiv = document.getElementById('errorMessage');
                    errorDiv.textContent = message;
                    errorDiv.style.display = 'block';
                    document.getElementById('successMessage').style.display = 'none';
                }
                
                function showSuccess(message) {
                    const successDiv = document.getElementById('successMessage');
                    successDiv.textContent = message;
                    successDiv.style.display = 'block';
                    document.getElementById('errorMessage').style.display = 'none';
                }
                
                function hideMessages() {
                    document.getElementById('errorMessage').style.display = 'none';
                    document.getElementById('successMessage').style.display = 'none';
                }
                
                // Handle messages from extension
                window.addEventListener('message', event => {
                    const message = event.data;
                    switch (message.command) {
                        case 'showError':
                            showError(message.message);
                            break;
                        case 'noteCreated':
                            showSuccess(message.message);
                            clearForm();
                            // Switch back to overview tab
                            showTab('overview');
                            document.querySelector('.tab').click();
                            break;
                    }
                });
            </script>
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
     * Handle note creation from webview
     */
    private async handleWebviewCreateNote(noteData: any, panel: vscode.WebviewPanel): Promise<void> {
        logCommandInfo('=== Handling webview create note ===');
        try {
            if (!noteData.title || !noteData.content || !noteData.section || !noteData.suggested_task_id) {
                panel.webview.postMessage({
                    command: 'showError',
                    message: 'All fields are required'
                });
                return;
            }
            const response = await this.client.createNote({
                title: noteData.title.trim(),
                content: noteData.content.trim(),
                section: noteData.section,
                suggested_task_id: noteData.suggested_task_id.trim(),
                suggested_status: noteData.suggested_status || 'todo'
            });
            if (response.success) {
                panel.webview.postMessage({
                    command: 'noteCreated',
                    message: `Note "${noteData.title}" created successfully!`,
                    noteId: response.note_id
                });
                this.noteProvider.refresh();
                logCommandInfo(`Note created successfully from webview: ${response.note_id}`);
                vscode.window.showInformationMessage(`Note "${noteData.title}" created successfully!`);
                const newOverview = await this.getTaskOverview();
                panel.webview.html = this.getTaskOverviewHtml(newOverview);
            } else {
                panel.webview.postMessage({
                    command: 'showError',
                    message: `Error creating note: ${response.message}`
                });
                logCommandError('Failed to create note from webview', response);
            }
        } catch (error) {
            panel.webview.postMessage({
                command: 'showError',
                message: 'An unexpected error occurred'
            });
            this.handleError('webview create note', error);
        }
    }


    /**
     * Get task overview data for webview
     */
    private async getTaskOverview(): Promise<any> {
        try {
            const serverOverview = await this.client.getTaskOverview();
            return {
                totalSections: serverOverview.sections.length,
                totalTasks: serverOverview.statistics.total_tasks,
                statusCounts: {
                    todo: serverOverview.statistics.by_status.todo,
                    in_progress: serverOverview.statistics.by_status.in_progress,
                    done: serverOverview.statistics.by_status.done,
                    blocked: serverOverview.statistics.by_status.blocked
                },
                sections: serverOverview.sections.map(section => ({
                    name: section.name,
                    taskCount: section.total_tasks,
                    tasks: section.tasks || []
                }))
            };
        } catch (error) {
            logCommandError('Failed to get server-side task overview', error);
            return {
                totalSections: 0,
                totalTasks: 0,
                statusCounts: { todo: 0, in_progress: 0, done: 0, blocked: 0 },
                sections: []
            };
        }
    }

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

    /**
     * Toggle debug mode on/off
     */
    private async toggleDebugMode(): Promise<void> {
        try {
            const currentMode = isDebugMode();
            const newMode = !currentMode;
            setDebugMode(newMode);
            const config = vscode.workspace.getConfiguration('anchora');
            await config.update('debugMode', newMode, vscode.ConfigurationTarget.Workspace);
            const message = `Debug mode ${newMode ? 'enabled' : 'disabled'}`;
            logCommandInfo(message);
            vscode.window.showInformationMessage(message);
            if (newMode) {
                getCommandOutputChannel().show();
            }
        } catch (error) {
            this.handleError('toggle debug mode', error);
        }
    }

    /**
     * Show output channels for debugging
     */
    private async showOutputChannel(): Promise<void> {
        try {
            const channels = [
                { label: 'Anchora Commands', channel: getCommandOutputChannel() },
                {
                    label: 'Anchora Client', action: () => {
                        const clientModule = require('./client');
                        if (clientModule.getOutputChannel) {
                            clientModule.getOutputChannel().show();
                        }
                    }
                }
            ];
            const selection = await vscode.window.showQuickPick(
                channels.map(c => c.label),
                { placeHolder: 'Select output channel to show' }
            );
            if (selection) {
                const selected = channels.find(c => c.label === selection);
                if (selected) {
                    if ('channel' in selected) {
                        selected.channel.show();
                    } else if ('action' in selected) {
                        selected.action();
                    }
                }
            }
        } catch (error) {
            this.handleError('show output channel', error);
        }
    }

    /**
     * Test enhanced error handling by triggering a backend error
     */
    private async testErrorHandling(): Promise<void> {
        try {
            logCommandInfo('Testing enhanced error handling...');
            await this.client.findTaskReferences({
                section: 'nonexistent_section',
                task_id: 'nonexistent_task'
            });
        } catch (error) {
            logCommandInfo('Error handling test completed - error was caught as expected');
            if (isDebugMode()) {
                vscode.window.showInformationMessage(
                    'Error handling test completed. Check the output channel for enhanced error details.',
                    'Show Output'
                ).then(selection => {
                    if (selection === 'Show Output') {
                        getCommandOutputChannel().show();
                    }
                });
            } else {
                vscode.window.showInformationMessage(
                    'Error handling test completed. Enable debug mode to see enhanced error details.',
                    'Enable Debug',
                    'Cancel'
                ).then(selection => {
                    if (selection === 'Enable Debug') {
                        this.toggleDebugMode();
                    }
                });
            }
        }
    }

    /**
     * Create a new note interactively
     */
    private async createNote(): Promise<void> {
        logCommandInfo('Opening Task Overview with Create Note tab');
        try {
            if (!this.client.isConnected()) {
                await this.client.connect();
            }
            const taskOverview = await this.getTaskOverview();
            await this.showTaskOverviewPanel(taskOverview, 'create-note');
        } catch (error) {
            this.handleError('open create note', error);
        }
    }

    /**
     * Refresh notes list
     */
    private async refreshNotes(): Promise<void> {
        logCommandInfo('Refreshing notes...');
        try {
            this.noteProvider.refresh();
            logCommandInfo('Notes refreshed successfully');
        } catch (error) {
            this.handleError('refresh notes', error);
        }
    }

    /**
     * Generate task link for a note
     */
    private async generateTaskLink(noteIdOrItem: string | any): Promise<void> {
        const noteId = typeof noteIdOrItem === 'string' ? noteIdOrItem : noteIdOrItem?.note?.id;
        if (!noteId) {
            vscode.window.showErrorMessage('Invalid note ID');
            return;
        }
        logCommandInfo(`Generating task link for note: ${noteId}`);
        try {
            const response = await this.client.generateTaskLink(noteId);
            if (response.success) {
                await vscode.env.clipboard.writeText(response.link);
                const selection = await vscode.window.showInformationMessage(
                    'Task link generated and copied to clipboard!',
                    'Insert in Code'
                );
                if (selection === 'Insert in Code') {
                    this.insertTaskLinkAtCursor(response.link);
                }
                this.noteProvider.refresh();
                logCommandInfo(`Task link generated successfully: ${response.link}`);
            } else {
                vscode.window.showErrorMessage('Error generating task link');
                logCommandError('Failed to generate task link', response);
            }
        } catch (error) {
            this.handleError('generate task link', error);
        }
    }

    /**
     * View note content
     */
    private async viewNote(noteIdOrItem: string | any): Promise<void> {
        const noteId = typeof noteIdOrItem === 'string' ? noteIdOrItem : noteIdOrItem?.note?.id;
        if (!noteId) {
            vscode.window.showErrorMessage('Invalid note ID');
            return;
        }
        logCommandInfo(`Viewing note: ${noteId}`);
        try {
            const note = this.noteProvider.getNote(noteId);
            if (!note) {
                vscode.window.showErrorMessage('Note not found');
                return;
            }
            const panel = vscode.window.createWebviewPanel(
                'noteView',
                `Note: ${note.title}`,
                vscode.ViewColumn.One,
                {
                    enableScripts: true
                }
            );
            panel.webview.html = this.getNoteWebviewContent(note);
            logCommandInfo(`Note webview opened for: ${note.title}`);
        } catch (error) {
            this.handleError('view note', error);
        }
    }

    /**
     * Delete a note
     */
    private async deleteNote(noteIdOrItem: string | any): Promise<void> {
        const noteId = typeof noteIdOrItem === 'string' ? noteIdOrItem : noteIdOrItem?.note?.id;
        if (!noteId) {
            vscode.window.showErrorMessage('Invalid note ID');
            return;
        }
        logCommandInfo(`Deleting note: ${noteId}`);
        try {
            const note = this.noteProvider.getNote(noteId);
            if (!note) {
                vscode.window.showErrorMessage('Note not found');
                return;
            }
            const confirmation = await vscode.window.showWarningMessage(
                `Are you sure you want to delete note "${note.title}"?`,
                { modal: true },
                'Delete',
                'Cancel'
            );
            if (confirmation !== 'Delete') {
                logCommandInfo('Note deletion cancelled');
                return;
            }
            const response = await this.client.deleteNote(noteId);
            if (response.success) {
                vscode.window.showInformationMessage('Note deleted successfully');
                this.noteProvider.refresh();
                logCommandInfo(`Note deleted successfully: ${noteId}`);
            } else {
                vscode.window.showErrorMessage(`Error deleting note: ${response.message}`);
                logCommandError('Failed to delete note', response);
            }
        } catch (error) {
            this.handleError('delete note', error);
        }
    }

    /**
     * Insert task link at cursor position
     */
    private insertTaskLinkAtCursor(link: string): void {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showInformationMessage('Open a file to insert the task link');
            return;
        }
        editor.edit(editBuilder => {
            editBuilder.insert(editor.selection.active, link);
        });
        logCommandInfo('Task link inserted at cursor position');
    }

    /**
     * Generate webview content for note viewing
     */
    private getNoteWebviewContent(note: Note): string {
        return `
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Note: ${note.title}</title>
            <style>
                body { 
                    font-family: var(--vscode-font-family); 
                    padding: 20px;
                    background-color: var(--vscode-editor-background);
                    color: var(--vscode-editor-foreground);
                }
                .header { 
                    border-bottom: 1px solid var(--vscode-panel-border); 
                    padding-bottom: 15px; 
                    margin-bottom: 20px; 
                }
                .title { 
                    font-size: 24px; 
                    font-weight: bold; 
                    margin-bottom: 10px; 
                }
                .meta { 
                    color: var(--vscode-descriptionForeground); 
                    font-size: 14px; 
                }
                .content { 
                    line-height: 1.6; 
                }
                .status { 
                    display: inline-block; 
                    padding: 4px 8px; 
                    border-radius: 4px; 
                    background-color: var(--vscode-badge-background);
                    color: var(--vscode-badge-foreground);
                    font-size: 12px;
                    font-weight: bold;
                }
                .converted { 
                    color: var(--vscode-testing-iconPassed); 
                }
                .link { 
                    font-family: monospace; 
                    background-color: var(--vscode-textCodeBlock-background); 
                    padding: 8px; 
                    border-radius: 4px; 
                }
            </style>
        </head>
        <body>
            <div class="header">
                <div class="title">${note.title}</div>
                <div class="meta">
                    <span class="status">${note.section}:${note.suggested_task_id}</span>
                    <span class="status">${note.suggested_status}</span>
                    ${note.is_converted ? '<span class="status converted">✅ Converted</span>' : ''}
                    <br>
                    <small>Created: ${new Date(note.created).toLocaleString()}</small>
                    ${note.updated !== note.created ? `<br><small>Updated: ${new Date(note.updated).toLocaleString()}</small>` : ''}
                </div>
            </div>
            <div class="content">
                ${note.content.replace(/\n/g, '<br>')}
            </div>
            ${note.generated_link ? `
                <div style="margin-top: 20px;">
                    <h3>Generated Link:</h3>
                    <div class="link">${note.generated_link}</div>
                </div>
            ` : ''}
        </body>
        </html>`;
    }
}