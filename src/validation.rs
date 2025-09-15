/*!
 * Validation Engine Module for Anchora Backend
 * 
 * Provides server-side validation for task creation and management with:
 * - Smart task ID validation
 * - Duplicate detection
 * - Conflict resolution suggestions
 * - Context-aware validation rules
 */

use std::collections::HashSet;
use std::sync::RwLock;
use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::task_manager::ProjectData;
use anyhow::Result;

/// Parameters for task validation
#[derive(Debug, Deserialize)]
pub struct ValidationParams {
    pub section: String,
    pub task_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub check_duplicates: Option<bool>,
    pub suggest_alternatives: Option<bool>,
}

/// Result of validation with errors, warnings, and suggestions
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
    pub alternative_ids: Vec<String>,
}

/// Validation error with specific details
#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub error_type: String,
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Validation warning for potential issues
#[derive(Debug, Serialize)]
pub struct ValidationWarning {
    pub warning_type: String,
    pub field: String,
    pub message: String,
    pub recommendation: Option<String>,
}

/// Conflict detection result
#[derive(Debug, Serialize)]
pub struct ConflictCheck {
    pub has_conflicts: bool,
    pub conflicts: Vec<Conflict>,
    pub resolutions: Vec<String>,
}

/// Individual conflict detected
#[derive(Debug, Serialize)]
pub struct Conflict {
    pub conflict_type: String,
    pub existing_task_section: String,
    pub existing_task_id: String,
    pub description: String,
    pub severity: String,
}

/// Validation engine configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub max_task_id_length: usize,
    pub min_task_id_length: usize,
    pub max_title_length: usize,
    pub max_description_length: usize,
    pub enable_smart_suggestions: bool,
    pub similarity_threshold: f32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_task_id_length: 50,
            min_task_id_length: 2,
            max_title_length: 200,
            max_description_length: 2000,
            enable_smart_suggestions: true,
            similarity_threshold: 0.8,
        }
    }
}

/// Validation engine with smart rules and suggestions
pub struct ValidationEngine {
    project_data: RwLock<Option<ProjectData>>,
    reserved_names: HashSet<String>,
    config: ValidationConfig,
    task_id_pattern: Regex,
}

impl ValidationEngine {
    /// Create a new validation engine
    pub fn new(config: Option<ValidationConfig>) -> Self {
        let config = config.unwrap_or_default();
        let reserved_names = Self::create_reserved_names();
        let task_id_pattern = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*$").unwrap();
        
        Self {
            project_data: RwLock::new(None),
            reserved_names,
            config,
            task_id_pattern,
        }
    }

    /// Update project data context for validation
    pub fn update_context(&self, project_data: ProjectData) -> Result<()> {
        let mut data = self.project_data.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock on project data"))?;
        *data = Some(project_data);
        Ok(())
    }

    /// Validate task creation parameters
    pub fn validate_task_creation(&self, params: &ValidationParams) -> Result<ValidationResult> {
        let start_time = std::time::Instant::now();
        
        let project_data = self.project_data.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on project data"))?;
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        let mut alternative_ids = Vec::new();
        
        if let Some(error) = self.validate_task_id_format(&params.task_id) {
            errors.push(error);
        }
        
        if self.reserved_names.contains(&params.task_id.to_lowercase()) {
            errors.push(ValidationError {
                error_type: "reserved_name".to_string(),
                field: "task_id".to_string(),
                message: format!("'{}' is a reserved name and cannot be used as task ID", params.task_id),
                suggestion: Some(format!("Try '{}' or '{}_task' instead", 
                                       params.task_id.clone() + "_1", params.task_id)),
            });
        }
        
        if let Some(data) = project_data.as_ref() {
            if let Some(section) = data.sections.get(&params.section) {
                if section.contains_key(&params.task_id) {
                    errors.push(ValidationError {
                        error_type: "duplicate_task_id".to_string(),
                        field: "task_id".to_string(),
                        message: format!("Task ID '{}' already exists in section '{}'", 
                                       params.task_id, params.section),
                        suggestion: Some("Please choose a different task ID".to_string()),
                    });
                    
                    if params.suggest_alternatives.unwrap_or(true) {
                        alternative_ids = self.generate_alternative_ids(&params.task_id, data);
                    }
                }
            }
        }
        
        if let Some(title) = &params.title {
            if title.trim().is_empty() {
                warnings.push(ValidationWarning {
                    warning_type: "empty_title".to_string(),
                    field: "title".to_string(),
                    message: "Task title is empty".to_string(),
                    recommendation: Some("Consider providing a descriptive title".to_string()),
                });
            } else if title.len() > self.config.max_title_length {
                errors.push(ValidationError {
                    error_type: "title_too_long".to_string(),
                    field: "title".to_string(),
                    message: format!("Title exceeds maximum length of {} characters", 
                                   self.config.max_title_length),
                    suggestion: Some("Please shorten the title".to_string()),
                });
            }
        }
        
        if let Some(description) = &params.description {
            if description.len() > self.config.max_description_length {
                errors.push(ValidationError {
                    error_type: "description_too_long".to_string(),
                    field: "description".to_string(),
                    message: format!("Description exceeds maximum length of {} characters", 
                                   self.config.max_description_length),
                    suggestion: Some("Please shorten the description".to_string()),
                });
            }
        }
        
        if self.config.enable_smart_suggestions && errors.is_empty() {
            suggestions = self.generate_smart_suggestions(params);
        }
        
        let is_valid = errors.is_empty();
        
        let duration = start_time.elapsed();
        eprintln!("[DEBUG] Task validation completed in {:?} (valid: {})", duration, is_valid);
        
        Ok(ValidationResult {
            is_valid,
            errors,
            warnings,
            suggestions,
            alternative_ids,
        })
    }

