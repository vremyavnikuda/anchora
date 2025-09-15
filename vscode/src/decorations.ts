/**
 * Decoration Provider for Anchora
 * Provides visual indicators for task references in code
 */

import * as vscode from 'vscode';
import { TaskTreeProvider } from './taskProvider';
import { NoteTreeProvider } from './noteProvider';
import { ExtensionConfig, TaskStatus } from './types';

export class DecorationProvider {
    private readonly decorationTypes: Map<TaskStatus, vscode.TextEditorDecorationType> = new Map();
    private readonly taskReferenceRegex = /\/\/\s*([a-zA-Z_][a-zA-Z0-9_]*):([a-zA-Z_][a-zA-Z0-9_]*)/g;
    constructor(
        private readonly taskProvider: TaskTreeProvider,
        private readonly noteProvider: NoteTreeProvider,
        private readonly config: ExtensionConfig
    ) {
        this.createDecorationTypes();
    }

    /**
     * Register the decoration provider with the extension context
     */
    register(context: vscode.ExtensionContext): void {
        for (const decorationType of this.decorationTypes.values()) {
            context.subscriptions.push(decorationType);
        }
        if (vscode.window.activeTextEditor) {
            this.decorateEditor(vscode.window.activeTextEditor);
        }
    }

    /**
     * Handle document changes to update decorations
     */
    onDocumentChanged(event: vscode.TextDocumentChangeEvent): void {
        const editor = vscode.window.visibleTextEditors.find(
            e => e.document === event.document
        );
        if (editor) {
            this.debounceDecoration(editor);
        }
    }

    /**
     * Handle active editor changes
     */
    onActiveEditorChanged(editor: vscode.TextEditor | undefined): void {
        if (editor) {
            this.decorateEditor(editor);
        }
    }

    /**
     * Refresh decorations for all visible editors
     */
    refreshDecorations(): void {
        for (const editor of vscode.window.visibleTextEditors) {
            this.decorateEditor(editor);
        }
    }

    /**
     * Create decoration types for different task statuses
     */
    private createDecorationTypes(): void {
        const statusColors = this.config.decorationColors;
        for (const [status, color] of Object.entries(statusColors)) {
            const decorationType = vscode.window.createTextEditorDecorationType({
                backgroundColor: color,
                color: '#ffffff',
                fontWeight: 'bold',
                overviewRulerColor: color,
                overviewRulerLane: vscode.OverviewRulerLane.Right,
                isWholeLine: false,
                rangeBehavior: vscode.DecorationRangeBehavior.ClosedClosed,
                after: {
                    contentText: ` ${this.getStatusIcon(status as TaskStatus)}`,
                    color: '#ffffff',
                    fontWeight: 'bold',
                    margin: '0 0 0 10px'
                }
            });
            this.decorationTypes.set(status as TaskStatus, decorationType);
        }
    }

    /**
     * Get status icon for a task status
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
     * Decorate an editor with task reference indicators
     */
    private async decorateEditor(editor: vscode.TextEditor): Promise<void> {
        try {
            for (const decorationType of this.decorationTypes.values()) {
                editor.setDecorations(decorationType, []);
            }
            const document = editor.document;
            const text = document.getText();
            const decorationsByStatus: Map<TaskStatus, vscode.DecorationOptions[]> = new Map();
            for (const status of ['todo', 'in_progress', 'done', 'blocked'] as TaskStatus[]) {
                decorationsByStatus.set(status, []);
            }
            let match;
            this.taskReferenceRegex.lastIndex = 0;
            while ((match = this.taskReferenceRegex.exec(text)) !== null) {
                const section = match[1];
                const taskId = match[2];
                const matchStart = match.index;
                const matchEnd = matchStart + match[0].length;
                if (section && taskId) {
                    const status = await this.getTaskStatus(section, taskId);
                    if (status && decorationsByStatus.has(status)) {
                        const startPos = document.positionAt(matchStart);
                        const endPos = document.positionAt(matchEnd);
                        const range = new vscode.Range(startPos, endPos);
                        const decoration: vscode.DecorationOptions = {
                            range,
                            hoverMessage: this.createHoverMessage(section, taskId, status)
                        };
                        decorationsByStatus.get(status)!.push(decoration);
                    }
                }
            }
            for (const [status, decorations] of decorationsByStatus) {
                const decorationType = this.decorationTypes.get(status);
                if (decorationType) {
                    editor.setDecorations(decorationType, decorations);
                }
            }
        } catch (error) {
            console.error('Error decorating editor:', error);
        }
    }

    /**
     * Get task status from the task provider
     */
    private async getTaskStatus(section: string, taskId: string): Promise<TaskStatus | null> {
        try {
            const tasks = await this.taskProvider['projectData'];
            const task = tasks?.[section]?.[taskId];
            return task?.status || null;
        } catch (error) {
            console.error('Error getting task status:', error);
            return null;
        }
    }

    /**
     * Get notes related to a specific task
     */
    private getNotesForTask(section: string, taskId: string) {
        return (this.noteProvider as any).notes?.filter((note: any) =>
            note.section === section && note.suggested_task_id === taskId
        ) || [];
    }

    /**
     * Create hover message for task reference
     */
    private createHoverMessage(section: string, taskId: string, status: TaskStatus): vscode.MarkdownString {
        const markdown = new vscode.MarkdownString();
        markdown.isTrusted = true;
        markdown.appendMarkdown(`**Task:** \`${section}:${taskId}\`\n\n`);
        markdown.appendMarkdown(`**Status:** ${this.getStatusIcon(status)} ${status}\n\n`);

        // Find and display related notes
        const relatedNotes = this.getNotesForTask(section, taskId);
        if (relatedNotes.length > 0) {
            markdown.appendMarkdown(`**Related Notes (${relatedNotes.length}):**\n\n`);
            for (const note of relatedNotes) {
                markdown.appendMarkdown(`• **${note.title}**\n`);
                if (note.content) {
                    // Truncate long content for hover display
                    const truncatedContent = note.content.length > 200
                        ? note.content.substring(0, 200) + '...'
                        : note.content;
                    markdown.appendMarkdown(`  ${truncatedContent}\n`);
                }
                if (note.is_converted) {
                    markdown.appendMarkdown(`  ✅ *Converted to task*\n`);
                } else {
                    markdown.appendMarkdown(`  📝 *Note created: ${new Date(note.created).toLocaleDateString()}*\n`);
                }
                markdown.appendMarkdown(`\n`);
            }
        }

        markdown.appendMarkdown(
            `[Go to Definition](command:anchora.goToTaskDefinition) | ` +
            `[Find References](command:anchora.findTaskReferences) | ` +
            `[Update Status](command:anchora.updateTaskStatus)`
        );
        return markdown;
    }

    /**
     * Debounced decoration update
     */
    private decorationTimeouts: Map<vscode.TextEditor, ReturnType<typeof setTimeout>> = new Map();
    private debounceDecoration(editor: vscode.TextEditor): void {
        const existingTimeout = this.decorationTimeouts.get(editor);
        if (existingTimeout) {
            clearTimeout(existingTimeout);
        }
        const timeout = setTimeout(() => {
            this.decorateEditor(editor);
            this.decorationTimeouts.delete(editor);
        }, 500);
        this.decorationTimeouts.set(editor, timeout);
    }
}