/**
 * JSON-RPC Client for communicating with Anchora Rust backend
 * Implements strict typing and error handling as per project rules
 */

import { spawn, ChildProcess } from 'node:child_process';
import * as path from 'node:path';
import * as vscode from 'vscode';
import {
    JsonRpcRequest,
    JsonRpcResponse,
    BackendConnectionError,
    JsonRpcClientError,
    ScanProjectParams,
    ScanProjectResult,
    GetTasksParams,
    CreateTaskParams,
    UpdateTaskStatusParams,
    FindTaskReferencesParams,
    TaskReference,
    ProjectData
} from './types';

let clientOutputChannel: vscode.OutputChannel | null = null;

function getOutputChannel(): vscode.OutputChannel {
    if (!clientOutputChannel) {
        clientOutputChannel = vscode.window.createOutputChannel('Anchora Client');
    }
    return clientOutputChannel;
}

function logClientInfo(message: string): void {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] CLIENT: ${message}`;
    console.log(logMessage);
    getOutputChannel().appendLine(logMessage);
}

function logClientError(message: string, error?: any): void {
    const timestamp = new Date().toISOString();
    const errorDetails = error ? ` - ${error instanceof Error ? error.message : String(error)}` : '';
    const logMessage = `[${timestamp}] CLIENT ERROR: ${message}${errorDetails}`;
    console.error(logMessage);
    getOutputChannel().appendLine(logMessage);
}

export class JsonRpcClient {
    private process: ChildProcess | null = null;
    private requestId = 0;
    private readonly pendingRequests = new Map<number, {
        resolve: (value: unknown) => void;
        reject: (error: Error) => void;
        timeout: ReturnType<typeof setTimeout>;
    }>();
    private readonly REQUEST_TIMEOUT = 30000;

    constructor(
        private readonly workspacePath: string,
        private readonly binaryPath?: string
    ) { }

    /**
     * Start the backend process and establish connection
     */
    async connect(): Promise<void> {
        try {
            const executablePath = this.binaryPath ?? this.getDefaultBinaryPath();
            logClientInfo(`Attempting to start backend process: ${executablePath}`);
            logClientInfo(`Workspace path: ${this.workspacePath}`);
            this.process = spawn(executablePath, [
                '--workspace', this.workspacePath,
                '--mode', 'server'
            ], {
                stdio: ['pipe', 'pipe', 'pipe']
            });
            if (!this.process.stdout || !this.process.stdin || !this.process.stderr) {
                throw new BackendConnectionError('Failed to establish stdio pipes with backend process');
            }
            logClientInfo('Backend process started, setting up handlers...');
            this.setupProcessHandlers();
            this.setupResponseHandler();
            logClientInfo('Waiting for process to be ready...');
            await this.waitForProcessReady();
            logClientInfo('Backend connection established successfully');
        } catch (error) {
            logClientError('Failed to connect to backend', error);
            throw new BackendConnectionError(
                `Failed to start backend process: ${error instanceof Error ? error.message : String(error)}`
            );
        }
    }

    /**
     * Disconnect from the backend process
     */
    async disconnect(): Promise<void> {
        if (this.process) {
            for (const [, request] of this.pendingRequests) {
                clearTimeout(request.timeout);
                request.reject(new BackendConnectionError('Connection closed'));
            }
            this.pendingRequests.clear();
            this.process.kill();
            this.process = null;
        }
    }

    /**
     * Check if client is connected to backend
     */
    isConnected(): boolean {
        return this.process !== null && !this.process.killed;
    }

    /**
     * Scan project for tasks
     */
    async scanProject(params: ScanProjectParams): Promise<ScanProjectResult> {
        const response = await this.sendRequest('scan_project', params);
        return response as ScanProjectResult;
    }

    /**
     * Get all tasks or filtered tasks
     */
    async getTasks(params?: GetTasksParams): Promise<ProjectData['sections']> {
        const response = await this.sendRequest('get_tasks', params);
        return response as ProjectData['sections'];
    }

    /**
     * Create a new task
     */
    async createTask(params: CreateTaskParams): Promise<{ success: boolean; message: string }> {
        const response = await this.sendRequest('create_task', params);
        return response as { success: boolean; message: string };
    }

    /**
     * Update task status
     */
    async updateTaskStatus(params: UpdateTaskStatusParams): Promise<{ success: boolean; message: string }> {
        const response = await this.sendRequest('update_task_status', params);
        return response as { success: boolean; message: string };
    }

    /**
     * Find all references to a task
     */
    async findTaskReferences(params: FindTaskReferencesParams): Promise<ReadonlyArray<TaskReference>> {
        const response = await this.sendRequest('find_task_references', params);
        return response as ReadonlyArray<TaskReference>;
    }

    /**
     * Send a JSON-RPC request to the backend
     */
    private async sendRequest(method: string, params?: unknown): Promise<unknown> {
        if (!this.isConnected()) {
            throw new BackendConnectionError('Not connected to backend');
        }
        const id = ++this.requestId;
        const request: JsonRpcRequest = {
            jsonrpc: '2.0',
            method,
            params,
            id
        };
        logClientInfo(`Sending request: ${method} (id: ${id})`);
        return new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
                this.pendingRequests.delete(id);
                logClientError(`Request timeout for method: ${method}`);
                reject(new JsonRpcClientError(`Request timeout for method: ${method}`));
            }, this.REQUEST_TIMEOUT);
            this.pendingRequests.set(id, { resolve, reject, timeout });
            try {
                const requestJson = JSON.stringify(request) + '\n';
                this.process!.stdin!.write(requestJson);
            } catch (error) {
                clearTimeout(timeout);
                this.pendingRequests.delete(id);
                logClientError('Failed to send request', error);
                reject(new BackendConnectionError(
                    `Failed to send request: ${error instanceof Error ? error.message : String(error)}`
                ));
            }
        });
    }

    /**
     * Set up process event handlers
     */
    private setupProcessHandlers(): void {
        if (!this.process) return;

        this.process.on('error', (error: Error) => {
            logClientError('Backend process error', error);
            this.handleProcessError(error);
        });

        this.process.on('exit', (code: number | null, signal: string | null) => {
            logClientInfo(`Backend process exited with code ${code}, signal ${signal}`);
            this.handleProcessExit();
        });

        this.process.stderr?.on('data', (data: Buffer) => {
            logClientError('Backend stderr', data.toString());
        });
    }

    /**
     * Set up response handler for incoming JSON-RPC responses
     */
    private setupResponseHandler(): void {
        if (!this.process?.stdout) return;
        let buffer = '';
        this.process.stdout.on('data', (data: Buffer) => {
            buffer += data.toString();
            let lineEnd;
            while ((lineEnd = buffer.indexOf('\n')) !== -1) {
                const line = buffer.slice(0, lineEnd).trim();
                buffer = buffer.slice(lineEnd + 1);

                if (line) {
                    this.handleResponse(line);
                }
            }
        });
    }

    /**
     * Handle incoming JSON-RPC response
     */
    private handleResponse(responseJson: string): void {
        try {
            const response = JSON.parse(responseJson) as JsonRpcResponse;
            if (response.id === null || response.id === undefined) {
                return;
            }
            const id = typeof response.id === 'string' ? parseInt(response.id, 10) : response.id;
            const pendingRequest = this.pendingRequests.get(id);
            if (!pendingRequest) {
                console.warn(`Received response for unknown request ID: ${id}`);
                return;
            }
            clearTimeout(pendingRequest.timeout);
            this.pendingRequests.delete(id);
            if (response.error) {
                pendingRequest.reject(new JsonRpcClientError(
                    `JSON-RPC error: ${response.error.message}`,
                    response.error
                ));
            } else {
                pendingRequest.resolve(response.result);
            }
        } catch (error) {
            console.error('Failed to parse JSON-RPC response:', error, 'Raw response:', responseJson);
        }
    }

    /**
     * Handle process errors
     */
    private handleProcessError(error: Error): void {
        for (const [, request] of this.pendingRequests) {
            clearTimeout(request.timeout);
            request.reject(new BackendConnectionError(`Process error: ${error.message}`));
        }
        this.pendingRequests.clear();
    }

    /**
     * Handle process exit
     */
    private handleProcessExit(): void {
        for (const [, request] of this.pendingRequests) {
            clearTimeout(request.timeout);
            request.reject(new BackendConnectionError('Backend process exited'));
        }
        this.pendingRequests.clear();
        this.process = null;
    }

    /**
     * Wait for process to be ready (simple implementation)
     */
    private async waitForProcessReady(): Promise<void> {
        await new Promise(resolve => setTimeout(resolve, 1000));
    }

    /**
     * Get the default path to the Anchora binary
     */
    private getDefaultBinaryPath(): string {
        const possiblePaths = [
            path.join(this.workspacePath, 'target', 'release', 'anchora.exe'),
            path.join(this.workspacePath, 'target', 'debug', 'anchora.exe'),
            path.join(this.workspacePath, 'anchora.exe'),
            'anchora.exe',
            'anchora'
        ];
        return possiblePaths[0] || 'anchora';
    }
}