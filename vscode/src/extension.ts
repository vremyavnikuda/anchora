/**
/**
 * Main extension entry point for Anchora
 * Follows VSCode extension best practices and TypeScript strict typing
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { JsonRpcClient } from './client';
import { TaskTreeProvider } from './taskProvider';
import { NoteTreeProvider } from './noteProvider';
import { CommandHandler } from './commands';
import { DecorationProvider } from './decorations';
import { StatusBarManager } from './statusBar';
import { ExtensionConfig, BackendConnectionError } from './types';

let outputChannel: vscode.OutputChannel;

/**
 * Logging functions for debugging
 */
function logInfo(message: string): void {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] INFO: ${message}`;
    console.log(logMessage);
    if (outputChannel) {
        outputChannel.appendLine(logMessage);
    }
}

function logError(message: string, error?: any): void {
    const timestamp = new Date().toISOString();
    const errorDetails = error ? ` - ${error instanceof Error ? error.message : String(error)}` : '';
    const logMessage = `[${timestamp}] ERROR: ${message}${errorDetails}`;
    console.error(logMessage);
    if (outputChannel) {
        outputChannel.appendLine(logMessage);
        if (error && error.stack) {
            outputChannel.appendLine(`Stack trace: ${error.stack}`);
        }
    }
}

function logWarning(message: string): void {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] WARNING: ${message}`;
    console.warn(logMessage);
    if (outputChannel) {
        outputChannel.appendLine(logMessage);
    }
}

