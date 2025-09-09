use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    #[serde(rename = "todo")]
    Todo,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "blocked")]
    Blocked,
}
impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Todo
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFile {
    pub lines: Vec<u32>,
    pub notes: HashMap<u32, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub files: HashMap<String, TaskFile>,
}

impl Task {
    pub fn new(title: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            title,
            description,
            status: TaskStatus::default(),
            created: now,
            updated: now,
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, file_path: String, line: u32, note: Option<String>) {
        let task_file = self.files.entry(file_path).or_insert_with(|| TaskFile {
            lines: Vec::new(),
            notes: HashMap::new(),
        });
        if !task_file.lines.contains(&line) {
            task_file.lines.push(line);
        }
        if let Some(note) = note {
            task_file.notes.insert(line, note);
        }
        self.updated = Utc::now();
    }

    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated = Utc::now();
    }
}

pub type TaskSection = HashMap<String, Task>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskIndex {
    pub files: HashMap<String, Vec<String>>,
    pub tasks_by_status: HashMap<TaskStatus, Vec<String>>,
}

impl TaskIndex {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            tasks_by_status: HashMap::new(),
        }
    }
    pub fn update_task(&mut self, section: &str, task_id: &str, task: &Task) {
        let full_task_id = format!("{}.{}", section, task_id);
        for file_path in task.files.keys() {
            self.files
                .entry(file_path.clone())
                .or_insert_with(Vec::new)
                .push(full_task_id.clone());
        }
        self.tasks_by_status
            .entry(task.status.clone())
            .or_insert_with(Vec::new)
            .push(full_task_id);
    }
    pub fn clear(&mut self) {
        self.files.clear();
        self.tasks_by_status.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub version: String,
    pub created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub project_name: Option<String>,
}

impl Default for ProjectMeta {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: "1.0.0".to_string(),
            created: now,
            last_updated: now,
            project_name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub meta: ProjectMeta,
    pub sections: HashMap<String, TaskSection>,
    pub index: TaskIndex,
}

impl ProjectData {
    pub fn new(project_name: Option<String>) -> Self {
        let mut meta = ProjectMeta::default();
        meta.project_name = project_name;
        Self {
            meta,
            sections: HashMap::new(),
            index: TaskIndex::new(),
        }
    }
    pub fn add_task(&mut self, section: &str, task_id: &str, title: String, description: Option<String>) -> anyhow::Result<()> {
        let task = Task::new(title, description);
        self.sections
            .entry(section.to_string())
            .or_insert_with(HashMap::new)
            .insert(task_id.to_string(), task.clone());
        self.index.update_task(section, task_id, &task);
        self.meta.last_updated = Utc::now();
        Ok(())
    }

    pub fn get_task(&self, section: &str, task_id: &str) -> Option<&Task> {
        self.sections.get(section)?.get(task_id)
    }

    pub fn get_task_mut(&mut self, section: &str, task_id: &str) -> Option<&mut Task> {
        self.sections.get_mut(section)?.get_mut(task_id)
    }

    pub fn update_task_file(&mut self, section: &str, task_id: &str, file_path: String, line: u32, note: Option<String>) -> anyhow::Result<()> {
        if let Some(task) = self.get_task_mut(section, task_id) {
            task.add_file(file_path, line, note);
            self.meta.last_updated = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found: {}:{}", section, task_id))
        }
    }

    pub fn update_task_status(&mut self, section: &str, task_id: &str, status: TaskStatus) -> anyhow::Result<()> {
        if let Some(task) = self.get_task_mut(section, task_id) {
            task.update_status(status);
            self.meta.last_updated = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found: {}:{}", section, task_id))
        }
    }

    pub fn delete_task(&mut self, section: &str, task_id: &str) -> anyhow::Result<()> {
        // Check if task exists
        if !self.sections.contains_key(section) {
            return Err(anyhow::anyhow!("Section not found: {}", section));
        }
        
        let section_tasks = self.sections.get_mut(section).unwrap();
        if !section_tasks.contains_key(task_id) {
            return Err(anyhow::anyhow!("Task not found: {}:{}", section, task_id));
        }
        
        // Remove the task
        section_tasks.remove(task_id);
        
        // If section is now empty, remove it
        if section_tasks.is_empty() {
            self.sections.remove(section);
        }
        
        // Update metadata
        self.meta.last_updated = Utc::now();
        
        // Rebuild index to ensure consistency
        self.rebuild_index();
        
        Ok(())
    }

    pub fn rebuild_index(&mut self) {
        self.index.clear();
        for (section_name, section) in &self.sections {
            for (task_id, task) in section {
                self.index.update_task(section_name, task_id, task);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_task_creation() {
        let task = Task::new("Test task".to_string(), Some("Description".to_string()));
        assert_eq!(task.title, "Test task");
        assert_eq!(task.description, Some("Description".to_string()));
        assert_eq!(task.status, TaskStatus::Todo);
    }

    #[test]
    fn test_project_data() {
        let mut project = ProjectData::new(Some("test-project".to_string()));
        project.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
        let task = project.get_task("dev", "task_1").unwrap();
        assert_eq!(task.title, "Test task");
    }

    #[test]
    fn test_delete_task() {
        let mut project = ProjectData::new(Some("test-project".to_string()));
        
        // Add a task
        project.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
        assert!(project.get_task("dev", "task_1").is_some());
        
        // Delete the task
        project.delete_task("dev", "task_1").unwrap();
        assert!(project.get_task("dev", "task_1").is_none());
        
        // Try to delete non-existent task
        let result = project.delete_task("dev", "task_1");
        assert!(result.is_err());
        
        // Try to delete from non-existent section
        let result = project.delete_task("nonexistent", "task_1");
        assert!(result.is_err());
    }
}