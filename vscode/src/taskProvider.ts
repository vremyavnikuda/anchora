/**
 * Task Tree Provider for VSCode Explorer
 * Displays Anchora tasks in a hierarchical tree view
 */

import * as vscode from 'vscode';
import { JsonRpcClient } from './client';
import {
    TaskTreeItem,
    TaskStatus,
    ProjectData
} from './types';

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
    constructor(private readonly client: JsonRpcClient) { }
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
        try {
            if (!this.client.isConnected()) {
                await this.client.connect();
            }
            this.projectData = await this.client.getTasks();
            this.refresh();
        } catch (error) {
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
                    item.command = {
                        command: 'vscode.open',
                        title: 'Open File',
                        arguments: [
                            vscode.Uri.file(element.filePath),
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