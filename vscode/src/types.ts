/**
 * Type definitions for Anchora VSCode Extension
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

export type TaskStatus = 'todo' | 'in_progress' | 'done' | 'blocked';

export interface Note {
    readonly id: string;
    readonly title: string;
    readonly content: string;
    readonly section: string;
    readonly suggested_task_id: string;
    readonly suggested_status: TaskStatus;
    readonly created: string;
    readonly updated: string;
    readonly is_converted: boolean;
    readonly converted_at?: string;
    readonly generated_link?: string;
}

export interface CreateNoteParams {
    readonly title: string;
    readonly content: string;
    readonly section: string;
    readonly suggested_task_id: string;
    readonly suggested_status?: TaskStatus;
}

export interface GenerateLinkParams {
    readonly note_id: string;
}

export interface DeleteNoteParams {
    readonly note_id: string;
}

export interface CreateNoteResponse {
    readonly success: boolean;
    readonly message: string;
    readonly note_id: string;
}

export interface GenerateLinkResponse {
    readonly success: boolean;
    readonly link: string;
}

export interface BasicResponse {
    readonly success: boolean;
    readonly message: string;
}

export interface TaskFile {
    readonly lines: ReadonlyArray<number>;
    readonly notes: Record<number, string>;
}

export interface Task {
    readonly title: string;
    readonly description?: string;
    readonly status: TaskStatus;
    readonly created: string;
    readonly updated: string;
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
    readonly notes?: Record<string, Note>;
}

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

export interface DeleteTaskParams {
    readonly section: string;
    readonly task_id: string;
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