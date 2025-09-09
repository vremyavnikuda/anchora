use tempfile::TempDir;

#[tokio::test]
async fn test_full_workflow() {
    // Создать временную директорию для тестов
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();
    
    // Создать тестовый файл с задачами
    let test_file_content = r#"
fn main() {
    // dev:task_1: добавить новый функционал проверки на ошибки
    println!("Hello, world!");
    
    // dev:task_1
    let x = 42;
    
    // ref:cleanup_task: провести рефакторинг парсера
    println!("Testing task parser");
    
    // dev:task_2:todo: реализовать автосохранение
    let auto_save = true;
    
    // dev:task_2:основная_логика
    if auto_save {
        println!("Auto save enabled");
    }
    
    // dev:task_1:done
    println!("Task 1 completed");
}
"#;
    
    let test_file_path = workspace_path.join("test_file.rs");
    std::fs::write(&test_file_path, test_file_content).unwrap();
    
    // Инициализировать компоненты системы
    let storage = anchora::storage::StorageManager::new(workspace_path);
    let parser = anchora::file_parser::TaskParser::new().unwrap();
    
    // Загрузить данные проекта
    let mut project_data = storage.load_project_data().await.unwrap();
    
    // Сканировать файл
    let labels = parser.scan_file(
        test_file_path.to_str().unwrap(),
        test_file_content
    ).unwrap();
    
    assert_eq!(labels.len(), 6, "Should find 6 task labels");
    
    // Обновить данные проекта
    parser.update_project_from_labels(
        &mut project_data,
        test_file_path.to_str().unwrap(),
        labels
    ).unwrap();
    
    // Проверить результаты
    assert!(project_data.get_task("dev", "task_1").is_some());
    assert!(project_data.get_task("dev", "task_2").is_some());
    assert!(project_data.get_task("ref", "cleanup_task").is_some());
    
    // Проверить статусы
    let task_1 = project_data.get_task("dev", "task_1").unwrap();
    assert_eq!(task_1.status, anchora::task_manager::TaskStatus::Done);
    
    let task_2 = project_data.get_task("dev", "task_2").unwrap();
    assert_eq!(task_2.status, anchora::task_manager::TaskStatus::Todo);
    
    // Сохранить данные
    storage.save_project_data(&project_data).await.unwrap();
    
    // Проверить что файл создался
    let anchora_dir = workspace_path.join(".anchora");
    let tasks_file = anchora_dir.join("tasks.json");
    assert!(tasks_file.exists());
    
    // Загрузить данные обратно и проверить
    let loaded_data = storage.load_project_data().await.unwrap();
    assert_eq!(loaded_data.sections.len(), 2); // dev и ref разделы
    
    println!("✅ Full workflow test passed!");
}

#[tokio::test] 
async fn test_task_status_updates() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();
    
    let storage = anchora::storage::StorageManager::new(workspace_path);
    let mut project_data = storage.load_project_data().await.unwrap();
    
    // Создать задачу
    project_data.add_task(
        "dev",
        "test_task",
        "Test task for status updates".to_string(),
        Some("Detailed description".to_string())
    ).unwrap();
    
    // Проверить начальный статус
    let task = project_data.get_task("dev", "test_task").unwrap();
    assert_eq!(task.status, anchora::task_manager::TaskStatus::Todo);
    
    // Обновить статус
    project_data.update_task_status(
        "dev", 
        "test_task", 
        anchora::task_manager::TaskStatus::InProgress
    ).unwrap();
    
    let task = project_data.get_task("dev", "test_task").unwrap();
    assert_eq!(task.status, anchora::task_manager::TaskStatus::InProgress);
    
    // Завершить задачу
    project_data.update_task_status(
        "dev",
        "test_task", 
        anchora::task_manager::TaskStatus::Done
    ).unwrap();
    
    let task = project_data.get_task("dev", "test_task").unwrap();
    assert_eq!(task.status, anchora::task_manager::TaskStatus::Done);
    
    println!("✅ Task status updates test passed!");
}

