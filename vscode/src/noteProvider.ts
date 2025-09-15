/**
 * Note Tree Provider for VSCode Extension
 * Displays notes in a separate tree view panel
 */

import * as vscode from 'vscode';
import { JsonRpcClient } from './client';
import { Note } from './types';

export class NoteTreeProvider implements vscode.TreeDataProvider<NoteItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<NoteItem | undefined | null | void> = new vscode.EventEmitter<NoteItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<NoteItem | undefined | null | void> = this._onDidChangeTreeData.event;
    private notes: ReadonlyArray<Note> = [];
    constructor(private client: JsonRpcClient) { }
    refresh(): void {
        this.loadNotes();
        this._onDidChangeTreeData.fire();
    }
    private async loadNotes(): Promise<void> {
        try {
            this.notes = await this.client.getNotes();
        } catch (error) {
            console.error('Error loading notes:', error);
            vscode.window.showErrorMessage(`Failed to load notes: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    getTreeItem(element: NoteItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: NoteItem): Thenable<NoteItem[]> {
        if (!element) {
            return Promise.resolve(
                this.notes.map(note => new NoteItem(
                    note,
                    vscode.TreeItemCollapsibleState.None
                ))
            );
        }
        return Promise.resolve([]);
    }

    getNote(noteId: string): Note | undefined {
        return this.notes.find(note => note.id === noteId);
    }
}

export class NoteItem extends vscode.TreeItem {
    constructor(
        public readonly note: Note,
        public override readonly collapsibleState: vscode.TreeItemCollapsibleState
    ) {
        super(note.title, collapsibleState);
        this.tooltip = this.createTooltip();
        this.description = this.createDescription();
        this.iconPath = this.getIcon();
        this.contextValue = this.getContextValue();
        this.command = {
            command: 'anchora.viewNote',
            title: 'View Note',
            arguments: [note.id]
        };
    }

    private createTooltip(): vscode.MarkdownString {
        const tooltip = new vscode.MarkdownString();
        tooltip.appendMarkdown(`**${this.note.title}**\n\n`);
        tooltip.appendMarkdown(`**Section:** ${this.note.section}\n`);
        tooltip.appendMarkdown(`**Task ID:** ${this.note.suggested_task_id}\n`);
        tooltip.appendMarkdown(`**Status:** ${this.note.suggested_status}\n`);
        tooltip.appendMarkdown(`**Created:** ${new Date(this.note.created).toLocaleString()}\n\n`);
        if (this.note.content) {
            tooltip.appendMarkdown(`**Content:**\n${this.note.content}\n\n`);
        }
        if (this.note.is_converted) {
            tooltip.appendMarkdown(`✅ **Converted to task** at ${new Date(this.note.converted_at!).toLocaleString()}`);
        } else if (this.note.generated_link) {
            tooltip.appendMarkdown(`🔗 **Link generated:** \`${this.note.generated_link}\``);
        }
        return tooltip;
    }

    private createDescription(): string {
        if (this.note.is_converted) {
            return 'Converted to task';
        } else if (this.note.generated_link) {
            return 'Link generated';
        }
        return `${this.note.section}:${this.note.suggested_task_id}`;
    }

    private getIcon(): vscode.ThemeIcon {
        if (this.note.is_converted) {
            return new vscode.ThemeIcon('check', new vscode.ThemeColor('charts.green'));
        } else if (this.note.generated_link) {
            return new vscode.ThemeIcon('link', new vscode.ThemeColor('charts.blue'));
        }
        return new vscode.ThemeIcon('lightbulb', new vscode.ThemeColor('charts.yellow'));
    }

    private getContextValue(): string {
        if (this.note.is_converted) {
            return 'convertedNote';
        } else if (this.note.generated_link) {
            return 'noteWithLink';
        }
        return 'note';
    }
}