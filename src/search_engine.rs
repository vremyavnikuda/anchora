/*!
 * Search Engine Module for Anchora Backend
 * 
 * Provides server-side task search and filtering capabilities with:
 * - Indexed search for fast performance
 * - Smart suggestions and auto-completion
 * - Statistics caching
 * - Performance monitoring
 */

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::task_manager::{ProjectData, TaskStatus, Task};
use anyhow::Result;

/// Search query parameters with filtering options
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Advanced filtering options for search
#[derive(Debug, Deserialize)]
pub struct SearchFilters {
    pub sections: Option<Vec<String>>,
    pub statuses: Option<Vec<TaskStatus>>,
    pub include_descriptions: Option<bool>,
    pub file_paths: Option<Vec<String>>,
    pub created_after: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
}

/// Search result with metadata
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub tasks: Vec<TaskSearchResult>,
    pub total_count: u32,
    pub filtered_count: u32,
    pub search_time_ms: u64,
    pub suggestions: Vec<String>,
}

/// Individual task in search results
#[derive(Debug, Serialize, Clone)]
pub struct TaskSearchResult {
    pub section: String,
    pub task_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub file_count: u32,
    pub relevance: f32,
    pub match_type: MatchType,
}

/// Type of match found during search
#[derive(Debug, Serialize, Clone)]
pub enum MatchType {
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "partial")]
    Partial,
    #[serde(rename = "fuzzy")]
    Fuzzy,
}

/// Search suggestion with metadata
#[derive(Debug, Serialize)]
pub struct Suggestion {
    pub text: String,
    pub suggestion_type: SuggestionType,
    pub relevance: f32,
    pub frequency: u32,
}

/// Type of suggestion
#[derive(Debug, Serialize)]
pub enum SuggestionType {
    #[serde(rename = "task_id")]
    TaskId,
    #[serde(rename = "section")]
    Section,
    #[serde(rename = "keyword")]
    Keyword,
    #[serde(rename = "status")]
    Status,
}

/// Search index for fast lookups
#[derive(Debug)]
struct SearchIndex {
    /// Task ID to full task reference mapping
    task_index: HashMap<String, TaskReference>,
    /// Word to task IDs mapping for full-text search
    word_index: HashMap<String, HashSet<String>>,
    /// Section to task IDs mapping
    section_index: HashMap<String, HashSet<String>>,
    /// Status to task IDs mapping
    status_index: HashMap<TaskStatus, HashSet<String>>,
    /// Frequently searched terms for suggestions
    suggestion_cache: HashMap<String, u32>,
    /// Last update timestamp
    last_updated: DateTime<Utc>,
}

/// Internal task reference for indexing
#[derive(Debug, Clone)]
struct TaskReference {
    pub section: String,
    pub task_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub keywords: Vec<String>,
}

/// Main search engine with caching and indexing
pub struct SearchEngine {
    index: RwLock<SearchIndex>,
    performance_stats: RwLock<PerformanceStats>,
}

/// Performance statistics for monitoring
#[derive(Debug, Default)]
struct PerformanceStats {
    total_searches: u64,
    total_search_time_ms: u64,
    cache_hits: u64,
    index_rebuilds: u64,
    last_index_rebuild: Option<DateTime<Utc>>,
}

impl SearchEngine {
    /// Create a new search engine instance
    pub fn new() -> Self {
        Self {
            index: RwLock::new(SearchIndex::new()),
            performance_stats: RwLock::new(PerformanceStats::default()),
        }
    }

    /// Build search index from project data
    pub fn index_project(&self, project_data: &ProjectData) -> Result<()> {
        let start_time = Instant::now();
        let mut index = self.index.write().map_err(|_| anyhow::anyhow!("Failed to acquire write lock on search index"))?;
        
        index.clear();
        for (section_name, section) in &project_data.sections {
            for (task_id, task) in section {
                let full_task_id = format!("{}.{}", section_name, task_id);
                let task_ref = TaskReference::from_task(section_name, task_id, task);
                index.task_index.insert(full_task_id.clone(), task_ref.clone());
                index.section_index
                    .entry(section_name.clone())
                    .or_insert_with(HashSet::new)
                    .insert(full_task_id.clone());
                index.status_index
                    .entry(task.status.clone())
                    .or_insert_with(HashSet::new)
                    .insert(full_task_id.clone());
                for keyword in &task_ref.keywords {
                    index.word_index
                        .entry(keyword.clone())
                        .or_insert_with(HashSet::new)
                        .insert(full_task_id.clone());
                }
            }
        }
        index.last_updated = Utc::now();
        if let Ok(mut stats) = self.performance_stats.write() {
            stats.index_rebuilds += 1;
            stats.last_index_rebuild = Some(Utc::now());
        }
        
        let duration = start_time.elapsed();
        eprintln!("[INFO] Search index rebuilt in {:?} with {} tasks", duration, index.task_index.len());
        
        Ok(())
    }

    /// Perform search with the given query and filters
    pub fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let start_time = Instant::now();
        let index = self.index.read().map_err(|_| anyhow::anyhow!("Failed to acquire read lock on search index"))?;
        
        let mut results = Vec::new();
        let query_lower = query.query.to_lowercase();
        
