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

export interface SearchTasksParams {
    readonly query: string;
    readonly filters?: {
        readonly sections?: ReadonlyArray<string>;
        readonly statuses?: ReadonlyArray<TaskStatus>;
        readonly include_descriptions?: boolean;
        readonly created_after?: string;
        readonly updated_after?: string;
    };
    readonly limit?: number;
    readonly offset?: number;
}

export interface SearchTaskResult {
    readonly section: string;
    readonly task_id: string;
    readonly title: string;
    readonly description?: string;
    readonly status: TaskStatus;
    readonly relevance_score: number;
    readonly match_type: 'title' | 'description' | 'task_id' | 'section' | 'multiple';
    readonly created: string;
    readonly updated: string;
}

export interface SearchResult {
    readonly tasks: ReadonlyArray<SearchTaskResult>;
    readonly total_count: number;
    readonly filtered_count: number;
    readonly search_time_ms: number;
    readonly suggestions: ReadonlyArray<string>;
    readonly performance_metrics?: {
        readonly search_duration_ms: number;
        readonly total_duration_ms: number;
        readonly operation: string;
        readonly timestamp: string;
    };
}

export interface TaskStatistics {
    readonly total_tasks: number;
    readonly by_status: Record<TaskStatus, number>;
    readonly by_section: Record<string, SectionStats>;
    readonly recent_updates: ReadonlyArray<TaskUpdate>;
    readonly performance_metrics: {
        readonly calculation_time_ms: number;
        readonly cache_hit_rate: number;
        readonly last_cache_update: string;
        readonly total_calculations: number;
    };
    readonly last_calculated: string;
    readonly trends: {
        readonly daily_completions: ReadonlyArray<DailyMetric>;
        readonly section_velocity: Record<string, number>;
        readonly status_transitions: Record<string, number>;
        readonly productivity_score: number;
    };
}

export interface SectionStats {
    readonly total: number;
    readonly by_status: Record<TaskStatus, number>;
    readonly files_count: number;
    readonly avg_tasks_per_file: number;
    readonly last_activity?: string;
    readonly completion_rate: number;
}

export interface TaskUpdate {
    readonly section: string;
    readonly task_id: string;
    readonly old_status?: TaskStatus;
    readonly new_status: TaskStatus;
    readonly timestamp: string;
    readonly change_type: 'created' | 'status_updated' | 'deleted' | 'modified';
}

export interface DailyMetric {
    readonly date: string;
    readonly completed: number;
    readonly created: number;
    readonly total_active: number;
}

export interface TaskOverview {
    readonly sections: ReadonlyArray<SectionSummary>;
    readonly statistics: TaskStatistics;
    readonly recent_activity: ReadonlyArray<TaskActivity>;
    readonly recommendations: ReadonlyArray<string>;
}

export interface SectionSummary {
    readonly name: string;
    readonly total_tasks: number;
    readonly completion_percentage: number;
    readonly active_tasks: number;
    readonly blocked_tasks: number;
    readonly recent_changes: number;
    readonly tasks?: ReadonlyArray<{
        readonly id: string;
        readonly title: string;
        readonly description?: string;
        readonly status: TaskStatus;
        readonly created: string;
        readonly updated: string;
        readonly fileCount: number;
    }>;
}

export interface TaskActivity {
    readonly description: string;
    readonly timestamp: string;
    readonly section: string;
    readonly task_id?: string;
    readonly activity_type: 'task_created' | 'task_completed' | 'status_changed' | 'section_updated';
}

export interface ValidateTaskParams {
    readonly section: string;
    readonly task_id: string;
    readonly title?: string;
    readonly description?: string;
    readonly check_duplicates?: boolean;
    readonly suggest_alternatives?: boolean;
}

export interface ValidationResult {
    readonly is_valid: boolean;
    readonly errors: ReadonlyArray<ValidationError>;
    readonly warnings: ReadonlyArray<ValidationWarning>;
    readonly suggestions: ReadonlyArray<string>;
    readonly alternatives: ReadonlyArray<string>;
    readonly validation_time_ms: number;
}

export interface ValidationError {
    readonly error_type: string;
    readonly field: string;
    readonly message: string;
    readonly suggested_fix?: string;
}

export interface ValidationWarning {
    readonly warning_type: string;
    readonly field: string;
    readonly message: string;
    readonly recommendation?: string;
}

export interface GetSuggestionsParams {
    readonly partial_query: string;
    readonly context?: string;
}

export interface TaskSuggestion {
    readonly text: string;
    readonly suggestion_type: 'task_id' | 'section' | 'keyword' | 'status';
    readonly frequency: number;
    readonly relevance: number;
}

export interface CheckConflictsParams {
    readonly section: string;
    readonly task_id: string;
}

export interface ConflictCheck {
    readonly has_conflicts: boolean;
    readonly conflicts: ReadonlyArray<TaskConflict>;
    readonly resolutions: ReadonlyArray<string>;
}

export interface TaskConflict {
    readonly conflict_type: string;
    readonly existing_task_section: string;
    readonly existing_task_id: string;
    readonly description: string;
    readonly severity: string;
}