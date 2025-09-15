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
    DeleteTaskParams,
    FindTaskReferencesParams,
    TaskReference,
    ProjectData,
    Note,
    CreateNoteParams,
    CreateNoteResponse,
    GenerateLinkResponse,
    BasicResponse,
    SearchTasksParams,
    SearchResult,
    TaskStatistics,
    TaskOverview,
    ValidateTaskParams,
    ValidationResult,
    GetSuggestionsParams,
    TaskSuggestion,
    CheckConflictsParams,
    ConflictCheck
} from './types';

let clientOutputChannel: vscode.OutputChannel | null = null;
let debugMode = false;

function getOutputChannel(): vscode.OutputChannel {
    if (!clientOutputChannel) {
        clientOutputChannel = vscode.window.createOutputChannel('Anchora Client');
    }
    return clientOutputChannel;
}

export function setDebugMode(enabled: boolean): void {
    debugMode = enabled;
    if (enabled) {
        logClientInfo('Debug mode enabled');
        getOutputChannel().show();
    }
}

export function isDebugMode(): boolean {
    return debugMode;
}

function logClientInfo(message: string): void {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] CLIENT: ${message}`;
    console.log(logMessage);
    getOutputChannel().appendLine(logMessage);

    if (debugMode) {
        console.trace('Client Info Stack:', message);
    }
}

function logClientError(message: string, error?: any): void {
    const timestamp = new Date().toISOString();
    let errorDetails = '';
    let errorStack = '';

    if (error) {
        if (error instanceof Error) {
            errorDetails = ` - ${error.message}`;
            errorStack = error.stack || '';
        } else if (typeof error === 'object') {
            if (error.code && error.message && error.data) {
                errorDetails = ` - [${error.code}] ${error.message}`;
                if (debugMode && error.data) {
                    const enhancedData = error.data;
                    getOutputChannel().appendLine(`Enhanced Error Details:`);
                    if (enhancedData.operation) {
                        getOutputChannel().appendLine(`  Operation: ${enhancedData.operation}`);
                    }
                    if (enhancedData.location) {
                        const loc = enhancedData.location;
                        getOutputChannel().appendLine(`  Location: ${loc.file}:${loc.line}:${loc.column} in ${loc.function}`);
                    }
                    if (enhancedData.method) {
                        getOutputChannel().appendLine(`  Method: ${enhancedData.method}`);
                    }
                    if (enhancedData.timestamp) {
                        getOutputChannel().appendLine(`  Timestamp: ${enhancedData.timestamp}`);
                    }
                    if (enhancedData.error_source) {
                        getOutputChannel().appendLine(`  Error Source: ${enhancedData.error_source}`);
                    }
                    if (enhancedData.error_chain) {
                        getOutputChannel().appendLine(`  Error Chain: ${enhancedData.error_chain}`);
                    }
                    if (enhancedData.additional_data && Object.keys(enhancedData.additional_data).length > 0) {
                        getOutputChannel().appendLine(`  Additional Data: ${JSON.stringify(enhancedData.additional_data, null, 2)}`);
                    }
                }
            } else {
                errorDetails = ` - ${String(error)}`;
            }
        } else {
            errorDetails = ` - ${String(error)}`;
        }
    }
    const logMessage = `[${timestamp}] CLIENT ERROR: ${message}${errorDetails}`;
    console.error(logMessage);
    getOutputChannel().appendLine(logMessage);
    if (errorStack && debugMode) {
        getOutputChannel().appendLine(`Stack trace: ${errorStack}`);
        console.error('Full error object:', error);
    }
    if (debugMode) {
        getOutputChannel().show(true);
    }
}

function logClientDebug(message: string, data?: any): void {
    if (!debugMode) return;
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] CLIENT DEBUG: ${message}`;
    console.debug(logMessage);
    getOutputChannel().appendLine(logMessage);
    if (data !== undefined) {
        const dataStr = typeof data === 'object' ? JSON.stringify(data, null, 2) : String(data);
        getOutputChannel().appendLine(`Data: ${dataStr}`);
        console.debug('Debug data:', data);
    }
}

