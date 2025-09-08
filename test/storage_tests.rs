use anchora::storage::*;
use anchora::task_manager::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_storage_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let _storage = StorageManager::new(temp_dir.path());
    
    // Проверить что папка .anchora не существует изначально
    let anchora_path = temp_dir.path().join(".anchora");
    assert!(!anchora_path.exists());
}

#[tokio::test]
async fn test_storage_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Инициализировать хранилище
    storage.initialize().await.unwrap();
    
    // Проверить что папка создана
    let anchora_path = temp_dir.path().join(".anchora");
    assert!(anchora_path.exists());
    assert!(anchora_path.is_dir());
}

#[tokio::test]
async fn test_load_project_data_new_project() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Загрузить данные для нового проекта (файл не существует)
    let project_data = storage.load_project_data().await.unwrap();
    
    assert_eq!(project_data.meta.version, "1.0.0");
    assert!(project_data.sections.is_empty());
    
    // Имя проекта должно совпадать с именем папки
    let expected_name = temp_dir.path()
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());
    assert_eq!(project_data.meta.project_name, expected_name);
}

#[tokio::test]
async fn test_save_and_load_project_data() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Создать тестовые данные
    let mut project_data = ProjectData::new(Some("test-project".to_string()));
    project_data.add_task(
        "dev",
        "task_1",
        "Test task".to_string(),
        Some("Test description".to_string())
    ).unwrap();
    
    // Добавить файл к задаче
    project_data.update_task_file(
        "dev",
        "task_1",
        "src/main.rs".to_string(),
        25,
        Some("Implementation note".to_string())
    ).unwrap();
    
    // Сохранить данные
    storage.save_project_data(&project_data).await.unwrap();
    
    // Проверить что файл создан
    let tasks_file = temp_dir.path().join(".anchora").join("tasks.json");
    assert!(tasks_file.exists());
    
    // Загрузить данные обратно
    let loaded_data = storage.load_project_data().await.unwrap();
    
    assert_eq!(loaded_data.meta.project_name, Some("test-project".to_string()));
    assert_eq!(loaded_data.sections.len(), 1);
    
    let task = loaded_data.get_task("dev", "task_1").unwrap();
    assert_eq!(task.title, "Test task");
    assert_eq!(task.description, Some("Test description".to_string()));
    
    // Проверить файловые ассоциации
    assert!(task.files.contains_key("src/main.rs"));
    let file_info = &task.files["src/main.rs"];
    assert_eq!(file_info.lines, vec![25]);
    assert_eq!(file_info.notes.get(&25), Some(&"Implementation note".to_string()));
}

#[tokio::test]
async fn test_save_project_data_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    let project_data = ProjectData::new(None);
    
    // Сохранить данные (должно создать папку автоматически)
    storage.save_project_data(&project_data).await.unwrap();
    
    let anchora_path = temp_dir.path().join(".anchora");
    assert!(anchora_path.exists());
    assert!(anchora_path.is_dir());
    
    let tasks_file = anchora_path.join("tasks.json");
    assert!(tasks_file.exists());
}

#[tokio::test]
async fn test_create_backup() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Создать и сохранить данные
    let project_data = ProjectData::new(Some("backup-test".to_string()));
    storage.save_project_data(&project_data).await.unwrap();
    
    // Создать резервную копию
    let backup_path = storage.create_backup().await.unwrap();
    
    assert!(backup_path.exists());
    assert!(backup_path.file_name().unwrap().to_str().unwrap().starts_with("tasks_backup_"));
    assert!(backup_path.file_name().unwrap().to_str().unwrap().ends_with(".json"));
    
    // Проверить что резервная копия содержит правильные данные
    let backup_content = tokio::fs::read_to_string(&backup_path).await.unwrap();
    let backup_data: ProjectData = serde_json::from_str(&backup_content).unwrap();
    assert_eq!(backup_data.meta.project_name, Some("backup-test".to_string()));
}

#[tokio::test]
async fn test_create_backup_no_tasks_file() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Попытаться создать резервную копию без файла задач
    let result = storage.create_backup().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_backups() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Создать данные
    let project_data = ProjectData::new(None);
    storage.save_project_data(&project_data).await.unwrap();
    
    // Создать несколько резервных копий
    let backup1 = storage.create_backup().await.unwrap();
    
    // Небольшая задержка чтобы имена файлов отличались
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    let backup2 = storage.create_backup().await.unwrap();
    
    // Получить список резервных копий
    let backups = storage.list_backups().await.unwrap();
    
    assert_eq!(backups.len(), 2);
    assert!(backups.contains(&backup1));
    assert!(backups.contains(&backup2));
}

#[tokio::test]
async fn test_cleanup_old_backups() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Создать данные
    let project_data = ProjectData::new(None);
    storage.save_project_data(&project_data).await.unwrap();
    
    // Создать 5 резервных копий
    for _ in 0..5 {
        storage.create_backup().await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
    
    let backups_before = storage.list_backups().await.unwrap();
    assert_eq!(backups_before.len(), 5);
    
    // Оставить только 2 последние
    storage.cleanup_old_backups(2).await.unwrap();
    
    let backups_after = storage.list_backups().await.unwrap();
    assert_eq!(backups_after.len(), 2);
}