#[tokio::test]
async fn test_file_associations() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();
    
    let storage = anchora::storage::StorageManager::new(workspace_path);
    let mut project_data = storage.load_project_data().await.unwrap();
    
    // Создать задачу
    project_data.add_task(
        "dev",
        "multi_file_task",
        "Task spanning multiple files".to_string(),
        None
    ).unwrap();
    
    // Добавить файлы к задаче
    project_data.update_task_file(
        "dev",
        "multi_file_task",
        "src/main.rs".to_string(),
        25,
        Some("Main implementation".to_string())
    ).unwrap();
    
    project_data.update_task_file(
        "dev",
        "multi_file_task", 
        "src/lib.rs".to_string(),
        15,
        None
    ).unwrap();
    
    project_data.update_task_file(
        "dev",
        "multi_file_task",
        "tests/integration.rs".to_string(),
        42,
        Some("Test coverage".to_string())
    ).unwrap();
    
    // Проверить файловые ассоциации
    let task = project_data.get_task("dev", "multi_file_task").unwrap();
    assert_eq!(task.files.len(), 3);
    
    assert!(task.files.contains_key("src/main.rs"));
    assert!(task.files.contains_key("src/lib.rs"));
    assert!(task.files.contains_key("tests/integration.rs"));
    
    // Проверить заметки
    let main_file = &task.files["src/main.rs"];
    assert_eq!(main_file.notes.get(&25), Some(&"Main implementation".to_string()));
    
    let test_file = &task.files["tests/integration.rs"];
    assert_eq!(test_file.notes.get(&42), Some(&"Test coverage".to_string()));
    
    // Пересоздать индекс
    project_data.rebuild_index();
    
    // Проверить индекс файлов
    assert!(project_data.index.files.contains_key("src/main.rs"));
    assert!(project_data.index.files.contains_key("src/lib.rs"));
    assert!(project_data.index.files.contains_key("tests/integration.rs"));
    
    println!("✅ File associations test passed!");
}

#[test]
fn test_task_label_parsing() {
    let parser = anchora::file_parser::TaskParser::new().unwrap();
    
    // Тест полного определения
    let result = parser.parse_line("// dev:task_1: добавить функционал проверки");
    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.section, "dev");
    assert_eq!(parsed.task_id, "task_1");
    assert_eq!(parsed.description, Some("добавить функционал проверки".to_string()));
    
    // Тест с статусом
    let result = parser.parse_line("// dev:task_1:todo: описание задачи");
    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.status, Some(anchora::task_manager::TaskStatus::Todo));
    
    // Тест простой ссылки
    let result = parser.parse_line("// dev:task_1");
    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.section, "dev");
    assert_eq!(parsed.task_id, "task_1");
    assert!(parsed.description.is_none());
    
    // Тест с заметкой
    let result = parser.parse_line("// dev:task_1:important_note");
    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.note, Some("important_note".to_string()));
    
    // Тест обновления статуса
    let result = parser.parse_line("// dev:task_1:done");
    assert!(result.is_some());
    let parsed = result.unwrap();
    assert_eq!(parsed.status, Some(anchora::task_manager::TaskStatus::Done));
    
    // Тест невалидного формата
    let result = parser.parse_line("// invalid format");
    assert!(result.is_none());
    
    println!("✅ Task label parsing test passed!");
}

#[tokio::test]
async fn test_storage_backup_restore() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();
    
    let storage = anchora::storage::StorageManager::new(workspace_path);
    let mut project_data = storage.load_project_data().await.unwrap();
    
    // Создать тестовые данные
    project_data.add_task(
        "test",
        "backup_task",
        "Task for backup testing".to_string(),
        None
    ).unwrap();
    
    // Сохранить первоначальные данные
    storage.save_project_data(&project_data).await.unwrap();
    
    // Создать резервную копию
    let backup_path = storage.create_backup().await.unwrap();
    assert!(backup_path.exists());
    
    // Изменить данные
    project_data.update_task_status(
        "test",
        "backup_task",
        anchora::task_manager::TaskStatus::Done
    ).unwrap();
    storage.save_project_data(&project_data).await.unwrap();
    
    // Восстановить из бэкапа
    storage.restore_from_backup(&backup_path).await.unwrap();
    
    // Проверить что данные восстановились  
    let restored_data = storage.load_project_data().await.unwrap();
    let restored_task = restored_data.get_task("test", "backup_task").unwrap();
    assert_eq!(restored_task.status, anchora::task_manager::TaskStatus::Todo); // Должен быть исходный статус
    
    println!("✅ Storage backup and restore test passed!");
}

#[tokio::test]
async fn test_delete_task_integration() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();
    
    let storage = anchora::storage::StorageManager::new(workspace_path);
    let mut project_data = anchora::task_manager::ProjectData::new(Some("test-project".to_string()));
    
    // 1. Create a task
    project_data.add_task(
        "test", 
        "delete_me", 
        "Task to delete".to_string(), 
        Some("Test description".to_string())
    ).unwrap();
    
    storage.save_project_data(&project_data).await.unwrap();
    
    // 2. Verify task exists
    let loaded_project = storage.load_project_data().await.unwrap();
    assert!(loaded_project.get_task("test", "delete_me").is_some());
    
    // 3. Delete the task
    let mut updated_project = loaded_project;
    let result = updated_project.delete_task("test", "delete_me");
    assert!(result.is_ok());
    
    storage.save_project_data(&updated_project).await.unwrap();
    
    // 4. Verify task is gone
    let final_project = storage.load_project_data().await.unwrap();
    assert!(final_project.get_task("test", "delete_me").is_none());
    
    // 5. Test error cases
    let mut error_project = final_project;
    let result = error_project.delete_task("test", "nonexistent");
    assert!(result.is_err());
    
    let result = error_project.delete_task("nonexistent", "any");
    assert!(result.is_err());
    
    println!("✅ Delete integration test passed!");
}
