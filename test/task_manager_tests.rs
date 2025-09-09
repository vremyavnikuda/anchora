use anchora::task_manager::*;

#[test]
fn test_task_creation() {
    let task = Task::new(
        "Test task".to_string(),
        Some("Test description".to_string())
    );
    
    assert_eq!(task.title, "Test task");
    assert_eq!(task.description, Some("Test description".to_string()));
    assert_eq!(task.status, TaskStatus::Todo);
    assert!(task.files.is_empty());
}

#[test]
fn test_task_add_file() {
    let mut task = Task::new("Test task".to_string(), None);
    
    task.add_file(
        "src/main.rs".to_string(),
        25,
        Some("Main implementation".to_string())
    );
    
    assert_eq!(task.files.len(), 1);
    assert!(task.files.contains_key("src/main.rs"));
    
    let file_info = &task.files["src/main.rs"];
    assert_eq!(file_info.lines, vec![25]);
    assert_eq!(file_info.notes.get(&25), Some(&"Main implementation".to_string()));
}

#[test]
fn test_task_add_multiple_lines_same_file() {
    let mut task = Task::new("Test task".to_string(), None);
    
    task.add_file("src/main.rs".to_string(), 25, None);
    task.add_file("src/main.rs".to_string(), 35, Some("Second reference".to_string()));
    task.add_file("src/main.rs".to_string(), 25, Some("Updated note".to_string())); // Duplicate line
    
    let file_info = &task.files["src/main.rs"];
    assert_eq!(file_info.lines.len(), 2); // Should have 2 unique lines
    assert!(file_info.lines.contains(&25));
    assert!(file_info.lines.contains(&35));
    
    // Note should be updated for line 25
    assert_eq!(file_info.notes.get(&25), Some(&"Updated note".to_string()));
    assert_eq!(file_info.notes.get(&35), Some(&"Second reference".to_string()));
}

#[test]
fn test_task_status_update() {
    let mut task = Task::new("Test task".to_string(), None);
    let initial_time = task.updated;
    
    // Небольшая задержка чтобы время обновления изменилось
    std::thread::sleep(std::time::Duration::from_millis(1));
    
    task.update_status(TaskStatus::InProgress);
    
    assert_eq!(task.status, TaskStatus::InProgress);
    assert!(task.updated > initial_time);
}

#[test]
fn test_project_data_creation() {
    let project = ProjectData::new(Some("test-project".to_string()));
    
    assert_eq!(project.meta.project_name, Some("test-project".to_string()));
    assert_eq!(project.meta.version, "1.0.0");
    assert!(project.sections.is_empty());
}

#[test]
fn test_project_add_task() {
    let mut project = ProjectData::new(None);
    
    let result = project.add_task(
        "dev",
        "task_1",
        "Test task".to_string(),
        Some("Description".to_string())
    );
    
    assert!(result.is_ok());
    assert!(project.get_task("dev", "task_1").is_some());
    
    let task = project.get_task("dev", "task_1").unwrap();
    assert_eq!(task.title, "Test task");
    assert_eq!(task.description, Some("Description".to_string()));
}

#[test]
fn test_project_update_task_file() {
    let mut project = ProjectData::new(None);
    
    project.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
    
    let result = project.update_task_file(
        "dev",
        "task_1",
        "src/main.rs".to_string(),
        42,
        Some("Implementation note".to_string())
    );
    
    assert!(result.is_ok());
    
    let task = project.get_task("dev", "task_1").unwrap();
    assert!(task.files.contains_key("src/main.rs"));
    
    let file_info = &task.files["src/main.rs"];
    assert_eq!(file_info.lines, vec![42]);
    assert_eq!(file_info.notes.get(&42), Some(&"Implementation note".to_string()));
}

#[test]
fn test_project_update_task_status() {
    let mut project = ProjectData::new(None);
    
    project.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
    
    let result = project.update_task_status("dev", "task_1", TaskStatus::Done);
    assert!(result.is_ok());
    
    let task = project.get_task("dev", "task_1").unwrap();
    assert_eq!(task.status, TaskStatus::Done);
}