function logDebug(message: string): void {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] DEBUG: ${message}`;
    console.log(logMessage);
    if (outputChannel) {
        outputChannel.appendLine(logMessage);
    }
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
    outputChannel = vscode.window.createOutputChannel('Anchora');
    context.subscriptions.push(outputChannel);
    logInfo('Anchora extension activation started');
    try {
        if (!vscode.workspace.workspaceFolders?.length) {
            logWarning('No workspace folder found, extension will not activate');
            return;
        }
        const workspaceFolder = vscode.workspace.workspaceFolders[0];
        if (!workspaceFolder) {
            logWarning('No workspace folder found, extension will not activate');
            return;
        }
        const workspacePath = workspaceFolder.uri.fsPath;
        logInfo(`Workspace path: ${workspacePath}`);
        logInfo('Checking if this is an Anchora project or initializing...');
        const isAnchoraProject = await checkIfAnchoraProject(workspacePath);
        logInfo(`Is Anchora project: ${isAnchoraProject}`);
        await vscode.commands.executeCommand('setContext', 'workspaceHasAnchoraProject', isAnchoraProject);
        await vscode.commands.executeCommand('setContext', 'anchoraExtensionActive', true);
        logInfo('Extension contexts set successfully');
        logInfo('Loading extension configuration...');
        const config = getExtensionConfig();
        logDebug(`Config: ${JSON.stringify(config)}`);
        const binaryPath = getBinaryPath(workspacePath);
        logInfo(`Binary path: ${binaryPath}`);
        const client = new JsonRpcClient(workspacePath, binaryPath);
        logInfo('Initializing providers and managers...');
        const taskProvider = new TaskTreeProvider(client);
        const noteProvider = new NoteTreeProvider(client);
        const welcomeProvider = new WelcomeViewProvider();
        const commandHandler = new CommandHandler(client, taskProvider, noteProvider, context);
        const decorationProvider = new DecorationProvider(taskProvider, config);
        const statusBarManager = new StatusBarManager(taskProvider);
        logInfo('Registering tree data providers...');
        const treeView = vscode.window.createTreeView('anchoraTaskTree', {
            treeDataProvider: taskProvider,
            showCollapseAll: true
        });
        const noteTreeView = vscode.window.createTreeView('anchoraNoteTree', {
            treeDataProvider: noteProvider,
            showCollapseAll: true
        });
        const explorerTreeView = vscode.window.createTreeView('anchoraTaskTreeExplorer', {
            treeDataProvider: taskProvider,
            showCollapseAll: true
        });
        const welcomeView = vscode.window.createTreeView('anchoraWelcome', {
            treeDataProvider: welcomeProvider,
            showCollapseAll: false
        });
        logInfo('Tree views registered successfully');
        logInfo('Registering commands...');
        commandHandler.registerCommands();
        logInfo('Registering providers...');
        decorationProvider.register(context);
        statusBarManager.register(context);
        context.subscriptions.push(
            treeView,
            noteTreeView,
            explorerTreeView,
            welcomeView,
            vscode.workspace.createFileSystemWatcher(
                path.join(workspacePath, '.anchora', 'tasks.json')
            ),
            vscode.workspace.onDidChangeConfiguration(event => {
                if (event.affectsConfiguration('anchora')) {
                    handleConfigurationChange();
                }
            }),
            vscode.workspace.onDidChangeTextDocument(event => {
                decorationProvider.onDocumentChanged(event);
            }),
            vscode.window.onDidChangeActiveTextEditor(editor => {
                decorationProvider.onActiveEditorChanged(editor);
                statusBarManager.updateForActiveEditor(editor);
            })
        );
        if (isAnchoraProject) {
            try {
                logInfo('Attempting to connect to backend...');
                await client.connect();
                logInfo('Backend connected successfully');
                logInfo('Loading tasks...');
                await taskProvider.loadTasks();
                logInfo('Loading notes...');
                noteProvider.refresh();
                logInfo('Refreshing decorations...');
                decorationProvider.refreshDecorations();
                logInfo('Refreshing status bar...');
                statusBarManager.refresh();
                logInfo('Anchora extension activated successfully');
                vscode.window.showInformationMessage('Anchora activated');
            } catch (error) {
                logError('Failed to initialize Anchora features', error);
                if (error instanceof BackendConnectionError) {
                    vscode.window.showWarningMessage(
                        'Could not connect to Anchora backend. Some features may not work properly.',
                        'Build Backend'
                    ).then(selection => {
                        if (selection === 'Build Backend') {
                            buildBackend(workspacePath);
                        }
                    });
                } else {
                    throw error;
                }
            }
        } else {
            logInfo('Extension activated in limited mode (not an Anchora project)');
        }
        context.subscriptions.push({
            dispose: () => {
                client.disconnect();
            }
        });
    } catch (error) {
        logError('Failed to activate Anchora extension', error);
        vscode.window.showErrorMessage(
            `Failed to activate Anchora: ${error instanceof Error ? error.message : String(error)}`
        );
    }
}

export function deactivate(): void {
    logInfo('Anchora extension is being deactivated');
    if (outputChannel) {
        outputChannel.dispose();
    }
}

/**
 * Check if the current workspace is an Anchora project or initialize it
 */
async function checkIfAnchoraProject(workspacePath: string): Promise<boolean> {
    try {
        logDebug(`Checking workspace path: ${workspacePath}`);
        const anchoraDir = vscode.Uri.file(path.join(workspacePath, '.anchora'));
        try {
            await vscode.workspace.fs.stat(anchoraDir);
            logInfo('Found .anchora directory - this is an Anchora project');
            return true;
        } catch {
            logDebug('.anchora directory not found, initializing Anchora project...');
        }
        await initializeAnchoraProject(workspacePath);
        return true;
    } catch (error) {
        logError('Error checking or initializing Anchora project', error);
        return false;
    }
}

/**
 * Initialize Anchora project structure
 */
async function initializeAnchoraProject(workspacePath: string): Promise<void> {
    try {
        logInfo('Initializing Anchora project structure...');
        const anchoraDir = vscode.Uri.file(path.join(workspacePath, '.anchora'));
        try {
            await vscode.workspace.fs.createDirectory(anchoraDir);
            logInfo('Created .anchora directory');
        } catch (error) {
            logWarning('Failed to create .anchora directory or it already exists');
        }
        const tasksFile = vscode.Uri.file(path.join(workspacePath, '.anchora', 'tasks.json'));
        const initialTasksData = {
            meta: {
                version: "1.0.0",
                created: new Date().toISOString(),
                last_updated: new Date().toISOString(),
                project_name: path.basename(workspacePath)
            },
            sections: {},
            notes: {},
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
        try {
            await vscode.workspace.fs.stat(tasksFile);
            logInfo('tasks.json already exists, skipping initialization');
        } catch {
            const tasksContent = JSON.stringify(initialTasksData, null, 2);
            await vscode.workspace.fs.writeFile(tasksFile, Buffer.from(tasksContent, 'utf8'));
            logInfo('Created tasks.json with initial structure');
        }
        vscode.window.showInformationMessage(
            'Anchora project initialized successfully! You can now start managing tasks.',
            'Open Task Dashboard'
        ).then(selection => {
            if (selection === 'Open Task Dashboard') {
                vscode.commands.executeCommand('anchora.openTaskDashboard');
            }
        });
        logInfo('Anchora project initialization completed');
    } catch (error) {
        logError('Failed to initialize Anchora project', error);
        vscode.window.showErrorMessage(
            `Failed to initialize Anchora project: ${error instanceof Error ? error.message : String(error)}`
        );
        throw error;
    }
}

/**
 * Get extension configuration
 */
function getExtensionConfig(): ExtensionConfig {
    const config = vscode.workspace.getConfiguration('anchora');
    return {
        filePatterns: config.get<string[]>('filePatterns') || [
            '**/*.rs', '**/*.ts', '**/*.js', '**/*.py', '**/*.java',
            '**/*.cpp', '**/*.c', '**/*.h', '**/*.go'
        ],
        ignoredDirectories: config.get<string[]>('ignoredDirectories') || [
            'target', 'node_modules', '.git', '.vscode', 'dist', 'build', '__pycache__'
        ],
        decorationColors: config.get<Record<string, string>>('decorationColors') || {
            'todo': '#ff6b6b',
            'in_progress': '#4ecdc4',
            'done': '#45b7d1',
            'blocked': '#f9ca24'
        }
    };
}

/**
 * Get the path to the Anchora binary
 */
function getBinaryPath(workspacePath: string): string {
    const binaryName = process.platform === 'win32' ? 'anchora.exe' : 'anchora';
    logDebug(`Platform: ${process.platform}, binary name: ${binaryName}`);
    const extensionBinaryPath = path.join(__dirname, '..', 'server', binaryName);
    logDebug(`Extension binary path: ${extensionBinaryPath}`);
    const fallbackPaths = [
        path.join(workspacePath, 'target', 'release', binaryName),
        path.join(workspacePath, 'target', 'debug', binaryName),
        path.join(workspacePath, binaryName)
    ];
    const allPaths = [extensionBinaryPath, ...fallbackPaths];
    logDebug(`All possible binary paths: ${JSON.stringify(allPaths)}`);
    const selectedPath = allPaths[0] || 'anchora';
    logInfo(`Selected binary path: ${selectedPath}`);
    return selectedPath;
}

/**
 * Handle configuration changes
 */
function handleConfigurationChange(): void {
    console.log('Anchora configuration changed, refreshing...');
    vscode.window.showInformationMessage(
        'Anchora configuration changed. Restart the extension for changes to take effect.',
        'Restart Extension'
    ).then(selection => {
        if (selection === 'Restart Extension') {
            vscode.commands.executeCommand('workbench.action.reloadWindow');
        }
    });
}

/**
 * Build the backend binary
 */
async function buildBackend(workspacePath: string): Promise<void> {
    const terminal = vscode.window.createTerminal({
        name: 'Anchora Build',
        cwd: workspacePath
    });
    terminal.show();
    terminal.sendText('cargo build --release');
    vscode.window.showInformationMessage(
        'Building Anchora backend... Check the terminal for progress.'
    );
}

/**
 * Welcome view provider for non-Anchora projects
 */
class WelcomeViewProvider implements vscode.TreeDataProvider<WelcomeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<WelcomeItem | undefined | null | void> = new vscode.EventEmitter<WelcomeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<WelcomeItem | undefined | null | void> = this._onDidChangeTreeData.event;
    refresh(): void {
        this._onDidChangeTreeData.fire();
    }
    getTreeItem(element: WelcomeItem): vscode.TreeItem {
        return element;
    }
    getChildren(element?: WelcomeItem): Thenable<WelcomeItem[]> {
        if (!element) {
            return Promise.resolve([
                new WelcomeItem(
                    '📋 Welcome to Anchora',
                    'Universal task manager for any project',
                    vscode.TreeItemCollapsibleState.None
                ),
                new WelcomeItem(
                    '🚀 Initialize Anchora Project',
                    'Set up task management for this workspace',
                    vscode.TreeItemCollapsibleState.None,
                    {
                        command: 'anchora.initializeProject',
                        title: 'Initialize Project',
                        arguments: []
                    }
                ),
                new WelcomeItem(
                    '📊 Open Task Dashboard',
                    'Quick access to all task management features',
                    vscode.TreeItemCollapsibleState.None,
                    {
                        command: 'anchora.openTaskDashboard',
                        title: 'Open Dashboard',
                        arguments: []
                    }
                ),
                new WelcomeItem(
                    '🔍 Scan for Tasks',
                    'Search this project for task comments',
                    vscode.TreeItemCollapsibleState.None,
                    {
                        command: 'anchora.scanProject',
                        title: 'Scan Project',
                        arguments: []
                    }
                ),
                new WelcomeItem(
                    '🐛 Show Debug Output',
                    'View extension debug information',
                    vscode.TreeItemCollapsibleState.None,
                    {
                        command: 'anchora.showOutputChannel',
                        title: 'Show Output',
                        arguments: []
                    }
                ),
                new WelcomeItem(
                    '📖 Learn More',
                    'Open Anchora documentation',
                    vscode.TreeItemCollapsibleState.None,
                    {
                        command: 'vscode.open',
                        title: 'Open Documentation',
                        arguments: [vscode.Uri.parse('https://github.com/vremyavnikuda/anchora')]
                    }
                ),
                new WelcomeItem(
                    '⚙️ Requirements',
                    'Works with any project - .anchora/ directory will be created automatically',
                    vscode.TreeItemCollapsibleState.None
                )
            ]);
        }
        return Promise.resolve([]);
    }
}

class WelcomeItem extends vscode.TreeItem {
    constructor(
        label: string,
        tooltip: string,
        collapsibleState: vscode.TreeItemCollapsibleState,
        command?: vscode.Command
    ) {
        super(label, collapsibleState);
        this.tooltip = tooltip;
        this.description = tooltip;
        if (command) {
            this.command = command;
        }
    }
}