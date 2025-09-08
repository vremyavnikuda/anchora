/**
 * Type definitions for Anchora Task Manager VSCode Extension
 * Following strict TypeScript practices as per project rules
 */

export interface JsonRpcRequest {
    readonly jsonrpc: '2.0';
    readonly method: string;
    readonly params?: unknown;
    readonly id?: string | number | null;
}

export interface JsonRpcResponse {
    readonly jsonrpc: '2.0';
    readonly result?: unknown;
    readonly error?: JsonRpcError;
    readonly id?: string | number | null;
}

export interface JsonRpcError {
    readonly code: number;
    readonly message: string;
    readonly data?: unknown;
}

// Task Management Types
export type TaskStatus = 'todo' | 'in_progress' | 'done' | 'blocked';

export interface TaskFile {
    readonly lines: ReadonlyArray<number>;
    readonly notes: Record<number, string>;
}

export interface Task {
    readonly title: string;
    readonly description?: string;
    readonly status: TaskStatus;
    readonly created: string; // ISO 8601 date
    readonly updated: string; // ISO 8601 date
    readonly files: Record<string, TaskFile>;
}

export interface TaskSection {
    readonly [taskId: string]: Task;
}

export interface ProjectData {
    readonly meta: {
        readonly version: string;
        readonly created: string;
        readonly last_updated: string;
        readonly project_name?: string;
    };
    readonly sections: Record<string, TaskSection>;
    readonly index: {
        readonly files: Record<string, ReadonlyArray<string>>;
        readonly tasks_by_status: Record<TaskStatus, ReadonlyArray<string>>;
    };
}

// Request/Response Parameter Types
export interface ScanProjectParams {
    readonly workspace_path: string;
    readonly file_patterns?: ReadonlyArray<string>;
}

export interface ScanProjectResult {
    readonly files_scanned: number;
    readonly tasks_found: number;
    readonly errors: ReadonlyArray<string>;
}

export interface GetTasksParams {
    readonly section?: string;
    readonly status?: TaskStatus;
}

export interface CreateTaskParams {
    readonly section: string;
    readonly task_id: string;
    readonly title: string;
    readonly description?: string;
}

export interface UpdateTaskStatusParams {
    readonly section: string;
    readonly task_id: string;
    readonly status: TaskStatus;
}

export interface FindTaskReferencesParams {
    readonly section: string;
    readonly task_id: string;
}

export interface TaskReference {
    readonly file_path: string;
    readonly line: number;
    readonly note?: string;
}

// VSCode Extension Types
export interface TaskTreeItem {
    readonly type: 'section' | 'task' | 'file';
    readonly label: string;
    readonly section?: string;
    readonly taskId?: string;
    readonly filePath?: string;
    readonly line?: number;
    readonly status?: TaskStatus;
    readonly description?: string;
}

export interface ExtensionConfig {
    readonly filePatterns: ReadonlyArray<string>;
    readonly ignoredDirectories: ReadonlyArray<string>;
    readonly decorationColors: Record<TaskStatus, string>;
}

// Error Types
export class AnchoraError extends Error {
    constructor(message: string, public readonly code?: number) {
        super(message);
        this.name = 'AnchoraError';
    }
}

export class JsonRpcClientError extends AnchoraError {
    constructor(message: string, public readonly rpcError?: JsonRpcError) {
        super(message);
        this.name = 'JsonRpcClientError';
    }
}

export class BackendConnectionError extends AnchoraError {
    constructor(message: string) {
        super(message);
        this.name = 'BackendConnectionError';
    }
}

// Utility types for strict typing
export type NonEmptyString = string & { readonly __brand: 'NonEmptyString' };
export type FilePath = string & { readonly __brand: 'FilePath' };
export type TaskId = string & { readonly __brand: 'TaskId' };
export type SectionName = string & { readonly __brand: 'SectionName' };

export function isNonEmptyString(value: string): value is NonEmptyString {
    return value.trim().length > 0;
}

export function createTaskId(value: string): TaskId | null {
    return isNonEmptyString(value) ? (value as unknown as TaskId) : null;
}

export function createSectionName(value: string): SectionName | null {
    return isNonEmptyString(value) ? (value as unknown as SectionName) : null;
}