#[test]
fn test_project_update_nonexistent_task() {
    let mut project = ProjectData::new(None);
    
    let result = project.update_task_status("dev", "nonexistent", TaskStatus::Done);
    assert!(result.is_err());
    
    let result = project.update_task_file(
        "dev",
        "nonexistent",
        "file.rs".to_string(),
        1,
        None
    );
    assert!(result.is_err());
}

#[test]
fn test_task_index_creation() {
    let mut index = TaskIndex::new();
    
    let task = Task::new("Test task".to_string(), None);
    index.update_task("dev", "task_1", &task);
    
    // Index should be empty since task has no files
    assert!(index.files.is_empty());
    assert_eq!(index.tasks_by_status.get(&TaskStatus::Todo).unwrap().len(), 1);
}

#[test]
fn test_task_index_with_files() {
    let mut index = TaskIndex::new();
    let mut task = Task::new("Test task".to_string(), None);
    
    task.add_file("src/main.rs".to_string(), 25, None);
    task.add_file("src/lib.rs".to_string(), 10, None);
    
    index.update_task("dev", "task_1", &task);
    
    assert_eq!(index.files.len(), 2);
    assert!(index.files.contains_key("src/main.rs"));
    assert!(index.files.contains_key("src/lib.rs"));
    
    let main_tasks = index.files.get("src/main.rs").unwrap();
    assert!(main_tasks.contains(&"dev.task_1".to_string()));
}

#[test]
fn test_project_rebuild_index() {
    let mut project = ProjectData::new(None);
    
    // Добавить задачи
    project.add_task("dev", "task_1", "Task 1".to_string(), None).unwrap();
    project.add_task("ref", "task_2", "Task 2".to_string(), None).unwrap();
    
    // Добавить файлы
    project.update_task_file("dev", "task_1", "file1.rs".to_string(), 10, None).unwrap();
    project.update_task_file("ref", "task_2", "file2.rs".to_string(), 20, None).unwrap();
    project.update_task_file("dev", "task_1", "file2.rs".to_string(), 15, None).unwrap();
    
    // Обновить статус
    project.update_task_status("dev", "task_1", TaskStatus::Done).unwrap();
    
    // Очистить индекс и пересоздать
    project.index.clear();
    project.rebuild_index();
    
    // Проверить индекс файлов
    assert_eq!(project.index.files.len(), 2);
    assert!(project.index.files.contains_key("file1.rs"));
    assert!(project.index.files.contains_key("file2.rs"));
    
    let file2_tasks = project.index.files.get("file2.rs").unwrap();
    assert_eq!(file2_tasks.len(), 2); // Обе задачи ссылаются на file2.rs
    
    // Проверить индекс статусов
    let done_tasks = project.index.tasks_by_status.get(&TaskStatus::Done).unwrap();
    assert!(done_tasks.contains(&"dev.task_1".to_string()));
    
    let todo_tasks = project.index.tasks_by_status.get(&TaskStatus::Todo).unwrap();
    assert!(todo_tasks.contains(&"ref.task_2".to_string()));
}

#[test]
fn test_task_status_serialization() {
    // Тест сериализации статусов
    let todo = TaskStatus::Todo;
    let json = serde_json::to_string(&todo).unwrap();
    assert_eq!(json, "\"todo\"");
    
    let in_progress = TaskStatus::InProgress;
    let json = serde_json::to_string(&in_progress).unwrap();
    assert_eq!(json, "\"in_progress\"");
    
    let done = TaskStatus::Done;
    let json = serde_json::to_string(&done).unwrap();
    assert_eq!(json, "\"done\"");
    
    let blocked = TaskStatus::Blocked;
    let json = serde_json::to_string(&blocked).unwrap();
    assert_eq!(json, "\"blocked\"");
}

