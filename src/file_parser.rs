use crate::task_manager::{ProjectData, TaskStatus};
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedTaskLabel {
    pub section: String,
    pub task_id: String,
    pub status: Option<TaskStatus>,
    pub description: Option<String>,
    pub note: Option<String>,
}
pub struct TaskParser {
    full_definition_regex: Regex,
    with_status_regex: Regex,
    simple_reference_regex: Regex,
    with_note_regex: Regex,
    status_update_regex: Regex,
}
impl TaskParser {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            full_definition_regex: Regex::new(
                r"//\s*([\p{L}_][\p{L}\p{N}_]*):([\p{L}_][\p{L}\p{N}_]*):\s+(.+)",
            )?,
            with_status_regex: Regex::new(
                r"//\s*([\p{L}_][\p{L}\p{N}_]*):([\p{L}_][\p{L}\p{N}_]*):([\p{L}_][\p{L}\p{N}_]*):\s+(.+)",
            )?,
            simple_reference_regex: Regex::new(
                r"//\s*([\p{L}_][\p{L}\p{N}_]*):([\p{L}_][\p{L}\p{N}_]*)\s*$",
            )?,
            with_note_regex: Regex::new(
                r"//\s*([\p{L}_][\p{L}\p{N}_]*):([\p{L}_][\p{L}\p{N}_]*):([\p{L}\p{N}_]+)\s*$",
            )?,
            status_update_regex: Regex::new(
                r"(?i)//\s*([\p{L}_][\p{L}\p{N}_]*):([\p{L}_][\p{L}\p{N}_]*):(todo|in_progress|inprogress|progress|done|completed|complete|blocked|block)\s*$",
            )?,
        })
    }
    pub fn parse_line(&self, line: &str) -> Option<ParsedTaskLabel> {
        let line = line.trim();
        if line.starts_with("/*")
            || line.ends_with("*/")
            || line.starts_with("* ")
            || line.starts_with("*")
        {
            return None;
        }
        if let Some(captures) = self.with_status_regex.captures(line) {
            let section = captures.get(1)?.as_str().to_string();
            let task_id = captures.get(2)?.as_str().to_string();
            let status_str = captures.get(3)?.as_str();
            let description = captures.get(4)?.as_str().to_string();
            if let Some(status) = self.parse_status(status_str) {
                return Some(ParsedTaskLabel {
                    section,
                    task_id,
                    status: Some(status),
                    description: Some(description),
                    note: None,
                });
            }
        }
        if let Some(captures) = self.full_definition_regex.captures(line) {
            let section = captures.get(1)?.as_str().to_string();
            let task_id = captures.get(2)?.as_str().to_string();
            let description = captures.get(3)?.as_str().to_string();
            return Some(ParsedTaskLabel {
                section,
                task_id,
                status: None,
                description: Some(description),
                note: None,
            });
        }
        if let Some(captures) = self.status_update_regex.captures(line) {
            let section = captures.get(1)?.as_str().to_string();
            let task_id = captures.get(2)?.as_str().to_string();
            let status_str = captures.get(3)?.as_str();
            if let Some(status) = self.parse_status(status_str) {
                return Some(ParsedTaskLabel {
                    section,
                    task_id,
                    status: Some(status),
                    description: None,
                    note: None,
                });
            }
        }
        if let Some(captures) = self.with_note_regex.captures(line) {
            let section = captures.get(1)?.as_str().to_string();
            let task_id = captures.get(2)?.as_str().to_string();
            let note = captures.get(3)?.as_str().to_string();
            if self.parse_status(&note).is_none() {
                return Some(ParsedTaskLabel {
                    section,
                    task_id,
                    status: None,
                    description: None,
                    note: Some(note),
                });
            }
        }
        if let Some(captures) = self.simple_reference_regex.captures(line) {
            let section = captures.get(1)?.as_str().to_string();
            let task_id = captures.get(2)?.as_str().to_string();

            return Some(ParsedTaskLabel {
                section,
                task_id,
                status: None,
                description: None,
                note: None,
            });
        }

        None
    }
    fn parse_status(&self, status_str: &str) -> Option<TaskStatus> {
        match status_str.to_lowercase().as_str() {
            "todo" => Some(TaskStatus::Todo),
            "in_progress" | "inprogress" | "progress" => Some(TaskStatus::InProgress),
            "done" | "completed" | "complete" => Some(TaskStatus::Done),
            "blocked" | "block" => Some(TaskStatus::Blocked),
            _ => None,
        }
    }
    pub fn scan_file(
        &self,
        _file_path: &str,
        content: &str,
    ) -> anyhow::Result<Vec<(u32, ParsedTaskLabel)>> {
        let mut results = Vec::new();
        for (line_number, line) in content.lines().enumerate() {
            if let Some(parsed_label) = self.parse_line(line) {
                results.push((line_number as u32 + 1, parsed_label));
            }
        }
        Ok(results)
    }
    pub fn update_project_from_labels(
        &self,
        project_data: &mut ProjectData,
        file_path: &str,
        labels: Vec<(u32, ParsedTaskLabel)>,
    ) -> anyhow::Result<()> {
        for (_section_name, section) in &mut project_data.sections {
            for (_task_id, task) in section {
                if let Some(task_file) = task.files.get_mut(file_path) {
                    task_file.lines.clear();
                    task_file.notes.clear();
                }
            }
        }
        for (line_number, label) in labels {
            if let Some(description) = &label.description {
                if project_data
                    .get_task(&label.section, &label.task_id)
                    .is_none()
                {
                    project_data.add_task(
                        &label.section,
                        &label.task_id,
                        description.clone(),
                        None,
                    )?;
                }
                if let Some(status) = label.status.clone() {
                    project_data.update_task_status(&label.section, &label.task_id, status)?;
                }
            }
            project_data.update_task_file(
                &label.section,
                &label.task_id,
                file_path.to_string(),
                line_number,
                label.note.clone(),
            )?;
            if label.description.is_none() && label.status.is_some() {
                if let Some(status) = label.status.clone() {
                    project_data.update_task_status(&label.section, &label.task_id, status)?;
                }
            }
        }

        Ok(())
    }
}
#[derive(Debug)]
pub struct ScanResult {
    pub files_scanned: u32,
    pub tasks_found: u32,
    pub tasks_removed: u32,
    pub errors: Vec<String>,
}
impl ScanResult {
    pub fn new() -> Self {
        Self {
            files_scanned: 0,
            tasks_found: 0,
            tasks_removed: 0,
            errors: Vec::new(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_full_definition() {
        let parser = TaskParser::new().unwrap();
        let result = parser.parse_line("// dev:task_1: добавить функционал проверки");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.section, "dev");
        assert_eq!(parsed.task_id, "task_1");
        assert_eq!(
            parsed.description,
            Some("добавить функционал проверки".to_string())
        );
        assert_eq!(parsed.status, None);
    }
    #[test]
    fn test_parse_with_status() {
        let parser = TaskParser::new().unwrap();
        let result = parser.parse_line("// dev:task_1:todo: добавить функционал проверки");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.section, "dev");
        assert_eq!(parsed.task_id, "task_1");
        assert_eq!(
            parsed.description,
            Some("добавить функционал проверки".to_string())
        );
        assert_eq!(parsed.status, Some(TaskStatus::Todo));
    }
    #[test]
    fn test_parse_simple_reference() {
        let parser = TaskParser::new().unwrap();
        let result = parser.parse_line("// dev:task_1");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.section, "dev");
        assert_eq!(parsed.task_id, "task_1");
        assert_eq!(parsed.description, None);
        assert_eq!(parsed.status, None);
    }
    #[test]
    fn test_parse_with_note() {
        let parser = TaskParser::new().unwrap();
        let result = parser.parse_line("// dev:task_1:основная_логика");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.section, "dev");
        assert_eq!(parsed.task_id, "task_1");
        assert_eq!(parsed.note, Some("основная_логика".to_string()));
    }
    #[test]
    fn test_parse_status_update() {
        let parser = TaskParser::new().unwrap();
        let result = parser.parse_line("// dev:task_1:done");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.section, "dev");
        assert_eq!(parsed.task_id, "task_1");
        assert_eq!(parsed.status, Some(TaskStatus::Done));
    }
    #[test]
    fn test_scan_file() {
        let parser = TaskParser::new().unwrap();
        let content = r#"
fn main() {
    // dev:task_1: добавить функционал проверки
    println!("Hello, world!");
    // dev:task_1
    let x = 42;
    // ref:cleanup: провести рефакторинг
}
"#;

        let results = parser.scan_file("test.rs", content).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0, 3);
        assert_eq!(results[0].1.section, "dev");
    }
}