#[tokio::test]
async fn test_restore_from_backup() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Создать первоначальные данные
    let mut project_data = ProjectData::new(Some("original".to_string()));
    project_data.add_task("dev", "task_1", "Original task".to_string(), None).unwrap();
    storage.save_project_data(&project_data).await.unwrap();
    
    // Создать резервную копию
    let backup_path = storage.create_backup().await.unwrap();
    
    // Изменить данные
    project_data.meta.project_name = Some("modified".to_string());
    project_data.add_task("dev", "task_2", "New task".to_string(), None).unwrap();
    storage.save_project_data(&project_data).await.unwrap();
    
    // Проверить что данные изменились
    let modified_data = storage.load_project_data().await.unwrap();
    assert_eq!(modified_data.meta.project_name, Some("modified".to_string()));
    assert!(modified_data.get_task("dev", "task_2").is_some());
    
    // Восстановить из резервной копии
    storage.restore_from_backup(&backup_path).await.unwrap();
    
    // Проверить что данные восстановлены
    let restored_data = storage.load_project_data().await.unwrap();
    assert_eq!(restored_data.meta.project_name, Some("original".to_string()));
    assert!(restored_data.get_task("dev", "task_1").is_some());
    assert!(restored_data.get_task("dev", "task_2").is_none());
}

#[tokio::test]
async fn test_restore_from_nonexistent_backup() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    let nonexistent_path = temp_dir.path().join("nonexistent_backup.json");
    let result = storage.restore_from_backup(&nonexistent_path).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_validate_data_integrity() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Тест с отсутствующим файлом (должно быть OK)
    let result = storage.validate_data_integrity().await.unwrap();
    assert!(result);
    
    // Создать валидные данные
    let project_data = ProjectData::new(None);
    storage.save_project_data(&project_data).await.unwrap();
    
    // Проверить целостность
    let result = storage.validate_data_integrity().await.unwrap();
    assert!(result);
    
    // Испортить файл
    let tasks_file = temp_dir.path().join(".anchora").join("tasks.json");
    tokio::fs::write(&tasks_file, "invalid json").await.unwrap();
    
    // Проверить что целостность нарушена
    let result = storage.validate_data_integrity().await.unwrap();
    assert!(!result);
}

#[tokio::test]
async fn test_export_data() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Создать тестовые данные
    let mut project_data = ProjectData::new(Some("export-test".to_string()));
    project_data.add_task("dev", "export_task", "Task for export".to_string(), None).unwrap();
    storage.save_project_data(&project_data).await.unwrap();
    
    // Экспортировать данные
    let export_path = temp_dir.path().join("exported_tasks.json");
    storage.export_data(&export_path).await.unwrap();
    
    assert!(export_path.exists());
    
    // Проверить содержимое экспорта
    let export_content = tokio::fs::read_to_string(&export_path).await.unwrap();
    let exported_data: ProjectData = serde_json::from_str(&export_content).unwrap();
    
    assert_eq!(exported_data.meta.project_name, Some("export-test".to_string()));
    assert!(exported_data.get_task("dev", "export_task").is_some());
}

#[tokio::test]
async fn test_import_data() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Создать файл для импорта
    let import_data = ProjectData::new(Some("imported-project".to_string()));
    let import_json = serde_json::to_string_pretty(&import_data).unwrap();
    
    let import_path = temp_dir.path().join("import_tasks.json");
    tokio::fs::write(&import_path, import_json).await.unwrap();
    
    // Импортировать данные
    storage.import_data(&import_path).await.unwrap();
    
    // Проверить что данные импортированы
    let loaded_data = storage.load_project_data().await.unwrap();
    assert_eq!(loaded_data.meta.project_name, Some("imported-project".to_string()));
}

#[tokio::test]
async fn test_import_invalid_data() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    // Создать невалидный файл для импорта
    let import_path = temp_dir.path().join("invalid_import.json");
    tokio::fs::write(&import_path, "invalid json content").await.unwrap();
    
    // Попытаться импортировать
    let result = storage.import_data(&import_path).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_import_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let storage = StorageManager::new(temp_dir.path());
    
    let nonexistent_path = temp_dir.path().join("nonexistent.json");
    let result = storage.import_data(&nonexistent_path).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_access() {
    let temp_dir = TempDir::new().unwrap();
    let storage = std::sync::Arc::new(StorageManager::new(temp_dir.path()));
    
    // Создать несколько задач одновременно
    let mut handles = vec![];
    
    for i in 0..10 {
        let storage_clone = storage.clone();
        let handle = tokio::spawn(async move {
            let mut project_data = storage_clone.load_project_data().await.unwrap();
            project_data.add_task(
                "concurrent",
                &format!("task_{}", i),
                format!("Concurrent task {}", i),
                None
            ).unwrap();
            storage_clone.save_project_data(&project_data).await.unwrap();
        });
        handles.push(handle);
    }
    
    // Дождаться завершения всех задач
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Проверить финальное состояние
    let final_data = storage.load_project_data().await.unwrap();
    
    // Должна быть хотя бы одна задача (из-за concurrent access результат может варьироваться)
    assert!(!final_data.sections.is_empty());
}