#[test]
fn test_task_status_deserialization() {
    // Тест десериализации статусов
    let todo: TaskStatus = serde_json::from_str("\"todo\"").unwrap();
    assert_eq!(todo, TaskStatus::Todo);
    
    let in_progress: TaskStatus = serde_json::from_str("\"in_progress\"").unwrap();
    assert_eq!(in_progress, TaskStatus::InProgress);
    
    let done: TaskStatus = serde_json::from_str("\"done\"").unwrap();
    assert_eq!(done, TaskStatus::Done);
    
    let blocked: TaskStatus = serde_json::from_str("\"blocked\"").unwrap();
    assert_eq!(blocked, TaskStatus::Blocked);
}

#[test]
fn test_delete_task_basic() {
    let mut project = ProjectData::new(Some("test-project".to_string()));
    
    // Add a task
    project.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
    assert!(project.get_task("dev", "task_1").is_some());
    
    // Delete the task
    let result = project.delete_task("dev", "task_1");
    assert!(result.is_ok());
    assert!(project.get_task("dev", "task_1").is_none());
}

#[test]
fn test_delete_task_with_files() {
    let mut project = ProjectData::new(None);
    
    // Add task with files
    project.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
    project.update_task_file("dev", "task_1", "src/main.rs".to_string(), 42, None).unwrap();
    project.update_task_file("dev", "task_1", "src/lib.rs".to_string(), 25, None).unwrap();
    
    // Verify task exists with files
    let task = project.get_task("dev", "task_1").unwrap();
    assert_eq!(task.files.len(), 2);
    
    // Delete the task
    let result = project.delete_task("dev", "task_1");
    assert!(result.is_ok());
    assert!(project.get_task("dev", "task_1").is_none());
}

#[test]
fn test_delete_task_updates_index() {
    let mut project = ProjectData::new(None);
    
    // Add multiple tasks
    project.add_task("dev", "task_1", "Task 1".to_string(), None).unwrap();
    project.add_task("dev", "task_2", "Task 2".to_string(), None).unwrap();
    project.add_task("bug", "task_3", "Task 3".to_string(), None).unwrap();
    
    // Add files to tasks
    project.update_task_file("dev", "task_1", "file1.rs".to_string(), 10, None).unwrap();
    project.update_task_file("dev", "task_2", "file1.rs".to_string(), 20, None).unwrap();
    
    // Verify index before deletion
    assert_eq!(project.sections.len(), 2);
    assert_eq!(project.sections["dev"].len(), 2);
    assert_eq!(project.sections["bug"].len(), 1);
    
    // Delete one task from dev section
    let result = project.delete_task("dev", "task_1");
    assert!(result.is_ok());
    
    // Verify section still exists (has task_2)
    assert_eq!(project.sections.len(), 2);
    assert_eq!(project.sections["dev"].len(), 1);
    assert!(project.sections["dev"].contains_key("task_2"));
    
    // Delete last task from dev section
    let result = project.delete_task("dev", "task_2");
    assert!(result.is_ok());
    
    // Verify dev section is removed when empty
    assert_eq!(project.sections.len(), 1);
    assert!(!project.sections.contains_key("dev"));
    assert!(project.sections.contains_key("bug"));
}

#[test]
fn test_delete_nonexistent_task() {
    let mut project = ProjectData::new(None);
    
    // Try to delete from non-existent section
    let result = project.delete_task("nonexistent", "task_1");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Section not found"));
    
    // Add section but try to delete non-existent task
    project.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
    let result = project.delete_task("dev", "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Task not found"));
}

#[test]
fn test_delete_task_updates_timestamp() {
    let mut project = ProjectData::new(None);
    
    project.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
    let initial_timestamp = project.meta.last_updated;
    
    // Small delay to ensure timestamp changes
    std::thread::sleep(std::time::Duration::from_millis(1));
    
    let result = project.delete_task("dev", "task_1");
    assert!(result.is_ok());
    assert!(project.meta.last_updated > initial_timestamp);
}