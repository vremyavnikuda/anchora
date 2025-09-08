use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use crate::task_manager::ProjectData;
pub struct StorageManager {
    anchora_dir: PathBuf,
    tasks_file: PathBuf,
}

impl StorageManager {
    pub fn new(workspace_path: &Path) -> Self {
        let anchora_dir = workspace_path.join(".anchora");
        let tasks_file = anchora_dir.join("tasks.json");

        Self {
            anchora_dir,
            tasks_file,
        }
    }
    pub async fn initialize(&self) -> anyhow::Result<()> {
        if !self.anchora_dir.exists() {
            async_fs::create_dir_all(&self.anchora_dir).await?;
            println!("Created .anchora directory: {:?}", self.anchora_dir);
        }
        Ok(())
    }
    pub async fn load_project_data(&self) -> anyhow::Result<ProjectData> {
        if !self.tasks_file.exists() {
            // Если файл не существует, создать новый проект
            let project_name = self.anchora_dir
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
                
            return Ok(ProjectData::new(project_name));
        }
        let content = async_fs::read_to_string(&self.tasks_file).await?;
        let project_data: ProjectData = serde_json::from_str(&content)?;
        println!("Loaded project data from: {:?}", self.tasks_file);
        Ok(project_data)
    }
    pub async fn save_project_data(&self, project_data: &ProjectData) -> anyhow::Result<()> {
        self.initialize().await?;
        let json_content = serde_json::to_string_pretty(project_data)?;
        async_fs::write(&self.tasks_file, json_content).await?;
        println!("Saved project data to: {:?}", self.tasks_file);
        Ok(())
    }
    pub async fn create_backup(&self) -> anyhow::Result<PathBuf> {
        if !self.tasks_file.exists() {
            return Err(anyhow::anyhow!("Tasks file does not exist"));
        }
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("tasks_backup_{}.json", timestamp);
        let backup_path = self.anchora_dir.join(backup_name);
        async_fs::copy(&self.tasks_file, &backup_path).await?;
        println!("Created backup: {:?}", backup_path);
        Ok(backup_path)
    }
    pub async fn list_backups(&self) -> anyhow::Result<Vec<PathBuf>> {
        let mut backups = Vec::new();
        if !self.anchora_dir.exists() {
            return Ok(backups);
        }
        let mut entries = async_fs::read_dir(&self.anchora_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("tasks_backup_") && name.ends_with(".json") {
                    backups.push(path);
                }
            }
        }
        backups.sort();
        Ok(backups)
    }
    pub async fn cleanup_old_backups(&self, keep_count: usize) -> anyhow::Result<()> {
        let mut backups = self.list_backups().await?;
        if backups.len() <= keep_count {
            return Ok(());
        }
        backups.sort();
        let to_remove = backups.len() - keep_count;
        for backup in backups.iter().take(to_remove) {
            async_fs::remove_file(backup).await?;
            println!("Removed old backup: {:?}", backup);
        }
        Ok(())
    }
    pub async fn restore_from_backup(&self, backup_path: &Path) -> anyhow::Result<()> {
        if !backup_path.exists() {
            return Err(anyhow::anyhow!("Backup file does not exist: {:?}", backup_path));
        }
        if self.tasks_file.exists() {
            self.create_backup().await?;
        }
        async_fs::copy(backup_path, &self.tasks_file).await?;
        println!("Restored from backup: {:?}", backup_path);
        Ok(())
    }
    pub async fn validate_data_integrity(&self) -> anyhow::Result<bool> {
        if !self.tasks_file.exists() {
            return Ok(true);
        }
        match self.load_project_data().await {
            Ok(_) => Ok(true),
            Err(e) => {
                println!("Data integrity check failed: {}", e);
                Ok(false)
            }
        }
    }
    pub async fn get_storage_info(&self) -> anyhow::Result<StorageInfo> {
        let mut info = StorageInfo {
            anchora_dir_exists: self.anchora_dir.exists(),
            tasks_file_exists: self.tasks_file.exists(),
            tasks_file_size: 0,
            backup_count: 0,
            last_modified: None,
        };
        if info.tasks_file_exists {
            if let Ok(metadata) = async_fs::metadata(&self.tasks_file).await {
                info.tasks_file_size = metadata.len();
                if let Ok(modified) = metadata.modified() {
                    info.last_modified = Some(modified.into());
                }
            }
        }
        info.backup_count = self.list_backups().await?.len();
        Ok(info)
    }
    pub async fn export_data(&self, export_path: &Path) -> anyhow::Result<()> {
        let project_data = self.load_project_data().await?;
        let json_content = serde_json::to_string_pretty(&project_data)?;
        async_fs::write(export_path, json_content).await?;
        println!("Exported data to: {:?}", export_path);
        Ok(())
    }
    pub async fn import_data(&self, import_path: &Path) -> anyhow::Result<()> {
        if !import_path.exists() {
            return Err(anyhow::anyhow!("Import file does not exist: {:?}", import_path));
        }
        if self.tasks_file.exists() {
            self.create_backup().await?;
        }
        let content = async_fs::read_to_string(import_path).await?;
        let project_data: ProjectData = serde_json::from_str(&content)?;
        self.save_project_data(&project_data).await?;
        println!("Imported data from: {:?}", import_path);
        Ok(())
    }
}

#[derive(Debug)]
pub struct StorageInfo {
    pub anchora_dir_exists: bool,
    pub tasks_file_exists: bool,
    pub tasks_file_size: u64,
    pub backup_count: usize,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    #[tokio::test]
    async fn test_storage_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path());
        assert!(!storage.anchora_dir.exists());
        storage.initialize().await.unwrap();
        assert!(storage.anchora_dir.exists());
    }
    #[tokio::test]
    async fn test_save_and_load_project_data() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path());
        let mut project_data = ProjectData::new(Some("test-project".to_string()));
        project_data.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
        storage.save_project_data(&project_data).await.unwrap();
        let loaded_data = storage.load_project_data().await.unwrap();
        assert_eq!(loaded_data.meta.project_name, Some("test-project".to_string()));
        assert!(loaded_data.get_task("dev", "task_1").is_some());
    }

    #[tokio::test]
    async fn test_backup_and_restore() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path());
        let project_data = ProjectData::new(Some("test-project".to_string()));
        storage.save_project_data(&project_data).await.unwrap();
        let backup_path = storage.create_backup().await.unwrap();
        assert!(backup_path.exists());
        let backups = storage.list_backups().await.unwrap();
        assert_eq!(backups.len(), 1);
    }
}