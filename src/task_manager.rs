use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub section: String,
    pub suggested_task_id: String,
    pub suggested_status: TaskStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub is_converted: bool,
    pub converted_at: Option<DateTime<Utc>>,
    pub generated_link: Option<String>,
}

impl Note {
    pub fn new(
        title: String,
        content: String,
        section: String,
        suggested_task_id: String,
        suggested_status: Option<TaskStatus>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            content,
            section,
            suggested_task_id,
            suggested_status: suggested_status.unwrap_or(TaskStatus::Todo),
            created: now,
            updated: now,
            is_converted: false,
            converted_at: None,
            generated_link: None,
        }
    }

    pub fn generate_task_link(&mut self) -> String {
        let status_str = match self.suggested_status {
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
            TaskStatus::Blocked => "blocked",
        };
        let link = format!(
            "// {}:{}:{}: {}",
            self.section,
            self.suggested_task_id,
            status_str,
            self.title
        );
        self.generated_link = Some(link.clone());
        self.updated = Utc::now();
        link
    }

    pub fn mark_as_converted(&mut self) {
        self.is_converted = true;
        self.converted_at = Some(Utc::now());
        self.updated = Utc::now();
    }
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
    #[serde(default)]
    pub notes: HashMap<String, Note>,
}

impl ProjectData {
    pub fn new(project_name: Option<String>) -> Self {
        let mut meta = ProjectMeta::default();
        meta.project_name = project_name;
        Self {
            meta,
            sections: HashMap::new(),
            index: TaskIndex::new(),
            notes: HashMap::new(),
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
        if !self.sections.contains_key(section) {
            return Err(anyhow::anyhow!("Section not found: {}", section));
        }
        let section_tasks = self.sections.get_mut(section).unwrap();
        if !section_tasks.contains_key(task_id) {
            return Err(anyhow::anyhow!("Task not found: {}:{}", section, task_id));
        }
        section_tasks.remove(task_id);
        if section_tasks.is_empty() {
            self.sections.remove(section);
        }
        self.meta.last_updated = Utc::now();
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

    pub fn add_note(
        &mut self,
        title: String,
        content: String,
        section: String,
        suggested_task_id: String,
        suggested_status: Option<TaskStatus>,
    ) -> anyhow::Result<String> {
        let note = Note::new(title, content, section, suggested_task_id, suggested_status);
        let note_id = note.id.clone();
        if self.notes.contains_key(&note_id) {
            return Err(anyhow::anyhow!("Note with ID '{}' already exists", note_id));
        }
        self.notes.insert(note_id.clone(), note);
        self.meta.last_updated = Utc::now();
        Ok(note_id)
    }

    pub fn get_note(&self, id: &str) -> Option<&Note> {
        self.notes.get(id)
    }

    pub fn get_note_mut(&mut self, id: &str) -> Option<&mut Note> {
        self.notes.get_mut(id)
    }

    pub fn update_note(
        &mut self,
        id: &str,
        title: Option<String>,
        content: Option<String>,
    ) -> anyhow::Result<()> {
        let note = self.notes.get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Note with ID '{}' not found", id))?;
        if let Some(title) = title {
            note.title = title;
        }
        if let Some(content) = content {
            note.content = content;
        }
        note.updated = Utc::now();
        self.meta.last_updated = Utc::now();
        Ok(())
    }

    pub fn delete_note(&mut self, id: &str) -> anyhow::Result<()> {
        if !self.notes.contains_key(id) {
            return Err(anyhow::anyhow!("Note with ID '{}' not found", id));
        }
        self.notes.remove(id);
        self.meta.last_updated = Utc::now();
        Ok(())
    }

    pub fn generate_note_link(&mut self, note_id: &str) -> anyhow::Result<String> {
        let note = self.notes.get_mut(note_id)
            .ok_or_else(|| anyhow::anyhow!("Note with ID '{}' not found", note_id))?;
        if note.is_converted {
            return Err(anyhow::anyhow!("Note is already converted to task"));
        }
        let link = note.generate_task_link();
        self.meta.last_updated = Utc::now();
        Ok(link)
    }

    pub fn convert_note_to_task(&mut self, note_id: &str) -> anyhow::Result<()> {
        let note = self.notes.get(note_id)
            .ok_or_else(|| anyhow::anyhow!("Note with ID '{}' not found", note_id))?;
        if note.is_converted {
            return Err(anyhow::anyhow!("Note is already converted to task"));
        }
        let note_clone = note.clone();
        let task = Task::new(note_clone.title, Some(note_clone.content));
        self.sections
            .entry(note_clone.section.clone())
            .or_insert_with(HashMap::new)
            .insert(note_clone.suggested_task_id.clone(), task.clone());
        if let Some(task) = self.get_task_mut(&note_clone.section, &note_clone.suggested_task_id) {
            task.update_status(note_clone.suggested_status);
        }
        if let Some(note) = self.notes.get_mut(note_id) {
            note.mark_as_converted();
        }
        self.meta.last_updated = Utc::now();
        self.rebuild_index();
        Ok(())
    }

    pub fn get_all_notes(&self) -> Vec<&Note> {
        self.notes.values().collect()
    }

    pub fn check_note_conversions(&mut self, scanned_content: &[(String, String)]) -> anyhow::Result<Vec<String>> {
        let mut converted_notes = Vec::new();
        for (note_id, note) in self.notes.clone() {
            if note.is_converted || note.generated_link.is_none() {
                continue;
            }
            let generated_link = note.generated_link.as_ref().unwrap();
            for (_, content) in scanned_content {
                if content.contains(generated_link) {
                    if let Err(e) = self.convert_note_to_task(&note_id) {
                        eprintln!("Error converting note to task: {}", e);
                    } else {
                        converted_notes.push(note_id.clone());
                    }
                    break;
                }
            }
        }
        Ok(converted_notes)
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
        project.add_task("dev", "task_1", "Test task".to_string(), None).unwrap();
        assert!(project.get_task("dev", "task_1").is_some());
        project.delete_task("dev", "task_1").unwrap();
        assert!(project.get_task("dev", "task_1").is_none());
        let result = project.delete_task("dev", "task_1");
        assert!(result.is_err());
        let result = project.delete_task("nonexistent", "task_1");
        assert!(result.is_err());
    }
}