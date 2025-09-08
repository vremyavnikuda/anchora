// Модуль для организации тестов

// Интеграционные тесты
pub mod integration_tests;

// Модульные тесты для каждого компонента
pub mod task_manager_tests;
pub mod file_parser_tests;
pub mod storage_tests;
pub mod communication_tests;

#[cfg(test)]
mod test_helpers {
    use tempfile::TempDir;
    use std::path::PathBuf;
    
    /// Создать временную директорию для тестов
    pub fn create_temp_workspace() -> TempDir {
        TempDir::new().expect("Failed to create temporary directory")
    }
    
    /// Создать тестовый файл с задачами
    pub fn create_test_file_with_tasks(workspace: &std::path::Path, filename: &str, content: &str) -> PathBuf {
        let file_path = workspace.join(filename);
        std::fs::write(&file_path, content).expect("Failed to write test file");
        file_path
    }
    
    /// Получить стандартное содержимое файла с задачами для тестов
    pub fn get_sample_task_content() -> &'static str {
        r#"
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
"#
    }
    
    /// Проверить что временная директория содержит ожидаемые файлы
    pub fn assert_anchora_structure_exists(workspace: &std::path::Path) {
        let anchora_dir = workspace.join(".anchora");
        assert!(anchora_dir.exists(), "Anchora directory should exist");
        assert!(anchora_dir.is_dir(), "Anchora should be a directory");
        
        let tasks_file = anchora_dir.join("tasks.json");
        assert!(tasks_file.exists(), "Tasks file should exist");
    }
    
    /// Создать базовый проект с тестовыми данными
    pub async fn setup_test_project() -> (TempDir, anchora::storage::StorageManager) {
        let temp_dir = create_temp_workspace();
        let storage = anchora::storage::StorageManager::new(temp_dir.path());
        
        // Создать тестовый файл
        create_test_file_with_tasks(
            temp_dir.path(),
            "test.rs",
            get_sample_task_content()
        );
        
        (temp_dir, storage)
    }
}

// Общие импорты для всех тестов
pub use test_helpers::*;