    /// Check for conflicts with existing tasks
    pub fn check_task_conflicts(&self, section: &str, task_id: &str) -> Result<ConflictCheck> {
        let project_data = self.project_data.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock on project data"))?;
        
        let mut conflicts = Vec::new();
        let mut resolutions = Vec::new();
        
        if let Some(data) = project_data.as_ref() {
            for (other_section, section_data) in &data.sections {
                if other_section != section && section_data.contains_key(task_id) {
                    conflicts.push(Conflict {
                        conflict_type: "duplicate_id_cross_section".to_string(),
                        existing_task_section: other_section.clone(),
                        existing_task_id: task_id.to_string(),
                        description: format!("Task ID '{}' already exists in section '{}'", 
                                           task_id, other_section),
                        severity: "medium".to_string(),
                    });
                    
                    resolutions.push(format!("Use a section-specific prefix like '{}_{}'", 
                                            section, task_id));
                }
            }
            
            if let Some(current_section) = data.sections.get(section) {
                for existing_id in current_section.keys() {
                    if self.calculate_similarity(task_id, existing_id) > self.config.similarity_threshold {
                        conflicts.push(Conflict {
                            conflict_type: "similar_id".to_string(),
                            existing_task_section: section.to_string(),
                            existing_task_id: existing_id.clone(),
                            description: format!("Task ID '{}' is very similar to existing ID '{}'", 
                                               task_id, existing_id),
                            severity: "low".to_string(),
                        });
                        
                        resolutions.push(format!("Consider using a more distinctive name"));
                    }
                }
            }
        }
        
        Ok(ConflictCheck {
            has_conflicts: !conflicts.is_empty(),
            conflicts,
            resolutions,
        })
    }

    /// Validate task ID format
    fn validate_task_id_format(&self, task_id: &str) -> Option<ValidationError> {
        if task_id.len() < self.config.min_task_id_length {
            return Some(ValidationError {
                error_type: "task_id_too_short".to_string(),
                field: "task_id".to_string(),
                message: format!("Task ID must be at least {} characters long", 
                               self.config.min_task_id_length),
                suggestion: Some("Please use a longer, more descriptive ID".to_string()),
            });
        }
        
        if task_id.len() > self.config.max_task_id_length {
            return Some(ValidationError {
                error_type: "task_id_too_long".to_string(),
                field: "task_id".to_string(),
                message: format!("Task ID cannot exceed {} characters", 
                               self.config.max_task_id_length),
                suggestion: Some("Please use a shorter ID".to_string()),
            });
        }
        
        if !self.task_id_pattern.is_match(task_id) {
            return Some(ValidationError {
                error_type: "invalid_task_id_format".to_string(),
                field: "task_id".to_string(),
                message: "Task ID can only contain letters, numbers, underscores, and hyphens, and must start with a letter or underscore".to_string(),
                suggestion: Some("Use only alphanumeric characters, underscores, and hyphens".to_string()),
            });
        }
        
        None
    }