function logClientWarning(message: string, data?: any): void {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] CLIENT WARNING: ${message}`;
    console.warn(logMessage);
    getOutputChannel().appendLine(logMessage);
    if (data !== undefined && debugMode) {
        const dataStr = typeof data === 'object' ? JSON.stringify(data, null, 2) : String(data);
        getOutputChannel().appendLine(`Warning data: ${dataStr}`);
        console.warn('Warning data:', data);
    }
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
        logClientInfo('=== Starting backend connection process ===');
        try {
            const executablePath = this.binaryPath ?? this.getDefaultBinaryPath();
            logClientInfo(`Attempting to start backend process: ${executablePath}`);
            logClientInfo(`Workspace path: ${this.workspacePath}`);
            logClientDebug('Spawn arguments', {
                executable: executablePath,
                args: ['--workspace', this.workspacePath, '--mode', 'server'],
                stdio: ['pipe', 'pipe', 'pipe']
            });
            this.process = spawn(executablePath, [
                '--workspace', this.workspacePath,
                '--mode', 'server'
            ], {
                stdio: ['pipe', 'pipe', 'pipe']
            });
            if (!this.process.stdout || !this.process.stdin || !this.process.stderr) {
                const error = new BackendConnectionError('Failed to establish stdio pipes with backend process');
                logClientError('Stdio pipes not available', {
                    stdout: !!this.process.stdout,
                    stdin: !!this.process.stdin,
                    stderr: !!this.process.stderr
                });
                throw error;
            }
            logClientInfo('Backend process started, setting up handlers...');
            logClientDebug('Process PID', this.process.pid);
            this.setupProcessHandlers();
            this.setupResponseHandler();
            logClientInfo('Waiting for process to be ready...');
            await this.waitForProcessReady();
            logClientInfo('=== Backend connection established successfully ===');
        } catch (error) {
            logClientError('=== Failed to connect to backend ===', error);
            throw new BackendConnectionError(
                `Failed to start backend process: ${error instanceof Error ? error.message : String(error)}`
            );
        }
    }

    /**
     * Disconnect from the backend process
     */
    async disconnect(): Promise<void> {
        logClientInfo('=== Starting backend disconnection process ===');
        if (this.process) {
            logClientDebug('Cleaning up pending requests', { pendingCount: this.pendingRequests.size });
            for (const [id, request] of this.pendingRequests) {
                clearTimeout(request.timeout);
                request.reject(new BackendConnectionError('Connection closed'));
                logClientDebug(`Cancelled pending request ${id}`);
            }
            this.pendingRequests.clear();
            logClientInfo(`Killing backend process (PID: ${this.process.pid})`);
            this.process.kill();
            this.process = null;
            logClientInfo('=== Backend disconnection completed ===');
        } else {
            logClientInfo('No backend process to disconnect');
        }
    }

    /**
     * Check if client is connected to backend
     */
    isConnected(): boolean {
        return this.process !== null && !this.process.killed;
    }

    /**
     * Send a JSON-RPC request to the backend
     */
    private async sendRequest(method: string, params?: unknown): Promise<unknown> {
        logClientDebug(`=== Sending ${method} request ===`);
        if (!this.isConnected()) {
            const error = new BackendConnectionError('Not connected to backend');
            logClientError('Cannot send request - not connected', { method, connected: false });
            throw error;
        }
        const id = ++this.requestId;
        const request: JsonRpcRequest = {
            jsonrpc: '2.0',
            method,
            params,
            id
        };
        logClientInfo(`Sending request: ${method} (id: ${id})`);
        logClientDebug('Request details', { method, id, params, hasParams: params !== undefined });
        return new Promise((resolve, reject) => {
            const timeout = setTimeout(() => {
                this.pendingRequests.delete(id);
                const timeoutError = new JsonRpcClientError(`Request timeout for method: ${method}`);
                logClientError(`Request timeout for method: ${method}`, { id, method, timeoutMs: this.REQUEST_TIMEOUT });
                reject(timeoutError);
            }, this.REQUEST_TIMEOUT);
            this.pendingRequests.set(id, { resolve, reject, timeout });
            logClientDebug('Request queued', { id, pendingCount: this.pendingRequests.size });
            try {
                const requestJson = JSON.stringify(request) + '\n';
                logClientDebug('Sending JSON', { json: requestJson.trim(), length: requestJson.length });
                this.process!.stdin!.write(requestJson);
                logClientDebug(`Request ${id} sent successfully`);
            } catch (error) {
                clearTimeout(timeout);
                this.pendingRequests.delete(id);
                logClientError('Failed to send request', { error, method, id });
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
            const message = data.toString().trim();
            if (message.startsWith('[DEBUG]')) {
                if (debugMode) {
                    logClientDebug('Backend debug message', { message });
                }
            } else {
                logClientError('Backend stderr', { message });
            }
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
        logClientDebug('=== Handling JSON-RPC response ===', {
            responseLength: responseJson.length,
            response: responseJson.substring(0, 200) + (responseJson.length > 200 ? '...' : '')
        });
        if (!responseJson.startsWith('{') || !responseJson.includes('"jsonrpc"')) {
            logClientDebug('Ignoring non-JSON-RPC message from backend stdout', {
                message: responseJson,
                reason: 'Not a JSON-RPC response'
            });
            return;
        }
        try {
            const response = JSON.parse(responseJson) as JsonRpcResponse;
            logClientDebug('Parsed response', {
                id: response.id,
                hasResult: !!response.result,
                hasError: !!response.error
            });
            if (response.id === null || response.id === undefined) {
                logClientWarning('Received response without ID - ignoring', { response });
                return;
            }
            const id = typeof response.id === 'string' ? parseInt(response.id, 10) : response.id;
            const pendingRequest = this.pendingRequests.get(id);
            if (!pendingRequest) {
                logClientWarning(`Received response for unknown request ID: ${id}`, {
                    id,
                    pendingRequests: Array.from(this.pendingRequests.keys())
                });
                return;
            }
            clearTimeout(pendingRequest.timeout);
            this.pendingRequests.delete(id);
            logClientDebug(`Removed request ${id} from pending queue`, {
                remainingPending: this.pendingRequests.size
            });
            if (response.error) {
                const rpcError = new JsonRpcClientError(
                    `JSON-RPC error: ${response.error.message}`,
                    response.error
                );
                logClientError(`JSON-RPC error for request ${id}`, response.error);
                if (debugMode && response.error.data) {
                    logClientInfo(`Enhanced error data available for debugging`);
                }
                pendingRequest.reject(rpcError);
            } else {
                logClientDebug(`Request ${id} completed successfully`, {
                    hasResult: !!response.result
                });
                pendingRequest.resolve(response.result);
            }
        } catch (error) {
            logClientError('Failed to parse JSON-RPC response', {
                error: error instanceof Error ? error.message : String(error),
                rawResponse: responseJson,
                responseLength: responseJson.length,
                errorType: error instanceof Error ? error.constructor.name : typeof error
            });
            if (debugMode) {
                console.error('Failed to parse JSON-RPC response:', error, 'Raw response:', responseJson);
            }
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
        const binaryName = process.platform === 'win32' ? 'anchora.exe' : 'anchora';
        const serverPath = path.join(__dirname, '..', 'server', binaryName);
        logClientInfo(`Using default binary path: ${serverPath}`);
        return serverPath;
    }

    async scanProject(params: ScanProjectParams): Promise<ScanProjectResult> {
        return await this.sendRequest('scan_project', params) as ScanProjectResult;
    }

    async getTasks(params?: GetTasksParams): Promise<ProjectData> {
        return await this.sendRequest('get_tasks', params) as ProjectData;
    }

    async createTask(params: CreateTaskParams): Promise<{ success: boolean; message: string }> {
        return await this.sendRequest('create_task', params) as { success: boolean; message: string };
    }

    async updateTaskStatus(params: UpdateTaskStatusParams): Promise<{ success: boolean; message: string }> {
        return await this.sendRequest('update_task_status', params) as { success: boolean; message: string };
    }

    async deleteTask(params: DeleteTaskParams): Promise<{ success: boolean; message: string }> {
        return await this.sendRequest('delete_task', params) as { success: boolean; message: string };
    }

    async findTaskReferences(params: FindTaskReferencesParams): Promise<ReadonlyArray<TaskReference>> {
        return await this.sendRequest('find_task_references', params) as ReadonlyArray<TaskReference>;
    }

    async createNote(params: CreateNoteParams): Promise<CreateNoteResponse> {
        return await this.sendRequest('create_note', params) as CreateNoteResponse;
    }

    async getNotes(): Promise<ReadonlyArray<Note>> {
        return await this.sendRequest('get_notes') as ReadonlyArray<Note>;
    }

    async generateTaskLink(noteId: string): Promise<GenerateLinkResponse> {
        return await this.sendRequest('generate_task_link', { note_id: noteId }) as GenerateLinkResponse;
    }

    async deleteNote(noteId: string): Promise<BasicResponse> {
        return await this.sendRequest('delete_note', { note_id: noteId }) as BasicResponse;
    }

    /**
     * Search tasks using server-side indexing and filtering
     */
    async searchTasks(params: SearchTasksParams): Promise<SearchResult> {
        return await this.sendRequest('search_tasks', params) as SearchResult;
    }

    /**
     * Get comprehensive task statistics from server cache
     */
    async getStatistics(): Promise<TaskStatistics> {
        return await this.sendRequest('get_statistics') as TaskStatistics;
    }

    /**
     * Get complete task overview for dashboard
     */
    async getTaskOverview(): Promise<TaskOverview> {
        return await this.sendRequest('get_task_overview') as TaskOverview;
    }

    /**
     * Validate task input with smart suggestions
     */
    async validateTaskInput(params: ValidateTaskParams): Promise<ValidationResult> {
        return await this.sendRequest('validate_task_input', params) as ValidationResult;
    }

    /**
     * Get smart suggestions for auto-completion
     */
    async getSuggestions(params: GetSuggestionsParams): Promise<ReadonlyArray<TaskSuggestion>> {
        return await this.sendRequest('get_suggestions', params) as ReadonlyArray<TaskSuggestion>;
    }

    /**
     * Check for task conflicts and get resolution suggestions
     */
    async checkTaskConflicts(params: CheckConflictsParams): Promise<ConflictCheck> {
        return await this.sendRequest('check_task_conflicts', params) as ConflictCheck;
    }
}