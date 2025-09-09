/**
 * Task Tree Provider for VSCode Explorer
 * Displays Anchora tasks in a hierarchical tree view
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { JsonRpcClient, isDebugMode } from './client';
import {
    TaskTreeItem,
    TaskStatus,
    ProjectData
} from './types';

let providerOutputChannel: vscode.OutputChannel | null = null;

function getProviderOutputChannel(): vscode.OutputChannel {
    if (!providerOutputChannel) {
        providerOutputChannel = vscode.window.createOutputChannel('Anchora TaskProvider');
    }
    return providerOutputChannel;
}

function logProviderInfo(message: string, data?: any): void {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] PROVIDER: ${message}`;
    console.log(logMessage);
    getProviderOutputChannel().appendLine(logMessage);

    if (data !== undefined && isDebugMode()) {
        const dataStr = typeof data === 'object' ? JSON.stringify(data, null, 2) : String(data);
        getProviderOutputChannel().appendLine(`Data: ${dataStr}`);
        console.log('Provider data:', data);
    }
}

function logProviderError(message: string, error?: any, context?: any): void {
    const timestamp = new Date().toISOString();
    const errorDetails = error ? ` - ${error instanceof Error ? error.message : String(error)}` : '';
    const errorStack = error instanceof Error ? error.stack : '';
    const logMessage = `[${timestamp}] PROVIDER ERROR: ${message}${errorDetails}`;

    console.error(logMessage);
    getProviderOutputChannel().appendLine(logMessage);

    if (context && isDebugMode()) {
        const contextStr = typeof context === 'object' ? JSON.stringify(context, null, 2) : String(context);
        getProviderOutputChannel().appendLine(`Context: ${contextStr}`);
        console.error('Error context:', context);
    }

    if (errorStack && isDebugMode()) {
        getProviderOutputChannel().appendLine(`Stack trace: ${errorStack}`);
        console.error('Full error object:', error);
    }

    // Always show output channel on errors in debug mode
    if (isDebugMode()) {
        getProviderOutputChannel().show(true);
    }
}

function logProviderDebug(message: string, data?: any): void {
    if (!isDebugMode()) return;

    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] PROVIDER DEBUG: ${message}`;
    console.debug(logMessage);
    getProviderOutputChannel().appendLine(logMessage);

    if (data !== undefined) {
        const dataStr = typeof data === 'object' ? JSON.stringify(data, null, 2) : String(data);
        getProviderOutputChannel().appendLine(`Debug data: ${dataStr}`);
        console.debug('Debug data:', data);
    }
}

export class TaskTreeProvider implements vscode.TreeDataProvider<TaskTreeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<TaskTreeItem | undefined | null | void> = new vscode.EventEmitter<TaskTreeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<TaskTreeItem | undefined | null | void> = this._onDidChangeTreeData.event;
    private projectData: ProjectData['sections'] | null = null;
    private readonly statusIcons: Record<TaskStatus, string> = {
        'todo': '○',
        'in_progress': '◐',
        'done': '●',
        'blocked': '◯'
    };

    constructor(private readonly client: JsonRpcClient) {
        logProviderInfo('TaskTreeProvider initialized');
    }
    /**
     * Refresh the tree view
     */
    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    /**
     * Load tasks from backend and refresh tree
     */
    async loadTasks(): Promise<void> {
        logProviderInfo('=== Loading tasks from backend ===');
        try {
            logProviderDebug('Checking client connection');
            if (!this.client.isConnected()) {
                logProviderInfo('Client not connected, attempting to connect...');
                await this.client.connect();
                logProviderInfo('Client connected successfully');
            }

            logProviderInfo('Fetching tasks from client');
            const startTime = Date.now();
            this.projectData = await this.client.getTasks();
            const loadTime = Date.now() - startTime;

            const sectionCount = this.projectData ? Object.keys(this.projectData).length : 0;
            let totalTasks = 0;
            if (this.projectData) {
                for (const section of Object.values(this.projectData)) {
                    totalTasks += Object.keys(section).length;
                }
            }

            logProviderInfo(`Tasks loaded successfully in ${loadTime}ms`, {
                sections: sectionCount,
                totalTasks,
                loadTimeMs: loadTime
            });

            logProviderDebug('Refreshing tree view');
            this.refresh();
            logProviderInfo('=== Task loading completed ===');
        } catch (error) {
            const errorContext = {
                isConnected: this.client.isConnected(),
                hasProjectData: !!this.projectData,
                timestamp: new Date().toISOString()
            };

            logProviderError('Failed to load tasks', error, errorContext);

            const message = error instanceof Error ? error.message : String(error);
            vscode.window.showErrorMessage(`Failed to load tasks: ${message}`);
            console.error('Failed to load tasks:', error);
        }
    }
    /**
     * Get tree item representation
     */
    getTreeItem(element: TaskTreeItem): vscode.TreeItem {
        const item = new vscode.TreeItem(
            element.label,
            element.type === 'file' ? vscode.TreeItemCollapsibleState.None : vscode.TreeItemCollapsibleState.Expanded
        );
        switch (element.type) {
            case 'section':
                item.iconPath = new vscode.ThemeIcon('folder');
                item.contextValue = 'section';
                item.tooltip = `Section: ${element.label}`;
                break;
            case 'task':
                const statusIcon = element.status ? this.statusIcons[element.status] : '○';
                item.label = `${statusIcon} ${element.label}`;
                item.iconPath = new vscode.ThemeIcon('symbol-event');
                item.contextValue = 'task';
                item.tooltip = this.createTaskTooltip(element);
                if (element.section && element.taskId) {
                    item.command = {
                        command: 'anchora.showTaskReferences',
                        title: 'Show Task References',
                        arguments: [element.section, element.taskId]
                    };
                }
                break;
            case 'file':
                item.iconPath = vscode.ThemeIcon.File;
                item.contextValue = 'file';
                item.tooltip = `${element.filePath}${element.line ? `:${element.line}` : ''}`;
                if (element.filePath) {
                    // Handle both relative and absolute paths
                    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
                    let absolutePath: string;

                    if (workspaceFolder && !path.isAbsolute(element.filePath)) {
                        // Convert relative path to absolute using workspace root
                        absolutePath = path.join(workspaceFolder.uri.fsPath, element.filePath);
                    } else {
                        absolutePath = element.filePath;
                    }

                    item.command = {
                        command: 'vscode.open',
                        title: 'Open File',
                        arguments: [
                            vscode.Uri.file(absolutePath),
                            {
                                selection: element.line ? new vscode.Range(element.line - 1, 0, element.line - 1, 0) : undefined
                            }
                        ]
                    };
                }
                break;
        }
        return item;
    }

    /**
     * Get children of a tree item
     */
    async getChildren(element?: TaskTreeItem): Promise<TaskTreeItem[]> {
        if (!this.projectData) {
            await this.loadTasks();
            if (!this.projectData) {
                return [];
            }
        }
        if (!element) {
            return this.getSections();
        }
        switch (element.type) {
            case 'section':
                return this.getTasksInSection(element.label);
            case 'task':
                return this.getFilesForTask(element.section!, element.taskId!);
            default:
                return [];
        }
    }

    /**
     * Get all sections as tree items
     */
    private getSections(): TaskTreeItem[] {
        if (!this.projectData) return [];
        return Object.keys(this.projectData)
            .sort()
            .map(sectionName => ({
                type: 'section' as const,
                label: sectionName
            }));
    }

    /**
     * Get all tasks in a section
     */
    private getTasksInSection(sectionName: string): TaskTreeItem[] {
        const section = this.projectData?.[sectionName];
        if (!section) return [];
        return Object.entries(section)
            .sort(([, a], [, b]) => a.title.localeCompare(b.title))
            .map(([taskId, task]) => ({
                type: 'task' as const,
                label: task.title,
                section: sectionName,
                taskId,
                status: task.status,
                description: task.description || ''
            }));
    }

    /**
     * Get all files that reference a task
     */
    private getFilesForTask(sectionName: string, taskId: string): TaskTreeItem[] {
        const task = this.projectData?.[sectionName]?.[taskId];
        if (!task) return [];
        return Object.entries(task.files)
            .sort(([a], [b]) => a.localeCompare(b))
            .flatMap(([filePath, taskFile]) =>
                taskFile.lines.map(line => ({
                    type: 'file' as const,
                    label: `${this.getFileName(filePath)}:${line}`,
                    filePath,
                    line,
                    section: sectionName,
                    taskId
                }))
            );
    }

    /**
     * Create tooltip for a task
     */
    private createTaskTooltip(element: TaskTreeItem): string {
        const parts: string[] = [];
        if (element.section && element.taskId) {
            parts.push(`Task: ${element.section}:${element.taskId}`);
        }
        if (element.status) {
            parts.push(`Status: ${element.status}`);
        }
        if (element.description) {
            parts.push(`Description: ${element.description}`);
        }
        return parts.join('\n');
    }

    /**
     * Extract filename from full path
     */
    private getFileName(filePath: string): string {
        return filePath.split(/[/\\]/).pop() || filePath;
    }

    /**
     * Update task status and refresh tree
     */
    async updateTaskStatus(sectionName: string, taskId: string, newStatus: TaskStatus): Promise<void> {
        try {
            await this.client.updateTaskStatus({
                section: sectionName,
                task_id: taskId,
                status: newStatus
            });
            if (this.projectData?.[sectionName]?.[taskId]) {
                const existingTask = this.projectData[sectionName]?.[taskId];
                if (existingTask) {
                    this.projectData = {
                        ...this.projectData,
                        [sectionName]: {
                            ...this.projectData[sectionName],
                            [taskId]: {
                                title: existingTask.title,
                                ...(existingTask.description !== undefined && { description: existingTask.description }),
                                status: newStatus,
                                created: existingTask.created,
                                updated: new Date().toISOString(),
                                files: existingTask.files
                            }
                        }
                    };
                }
            }
            this.refresh();
            vscode.window.showInformationMessage(`Task ${sectionName}:${taskId} status updated to ${newStatus}`);
        } catch (error) {
            const message = error instanceof Error ? error.message : String(error);
            vscode.window.showErrorMessage(`Failed to update task status: ${message}`);
            console.error('Failed to update task status:', error);
        }
    }

    /**
     * Get task count by status for status bar
     */
    getTaskCounts(): Record<TaskStatus, number> {
        const counts: Record<TaskStatus, number> = {
            todo: 0,
            in_progress: 0,
            done: 0,
            blocked: 0
        };
        if (!this.projectData) return counts;
        for (const section of Object.values(this.projectData)) {
            for (const task of Object.values(section)) {
                counts[task.status]++;
            }
        }
        return counts;
    }

    /**
     * Search tasks by query
     */
    searchTasks(query: string): TaskTreeItem[] {
        if (!this.projectData || !query.trim()) return [];
        const lowerQuery = query.toLowerCase();
        const results: TaskTreeItem[] = [];
        for (const [sectionName, section] of Object.entries(this.projectData)) {
            for (const [taskId, task] of Object.entries(section)) {
                const matches =
                    task.title.toLowerCase().includes(lowerQuery) ||
                    (task.description?.toLowerCase().includes(lowerQuery)) ||
                    taskId.toLowerCase().includes(lowerQuery) ||
                    sectionName.toLowerCase().includes(lowerQuery);
                if (matches) {
                    results.push({
                        type: 'task',
                        label: `${sectionName}:${taskId} - ${task.title}`,
                        section: sectionName,
                        taskId,
                        status: task.status,
                        description: task.description || ''
                    });
                }
            }
        }
        return results.sort((a, b) => a.label.localeCompare(b.label));
    }
}