    /// Generate alternative task IDs
    fn generate_alternative_ids(&self, base_id: &str, project_data: &ProjectData) -> Vec<String> {
        let mut alternatives = Vec::new();
        let section_data = project_data.sections.values().next();
        
        if let Some(section) = section_data {
            for i in 1..=5 {
                let alternative = format!("{}_{}", base_id, i);
                if !section.contains_key(&alternative) {
                    alternatives.push(alternative);
                }
            }
            
            let suffixes = ["_new", "_v2", "_alt", "_task", "_item"];
            for suffix in &suffixes {
                let alternative = format!("{}{}", base_id, suffix);
                if !section.contains_key(&alternative) && alternatives.len() < 5 {
                    alternatives.push(alternative);
                }
            }
        }
        
        alternatives.truncate(3);
        alternatives
    }

    /// Generate smart suggestions based on context
    fn generate_smart_suggestions(&self, params: &ValidationParams) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if params.task_id.contains("bug") {
            suggestions.push("Consider using status 'blocked' if this is a critical bug".to_string());
        }
        
        if params.task_id.contains("test") || params.task_id.contains("spec") {
            suggestions.push("Consider linking this to the corresponding implementation task".to_string());
        }
        
        if params.task_id.len() < 5 {
            suggestions.push("Consider using a more descriptive task ID for better clarity".to_string());
        }
        
        suggestions
    }

    /// Calculate similarity between two strings (simple implementation)
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f32 {
        if s1 == s2 {
            return 1.0;
        }
        
        let longer = if s1.len() > s2.len() { s1 } else { s2 };
        let _shorter = if s1.len() <= s2.len() { s1 } else { s2 };
        
        if longer.len() == 0 {
            return 1.0;
        }
        
        let edit_distance = self.levenshtein_distance(s1, s2);
        (longer.len() - edit_distance) as f32 / longer.len() as f32
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1,
                        matrix[i][j - 1] + 1
                    ),
                    matrix[i - 1][j - 1] + cost
                );
            }
        }
        
        matrix[len1][len2]
    }

    /// Create set of reserved names that cannot be used as task IDs
    fn create_reserved_names() -> HashSet<String> {
        let mut reserved = HashSet::new();
        
        reserved.insert("class".to_string());
        reserved.insert("function".to_string());
        reserved.insert("var".to_string());
        reserved.insert("const".to_string());
        reserved.insert("let".to_string());
        reserved.insert("if".to_string());
        reserved.insert("else".to_string());
        reserved.insert("for".to_string());
        reserved.insert("while".to_string());
        reserved.insert("return".to_string());
        reserved.insert("true".to_string());
        reserved.insert("false".to_string());
        reserved.insert("null".to_string());
        reserved.insert("undefined".to_string());
        
        reserved.insert("system".to_string());
        reserved.insert("admin".to_string());
        reserved.insert("root".to_string());
        reserved.insert("user".to_string());
        reserved.insert("guest".to_string());
        reserved.insert("public".to_string());
        reserved.insert("private".to_string());
        reserved.insert("protected".to_string());
        
        reserved.insert("new".to_string());
        reserved.insert("create".to_string());
        reserved.insert("delete".to_string());
        reserved.insert("update".to_string());
        reserved.insert("edit".to_string());
        reserved.insert("remove".to_string());
        reserved.insert("add".to_string());
        reserved.insert("get".to_string());
        reserved.insert("set".to_string());
        reserved.insert("config".to_string());
        reserved.insert("settings".to_string());
        reserved.insert("system".to_string());
        reserved.insert("admin".to_string());
        reserved.insert("root".to_string());
        reserved.insert("default".to_string());
        
        reserved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_engine_creation() {
        let engine = ValidationEngine::new(None);
        assert!(!engine.reserved_names.is_empty());
    }

    #[test]
    fn test_task_id_validation() {
        let engine = ValidationEngine::new(None);
        let params = ValidationParams {
            section: "test".to_string(),
            task_id: "valid_task_id".to_string(),
            title: Some("Test task".to_string()),
            description: None,
            check_duplicates: Some(true),
            suggest_alternatives: Some(true),
        };
        
        let result = engine.validate_task_creation(&params).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_reserved_name_validation() {
        let engine = ValidationEngine::new(None);
        let params = ValidationParams {
            section: "test".to_string(),
            task_id: "class".to_string(),
            title: Some("Test task".to_string()),
            description: None,
            check_duplicates: Some(true),
            suggest_alternatives: Some(true),
        };
        
        let result = engine.validate_task_creation(&params).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_similarity_calculation() {
        let engine = ValidationEngine::new(None);
        assert_eq!(engine.calculate_similarity("test", "test"), 1.0);
        assert!(engine.calculate_similarity("test", "tast") > 0.5);
        assert!(engine.calculate_similarity("hello", "world") < 0.5);
    }
}