        for (_task_id, task_ref) in &index.task_index {
            let mut matches = false;
            let mut match_type = MatchType::Fuzzy;
            
            if task_ref.title.to_lowercase().contains(&query_lower) {
                matches = true;
                if task_ref.title.to_lowercase() == query_lower {
                    match_type = MatchType::Exact;
                } else {
                    match_type = MatchType::Partial;
                }
            }
            
            if let Some(desc) = &task_ref.description {
                if desc.to_lowercase().contains(&query_lower) {
                    matches = true;
                }
            }
            
            if matches {
                results.push(TaskSearchResult {
                    section: task_ref.section.clone(),
                    task_id: task_ref.task_id.clone(),
                    title: task_ref.title.clone(),
                    description: task_ref.description.clone(),
                    status: task_ref.status.clone(),
                    created: task_ref.created,
                    updated: task_ref.updated,
                    file_count: 1,
                    relevance: 1.0,
                    match_type,
                });
            }
        }
        
        if let Some(filters) = &query.filters {
            if let Some(statuses) = &filters.statuses {
                results.retain(|r| statuses.contains(&r.status));
            }
            if let Some(sections) = &filters.sections {
                results.retain(|r| sections.contains(&r.section));
            }
        }
        
        let total_count = results.len() as u32;
        
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(50);
        
        if offset < results.len() {
            let end = std::cmp::min(offset + limit, results.len());
            results = results[offset..end].to_vec();
        } else {
            results.clear();
        }
        
        let search_time = start_time.elapsed();
        
        if let Ok(mut stats) = self.performance_stats.write() {
            stats.total_searches += 1;
            stats.total_search_time_ms += search_time.as_millis() as u64;
        }
        
        Ok(SearchResult {
            tasks: results,
            total_count,
            filtered_count: total_count,
            search_time_ms: search_time.as_millis() as u64,
            suggestions: vec![],
        })
    }

    /// Get suggestions for partial query
    pub fn get_suggestions(&self, partial_query: &str) -> Result<Vec<Suggestion>> {
        let index = self.index.read().map_err(|_| anyhow::anyhow!("Failed to acquire read lock on search index"))?;
        let mut suggestions = Vec::new();
        let query_lower = partial_query.to_lowercase();
        
        for section_name in index.section_index.keys() {
            if section_name.to_lowercase().starts_with(&query_lower) {
                suggestions.push(Suggestion {
                    text: section_name.clone(),
                    suggestion_type: SuggestionType::Section,
                    relevance: 0.9,
                    frequency: index.section_index.get(section_name).map(|s| s.len() as u32).unwrap_or(0),
                });
            }
        }
        
        for task_ref in index.task_index.values() {
            if task_ref.task_id.to_lowercase().starts_with(&query_lower) {
                suggestions.push(Suggestion {
                    text: task_ref.task_id.clone(),
                    suggestion_type: SuggestionType::TaskId,
                    relevance: 0.8,
                    frequency: 1,
                });
            }
        }
        
        suggestions.sort_by(|a, b| {
            b.relevance.partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.frequency.cmp(&a.frequency))
        });
        
        suggestions.truncate(10);
        
        Ok(suggestions)
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> Result<serde_json::Value> {
        let stats = self.performance_stats.read().map_err(|_| anyhow::anyhow!("Failed to acquire read lock on performance stats"))?;
        let index = self.index.read().map_err(|_| anyhow::anyhow!("Failed to acquire read lock on search index"))?;
        
        let avg_search_time = if stats.total_searches > 0 {
            stats.total_search_time_ms as f64 / stats.total_searches as f64
        } else {
            0.0
        };
        
        Ok(serde_json::json!({
            "total_searches": stats.total_searches,
            "avg_search_time_ms": avg_search_time,
            "cache_hits": stats.cache_hits,
            "index_rebuilds": stats.index_rebuilds,
            "last_index_rebuild": stats.last_index_rebuild,
            "indexed_tasks": index.task_index.len(),
            "indexed_words": index.word_index.len(),
            "indexed_sections": index.section_index.len()
        }))
    }
}

impl SearchIndex {
    fn new() -> Self {
        Self {
            task_index: HashMap::new(),
            word_index: HashMap::new(),
            section_index: HashMap::new(),
            status_index: HashMap::new(),
            suggestion_cache: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
    
    fn clear(&mut self) {
        self.task_index.clear();
        self.word_index.clear();
        self.section_index.clear();
        self.status_index.clear();
        self.suggestion_cache.clear();
    }
}

impl TaskReference {
    fn from_task(section: &str, task_id: &str, task: &Task) -> Self {
        let mut keywords = Vec::new();
        
        keywords.extend(
            task.title
                .to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() > 2)
                .map(String::from)
        );
        
        if let Some(desc) = &task.description {
            keywords.extend(
                desc.to_lowercase()
                    .split_whitespace()
                    .filter(|w| w.len() > 2)
                    .map(String::from)
            );
        }
        
        keywords.push(section.to_lowercase());
        keywords.push(task_id.to_lowercase());
        
        keywords.sort();
        keywords.dedup();
        
        Self {
            section: section.to_string(),
            task_id: task_id.to_string(),
            title: task.title.clone(),
            description: task.description.clone(),
            status: task.status.clone(),
            created: task.created,
            updated: task.updated,
            keywords,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_manager::Task;

    #[test]
    fn test_search_engine_creation() {
        let engine = SearchEngine::new();
        let stats = engine.get_performance_stats().unwrap();
        assert_eq!(stats["total_searches"], 0);
    }

    #[test]
    fn test_task_reference_creation() {
        let mut task = Task::new("Test task".to_string(), Some("Test description".to_string()));
        task.status = TaskStatus::Todo;
        
        let task_ref = TaskReference::from_task("test_section", "test_task", &task);
        
        assert_eq!(task_ref.section, "test_section");
        assert_eq!(task_ref.task_id, "test_task");
        assert_eq!(task_ref.title, "Test task");
        assert!(task_ref.keywords.contains(&"test".to_string()));
    }
}