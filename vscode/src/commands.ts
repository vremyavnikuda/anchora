/**
 * Command handlers for Anchora VSCode extension
 * Implements all user-facing commands with proper error handling
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { JsonRpcClient, setDebugMode, isDebugMode } from './client';
import { TaskTreeProvider } from './taskProvider';
import {
    TaskStatus,
    CreateTaskParams,
    ScanProjectParams,
    DeleteTaskParams,
    createTaskId,
    createSectionName,
    AnchoraError
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

    // Always show output channel on errors in debug mode
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
        private readonly context: vscode.ExtensionContext
    ) {
        logCommandInfo('CommandHandler initialized');

        // Enable debug mode based on configuration
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
            vscode.commands.registerCommand('anchora.createTask', () => this.createTask()),
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
            vscode.commands.registerCommand('anchora.testErrorHandling', () => this.testErrorHandling())
        ];

        commands.forEach(command => this.context.subscriptions.push(command));
        logCommandInfo(`Registered ${commands.length} commands successfully`);
    }

    /**
     * Create a new task interactively
     */
    private async createTask(): Promise<void> {
        logCommandInfo('=== Starting create task workflow ===');
        try {
            logCommandDebug('Prompting for section name');
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

            if (!section) {
                logCommandInfo('Create task cancelled - no section provided');
                return;
            }

            logCommandDebug('Section selected', { section });
            logCommandDebug('Prompting for task ID');

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

            if (!taskId) {
                logCommandInfo('Create task cancelled - no task ID provided');
                return;
            }

            logCommandDebug('Task ID selected', { taskId });
            logCommandDebug('Prompting for task title');

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

            if (!title) {
                logCommandInfo('Create task cancelled - no title provided');
                return;
            }

            logCommandDebug('Title provided', { title });
            logCommandDebug('Prompting for optional description');

            const description = await vscode.window.showInputBox({
                prompt: 'Enter detailed description (optional)',
                placeHolder: 'Detailed implementation notes...'
            });

            logCommandDebug('Description provided', { description: description || 'none' });

            const validSection = createSectionName(section);
            const validTaskId = createTaskId(taskId);

            if (!validSection || !validTaskId) {
                const validationError = new AnchoraError('Invalid section name or task ID');
                logCommandError('Validation failed', validationError, { section, taskId, validSection, validTaskId });
                throw validationError;
            }

            const params: CreateTaskParams = {
                section: validSection,
                task_id: validTaskId,
                title: title.trim(),
                ...(description?.trim() && { description: description.trim() })
            };

            logCommandInfo('Creating task via backend', { section: validSection, taskId: validTaskId });
            logCommandDebug('Create task parameters', params);

            const result = await this.client.createTask(params);

            logCommandDebug('Create task result', result);

            if (result.success) {
                logCommandInfo(`Task created successfully: ${section}:${taskId}`);
                vscode.window.showInformationMessage(`Task created: ${section}:${taskId}`);

                logCommandInfo('Refreshing task provider');
                await this.taskProvider.loadTasks();

                logCommandDebug('Prompting for task reference insertion');
                const insertReference = await vscode.window.showQuickPick(
                    ['Yes', 'No'],
                    { placeHolder: 'Insert task reference at current cursor position?' }
                );

                if (insertReference === 'Yes') {
                    logCommandInfo('Inserting task reference at cursor');
                    await this.insertTaskReference(section, taskId);
                }

                logCommandInfo('=== Create task workflow completed successfully ===');
            } else {
                logCommandError('Backend reported task creation failure', null, { result });
                vscode.window.showErrorMessage(`Failed to create task: ${result.message}`);
            }
        } catch (error) {
            this.handleError('create task', error, { workflow: 'createTask' });
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
     * Delete task interactively with confirmation
     * Also removes all task anchors from source code files
     */
    private async deleteTask(section?: string, taskId?: string): Promise<void> {
        try {
            let taskRef: { section: string; taskId: string } | null = null;

            if (section && taskId) {
                taskRef = { section, taskId };
            } else {
                // Try to get task reference from current line first
                taskRef = await this.getCurrentTaskReference();

                // If no task reference found on current line, let user select from available tasks
                if (!taskRef) {
                    taskRef = await this.selectTaskInteractively('Select task to delete');
                }
            }

            if (!taskRef) return;

            // Show confirmation dialog
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

            // Step 1: Get all task references (anchors) from code files
            let taskReferences: ReadonlyArray<any> = [];
            try {
                taskReferences = await this.client.findTaskReferences({
                    section: taskRef.section,
                    task_id: taskRef.taskId
                });
                logCommandDebug(`Found ${taskReferences.length} task references to remove`, taskReferences);
            } catch (error) {
                logCommandError('Failed to find task references', error);
                // Continue with deletion even if we can't find references
            }

            // Step 2: Remove task anchors from source files
            if (taskReferences.length > 0) {
                const removedAnchors = await this.removeTaskAnchors(taskRef.section, taskRef.taskId, taskReferences);
                logCommandInfo(`Removed ${removedAnchors} task anchors from source files`);
            }

            // Step 3: Delete the task from backend
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

        // Group references by file to process each file once
        const fileGroups = new Map<string, number[]>();
        for (const ref of taskReferences) {
            if (!fileGroups.has(ref.file_path)) {
                fileGroups.set(ref.file_path, []);
            }
            fileGroups.get(ref.file_path)!.push(ref.line);
        }

        logCommandDebug(`Processing ${fileGroups.size} files for anchor removal`);

        // Process each file
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
                // Continue processing other files even if one fails
            }
        }

        return removedCount;
    }

    /**
     * Remove task anchors from a specific file
     */
    private async removeTaskAnchorsFromFile(section: string, taskId: string, filePath: string, lineNumbers: number[]): Promise<number> {
        try {
            // Get workspace folder to resolve relative paths
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                throw new Error('No workspace folder available');
            }

            // Handle both relative and absolute paths
            let absolutePath: string;
            if (path.isAbsolute(filePath)) {
                absolutePath = filePath;
            } else {
                absolutePath = path.join(workspaceFolder.uri.fsPath, filePath);
            }

            const uri = vscode.Uri.file(absolutePath);

            // Open the document (this will create it in memory if not already open)
            const document = await vscode.workspace.openTextDocument(uri);

            // Create task anchor patterns to match
            const taskPatterns = [
                // Pattern 1: With status and description: // section:task_id:status: description
                new RegExp(`^\\s*//\\s*${section}:${taskId}:[a-zA-Z_][a-zA-Z0-9_]*:\\s+.+$`),
                // Pattern 2: Full definition: // section:task_id: description  
                new RegExp(`^\\s*//\\s*${section}:${taskId}:\\s+.+$`),
                // Pattern 3: Status update: // section:task_id:status
                new RegExp(`^\\s*//\\s*${section}:${taskId}:(todo|in_progress|inprogress|progress|done|completed|complete|blocked|block)\\s*$`, 'i'),
                // Pattern 4: With note: // section:task_id:note
                new RegExp(`^\\s*//\\s*${section}:${taskId}:[a-zA-Z0-9_]+\\s*$`),
                // Pattern 5: Simple reference: // section:task_id
                new RegExp(`^\\s*//\\s*${section}:${taskId}\\s*$`)
            ];

            // Build list of lines to remove (convert to 0-based indexing and sort in reverse order)
            const linesToRemove = lineNumbers
                .map(line => line - 1) // Convert from 1-based to 0-based
                .filter(line => line >= 0 && line < document.lineCount)
                .sort((a, b) => b - a); // Sort in reverse order to avoid index shifting

            if (linesToRemove.length === 0) {
                logCommandDebug(`No valid lines to remove in ${filePath}`);
                return 0;
            }

            // Verify that lines actually contain task anchors before removing
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

            // Apply edits to remove the lines
            const editor = await vscode.window.showTextDocument(document, { preview: false, preserveFocus: true });

            const success = await editor.edit(editBuilder => {
                for (const lineIndex of validLinesToRemove) {
                    // Remove the entire line including the line ending
                    const line = document.lineAt(lineIndex);
                    const range = line.rangeIncludingLineBreak;
                    editBuilder.delete(range);
                    logCommandDebug(`Removing line ${lineIndex + 1}: ${line.text.trim()}`);
                }
            });

            if (success) {
                // Save the document
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

        // Pattern 1: With status and description: // section:task_id:status: description
        let taskMatch = lineText.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):\s+(.+)/);
        if (taskMatch) {
            return {
                section: taskMatch[1] || '',
                taskId: taskMatch[2] || ''
            };
        }

        // Pattern 2: Full definition: // section:task_id: description
        taskMatch = lineText.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):\s+(.+)/);
        if (taskMatch) {
            return {
                section: taskMatch[1] || '',
                taskId: taskMatch[2] || ''
            };
        }

        // Pattern 3: Status update: // section:task_id:status
        taskMatch = lineText.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):(todo|in_progress|inprogress|progress|done|completed|complete|blocked|block)\s*$/i);
        if (taskMatch) {
            return {
                section: taskMatch[1] || '',
                taskId: taskMatch[2] || ''
            };
        }

        // Pattern 4: With note: // section:task_id:note
        taskMatch = lineText.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z0-9_]+)\s*$/);
        if (taskMatch && taskMatch[3]) {
            // Check if the third part is not a status (to distinguish from status updates)
            const thirdPart = taskMatch[3].toLowerCase();
            const statusKeywords = ['todo', 'in_progress', 'inprogress', 'progress', 'done', 'completed', 'complete', 'blocked', 'block'];
            if (!statusKeywords.includes(thirdPart)) {
                return {
                    section: taskMatch[1] || '',
                    taskId: taskMatch[2] || ''
                };
            }
        }

        // Pattern 5: Simple reference: // section:task_id
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
            if (!projectData || Object.keys(projectData).length === 0) {
                vscode.window.showInformationMessage('No tasks found in the project.');
                return null;
            }

            // Build list of all tasks
            const taskItems: Array<{
                label: string;
                description: string;
                detail: string;
                section: string;
                taskId: string;
            }> = [];

            for (const [sectionName, section] of Object.entries(projectData)) {
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
        try {
            // Get the workspace folder to resolve relative paths
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (!workspaceFolder) {
                throw new Error('No workspace folder available');
            }

            // Handle both relative and absolute paths
            let absolutePath: string;
            if (path.isAbsolute(filePath)) {
                absolutePath = filePath;
            } else {
                // Convert relative path to absolute using workspace root
                absolutePath = path.join(workspaceFolder.uri.fsPath, filePath);
            }

            const uri = vscode.Uri.file(absolutePath);
            const document = await vscode.workspace.openTextDocument(uri);
            const editor = await vscode.window.showTextDocument(document);
            const position = new vscode.Position(Math.max(0, line - 1), 0);
            editor.selection = new vscode.Selection(position, position);
            editor.revealRange(new vscode.Range(position, position));
        } catch (error) {
            // Enhanced error handling with path debugging
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

        // Show additional debug information in debug mode
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
                <h1>Anchora</h1>
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
                        // Import and show client channel
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

            // Try to find a non-existent task to trigger an error
            await this.client.findTaskReferences({
                section: 'nonexistent_section',
                task_id: 'nonexistent_task'
            });

        } catch (error) {
            // This is expected - we're testing error handling
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
}