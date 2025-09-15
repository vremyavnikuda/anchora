/**
 * Status Bar Manager for Anchora
 * Displays task statistics and context in VSCode status bar
 */

import * as vscode from 'vscode';
import { TaskTreeProvider } from './taskProvider';
import { TaskStatus } from './types';

export class StatusBarManager {
    private statusBarItem: vscode.StatusBarItem;
    private currentTaskItem: vscode.StatusBarItem;
    constructor(private readonly taskProvider: TaskTreeProvider) {
        this.statusBarItem = vscode.window.createStatusBarItem(
            vscode.StatusBarAlignment.Left,
            100
        );
        this.currentTaskItem = vscode.window.createStatusBarItem(
            vscode.StatusBarAlignment.Left,
            99
        );
        this.setupStatusBarItems();
    }

    /**
     * Register status bar items with the extension context
     */
    register(context: vscode.ExtensionContext): void {
        context.subscriptions.push(this.statusBarItem, this.currentTaskItem);
        this.refresh().catch(error => console.error('Error during initial status bar refresh:', error));
    }

    /**
     * Refresh status bar with current task statistics
     */
    async refresh(): Promise<void> {
        try {
            const taskCounts = await this.taskProvider.getTaskCounts();
            this.updateTaskStatistics(taskCounts);
            this.statusBarItem.show();
        } catch (error) {
            console.error('Error refreshing status bar:', error);
            this.statusBarItem.hide();
        }
    }

    /**
     * Update status bar for active editor
     */
    updateForActiveEditor(editor: vscode.TextEditor | undefined): void {
        if (!editor) {
            this.currentTaskItem.hide();
            return;
        }
        const currentTask = this.getCurrentTaskFromEditor(editor);
        if (currentTask) {
            this.updateCurrentTask(currentTask);
            this.currentTaskItem.show();
        } else {
            this.currentTaskItem.hide();
        }
    }

    /**
     * Setup initial status bar item properties
     */
    private setupStatusBarItems(): void {
        this.statusBarItem.command = 'anchora.searchTasks';
        this.statusBarItem.tooltip = 'Click to search tasks';
        this.currentTaskItem.command = 'anchora.findTaskReferences';
        this.currentTaskItem.tooltip = 'Click to find task references';
    }

    /**
     * Update task statistics in status bar
     */
    private updateTaskStatistics(taskCounts: Record<TaskStatus, number>): void {
        const total = Object.values(taskCounts).reduce((sum, count) => sum + count, 0);
        if (total === 0) {
            this.statusBarItem.text = '$(checklist) No tasks';
            this.statusBarItem.tooltip = 'No tasks found. Click to search or scan project.';
            return;
        }
        const statusIcons: Record<TaskStatus, string> = {
            'todo': '○',
            'in_progress': '◐',
            'done': '●',
            'blocked': '◯'
        };
        const statusParts: string[] = [];
        for (const [status, count] of Object.entries(taskCounts)) {
            if (count > 0) {
                const icon = statusIcons[status as TaskStatus];
                statusParts.push(`${icon}${count}`);
            }
        }
        this.statusBarItem.text = `$(checklist) ${statusParts.join(' ')}`;
        this.statusBarItem.tooltip = this.createTaskStatisticsTooltip(taskCounts, total);
    }

    /**
     * Update current task context in status bar
     */
    private updateCurrentTask(taskRef: { section: string; taskId: string; status?: TaskStatus }): void {
        const statusIcon = taskRef.status ? this.getStatusIcon(taskRef.status) : '○';
        this.currentTaskItem.text = `$(tag) ${statusIcon} ${taskRef.section}:${taskRef.taskId}`;
        this.currentTaskItem.tooltip = `Current task: ${taskRef.section}:${taskRef.taskId}${taskRef.status ? ` (${taskRef.status})` : ''
            }\nClick to find all references`;
    }

    /**
     * Get current task reference from active editor
     */
    private getCurrentTaskFromEditor(editor: vscode.TextEditor): {
        section: string;
        taskId: string;
        status?: TaskStatus
    } | null {
        try {
            const line = editor.document.lineAt(editor.selection.active.line);
            const taskMatch = line.text.match(/\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*)/);
            if (!taskMatch) {
                return null;
            }
            const section = taskMatch[1];
            const taskId = taskMatch[2];
            const status = this.getTaskStatusFromProvider(section || '', taskId || '');
            return {
                section: section || '',
                taskId: taskId || '',
                ...(status !== undefined && { status })
            };
        } catch (error) {
            console.error('Error getting current task from editor:', error);
            return null;
        }
    }

    /**
     * Get task status from the task provider (simplified)
     */
    private getTaskStatusFromProvider(section: string, taskId: string): TaskStatus | undefined {
        try {
            const projectData = (this.taskProvider as any).projectData;
            return projectData?.[section]?.[taskId]?.status;
        } catch (error) {
            console.error('Error getting task status:', error);
            return undefined;
        }
    }

    /**
     * Create detailed tooltip for task statistics
     */
    private createTaskStatisticsTooltip(taskCounts: Record<TaskStatus, number>, total: number): string {
        const lines = [
            `Anchora Tasks: ${total} total`,
            '',
            `○ Todo: ${taskCounts.todo}`,
            `◐ In Progress: ${taskCounts.in_progress}`,
            `● Done: ${taskCounts.done}`,
            `◯ Blocked: ${taskCounts.blocked}`,
            '',
            'Click to search tasks'
        ];
        return lines.join('\n');
    }

    /**
     * Get status icon for task status
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
     * Hide all status bar items
     */
    hide(): void {
        this.statusBarItem.hide();
        this.currentTaskItem.hide();
    }
    /**
     * Show all status bar items
     */
    show(): void {
        this.statusBarItem.show();
    }
}