/*!
 * Statistics Module for Anchora Backend
 *
 * Provides server-side statistics calculation and caching with:
 * - Task completion rates and trends
 * - Section-wise analytics
 * - Performance monitoring
 * - Intelligent caching
 */

use crate::task_manager::{ProjectData, Task, TaskStatus};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::RwLock;

/// Main statistics structure
#[derive(Debug, Serialize, Clone)]
pub struct TaskStatistics {
    pub overview: TaskOverview,
    pub sections: HashMap<String, SectionStats>,
    pub recent_activity: Vec<TaskActivity>,
    pub trends: StatsTrends,
    pub generated_at: DateTime<Utc>,
}

/// High-level task overview
#[derive(Debug, Serialize, Clone)]
pub struct TaskOverview {
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub in_progress_tasks: u32,
    pub blocked_tasks: u32,
    pub completion_rate: f32,
    pub sections: Vec<SectionSummary>,
}

/// Statistics for a specific section
#[derive(Debug, Serialize, Clone)]
pub struct SectionStats {
    pub name: String,
    pub total: u32,
    pub todo: u32,
    pub in_progress: u32,
    pub done: u32,
    pub blocked: u32,
    pub completion_rate: f32,
    pub avg_completion_time_days: Option<f32>,
    pub most_active_files: Vec<String>,
}

/// Trends and historical data
#[derive(Debug, Serialize, Clone)]
pub struct StatsTrends {
    pub completion_trend_7d: f32,
    pub creation_trend_7d: f32,
    pub productivity_score: f32,
    pub busiest_sections: Vec<String>,
}

/// Task update record for tracking changes
#[derive(Debug, Clone, Serialize)]
pub struct TaskUpdate {
    pub section: String,
    pub task_id: String,
    pub old_status: Option<TaskStatus>,
    pub new_status: TaskStatus,
    pub timestamp: DateTime<Utc>,
    pub change_type: ChangeType,
}

/// Type of change made to a task
#[derive(Debug, Clone, Serialize)]
pub enum ChangeType {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "status_updated")]
    StatusUpdated,
    #[serde(rename = "deleted")]
    Deleted,
    #[serde(rename = "modified")]
    Modified,
}

/// Summary of a section for overview
#[derive(Debug, Serialize, Clone)]
pub struct SectionSummary {
    pub name: String,
    pub total_tasks: u32,
    pub completion_percentage: f32,
    pub active_tasks: u32,
    pub blocked_tasks: u32,
    pub recent_changes: u32,
}

/// Activity entry for recent activity feed
#[derive(Debug, Serialize, Clone)]
pub struct TaskActivity {
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub section: String,
    pub task_id: Option<String>,
    pub activity_type: ActivityType,
}

#[derive(Debug, Serialize, Clone)]
pub enum ActivityType {
    #[serde(rename = "task_created")]
    TaskCreated,
    #[serde(rename = "task_completed")]
    TaskCompleted,
    #[serde(rename = "status_changed")]
    StatusChanged,
    #[serde(rename = "section_updated")]
    SectionUpdated,
}

/// Cached statistic entry
#[derive(Debug, Clone)]
struct CachedStatistic {
    data: TaskStatistics,
    created_at: DateTime<Utc>,
    access_count: u32,
    last_accessed: DateTime<Utc>,
}

/// Statistics manager with intelligent caching
pub struct StatisticsManager {
    /// Cached statistics by cache key
    cached_stats: RwLock<HashMap<String, CachedStatistic>>,
    /// Update history for trend analysis
    update_history: RwLock<Vec<TaskUpdate>>,
    /// Performance tracking
    performance_stats: RwLock<StatisticsPerformance>,
    /// Configuration
    config: StatisticsConfig,
}

/// Configuration for statistics management
#[derive(Debug, Clone)]
pub struct StatisticsConfig {
    pub cache_ttl_seconds: u64,
    pub max_cache_entries: usize,
    pub max_history_entries: usize,
    pub trend_analysis_days: u32,
}

impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 300,
            max_cache_entries: 100,
            max_history_entries: 1000,
            trend_analysis_days: 30,
        }
    }
}

/// Performance tracking for statistics operations
#[derive(Debug, Default)]
struct StatisticsPerformance {
    cache_hits: u64,
    cache_misses: u64,
    total_calculations: u64,
    avg_calculation_time_ms: f64,
}

impl StatisticsManager {
    /// Create a new statistics manager
    pub fn new(config: Option<StatisticsConfig>) -> Self {
        Self {
            cached_stats: RwLock::new(HashMap::new()),
            update_history: RwLock::new(Vec::new()),
            performance_stats: RwLock::new(StatisticsPerformance::default()),
            config: config.unwrap_or_default(),
        }
    }

    /// Get task statistics with caching
    pub fn get_statistics(&self, project_data: &ProjectData) -> Result<TaskStatistics> {
        let start_time = std::time::Instant::now();
        let cache_key = self.generate_cache_key(project_data);

        {
            if let Ok(cache) = self.cached_stats.read() {
                if let Some(cached) = cache.get(&cache_key) {
                    let age = Utc::now().signed_duration_since(cached.created_at);
                    if age.num_seconds() < self.config.cache_ttl_seconds as i64 {
                        {
                            if let Ok(mut cache_write) = self.cached_stats.write() {
                                if let Some(entry) = cache_write.get_mut(&cache_key) {
                                    entry.access_count += 1;
                                    entry.last_accessed = Utc::now();
                                }
                            }
                        }

                        self.update_cache_hit_rate(true);
                        eprintln!("[DEBUG] Statistics cache hit for key: {}", cache_key);
                        return Ok(cached.data.clone());
                    }
                }
            }
        }

        self.update_cache_hit_rate(false);
        let stats = self.calculate_statistics(project_data)?;

        if let Ok(mut cache) = self.cached_stats.write() {
            cache.insert(
                cache_key,
                CachedStatistic {
                    data: stats.clone(),
                    created_at: Utc::now(),
                    access_count: 1,
                    last_accessed: Utc::now(),
                },
            );

            if cache.len() > self.config.max_cache_entries {
                self.cleanup_cache(&mut cache);
            }
        }

        let calculation_time = start_time.elapsed();
        eprintln!("[INFO] Task overview calculated in {:?}", calculation_time);

        Ok(stats)
    }

    /// Get task overview (simplified statistics)
    pub fn get_overview(&self, project_data: &ProjectData) -> Result<TaskOverview> {
        let mut total_tasks = 0u32;
        let mut completed_tasks = 0u32;
        let mut in_progress_tasks = 0u32;
        let mut blocked_tasks = 0u32;
        let mut sections = Vec::new();

        for (section_name, section) in &project_data.sections {
            let section_stats = self.calculate_section_stats(section)?;

            total_tasks += section_stats.total;
            completed_tasks += section_stats.done;
            in_progress_tasks += section_stats.in_progress;
            blocked_tasks += section_stats.blocked;

            sections.push(SectionSummary {
                name: section_name.clone(),
                total_tasks: section_stats.total,
                completion_percentage: section_stats.completion_rate,
                active_tasks: section_stats.in_progress,
                blocked_tasks: section_stats.blocked,
                recent_changes: 0,
            });
        }

        let completion_rate = if total_tasks > 0 {
            (completed_tasks as f32 / total_tasks as f32) * 100.0
        } else {
            0.0
        };

        Ok(TaskOverview {
            total_tasks,
            completed_tasks,
            in_progress_tasks,
            blocked_tasks,
            completion_rate,
            sections,
        })
    }

    /// Record a task update for trend analysis
    pub fn record_task_update(&self, update: TaskUpdate) -> Result<()> {
        if let Ok(mut history) = self.update_history.write() {
            history.push(update);

            if history.len() > self.config.max_history_entries {
                history.remove(0);
            }
        }
        Ok(())
    }

    /// Calculate statistics for the given project data
    fn calculate_statistics(&self, project_data: &ProjectData) -> Result<TaskStatistics> {
        let overview = self.get_overview(project_data)?;
        let mut sections = HashMap::new();

        for (section_name, section) in &project_data.sections {
            let section_stats = self.calculate_section_stats(section)?;
            sections.insert(section_name.clone(), section_stats);
        }

        let recent_activity = self.get_recent_activity()?;
        let trends = self.calculate_trends()?;

        Ok(TaskStatistics {
            overview,
            sections,
            recent_activity,
            trends,
            generated_at: Utc::now(),
        })
    }

    /// Calculate statistics for a single section
    fn calculate_section_stats(&self, section: &HashMap<String, Task>) -> Result<SectionStats> {
        let mut total = 0u32;
        let mut todo = 0u32;
        let mut in_progress = 0u32;
        let mut done = 0u32;
        let mut blocked = 0u32;

        for task in section.values() {
            total += 1;
            match task.status {
                TaskStatus::Todo => todo += 1,
                TaskStatus::InProgress => in_progress += 1,
                TaskStatus::Done => done += 1,
                TaskStatus::Blocked => blocked += 1,
            }
        }

        let completion_rate = if total > 0 {
            (done as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        Ok(SectionStats {
            name: "".to_string(), // Will be set by caller
            total,
            todo,
            in_progress,
            done,
            blocked,
            completion_rate,
            avg_completion_time_days: None,
            most_active_files: vec![],
        })
    }

    /// Get recent activity from update history
    pub fn get_recent_activity(&self) -> Result<Vec<TaskActivity>> {
        let mut activities = Vec::new();

        if let Ok(history) = self.update_history.read() {
            for update in history.iter().take(10) {
                let activity_type = match update.change_type {
                    ChangeType::Created => ActivityType::TaskCreated,
                    ChangeType::StatusUpdated if update.new_status == TaskStatus::Done => {
                        ActivityType::TaskCompleted
                    }
                    ChangeType::StatusUpdated => ActivityType::StatusChanged,
                    _ => ActivityType::SectionUpdated,
                };

                activities.push(TaskActivity {
                    description: format!(
                        "Task {}:{} {}",
                        update.section,
                        update.task_id,
                        match update.change_type {
                            ChangeType::Created => "created",
                            ChangeType::StatusUpdated => "status updated",
                            ChangeType::Deleted => "deleted",
                            ChangeType::Modified => "modified",
                        }
                    ),
                    timestamp: update.timestamp,
                    section: update.section.clone(),
                    task_id: Some(update.task_id.clone()),
                    activity_type,
                });
            }
        }

        Ok(activities)
    }

    /// Calculate trends from historical data
    fn calculate_trends(&self) -> Result<StatsTrends> {
        // Simple implementation for now
        Ok(StatsTrends {
            completion_trend_7d: 0.0,
            creation_trend_7d: 0.0,
            productivity_score: 75.0,
            busiest_sections: vec![],
        })
    }

    /// Generate cache key for project data
    fn generate_cache_key(&self, project_data: &ProjectData) -> String {
        format!(
            "stats_{}_{}",
            project_data.sections.len(),
            project_data.meta.last_updated.timestamp()
        )
    }

    /// Update cache hit rate statistics
    fn update_cache_hit_rate(&self, hit: bool) {
        if let Ok(mut stats) = self.performance_stats.write() {
            if hit {
                stats.cache_hits += 1;
            } else {
                stats.cache_misses += 1;
            }
        }
    }

    /// Clean up old cache entries
    fn cleanup_cache(&self, cache: &mut HashMap<String, CachedStatistic>) {
        let entries: Vec<_> = cache
            .iter()
            .map(|(k, v)| (k.clone(), v.last_accessed))
            .collect();
        let mut sorted_entries = entries;
        sorted_entries.sort_by_key(|(_, last_accessed)| *last_accessed);

        let remove_count = cache.len() - self.config.max_cache_entries + 1;
        for i in 0..remove_count.min(sorted_entries.len()) {
            cache.remove(&sorted_entries[i].0);
        }
    }

    /// Get performance metrics for monitoring
    pub fn get_performance_metrics(&self) -> Result<serde_json::Value> {
        let stats = self
            .performance_stats
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to read performance stats"))?;
        let cache = self
            .cached_stats
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to read cache"))?;

        let total_requests = stats.cache_hits + stats.cache_misses;
        let cache_hit_rate = if total_requests > 0 {
            stats.cache_hits as f64 / total_requests as f64 * 100.0
        } else {
            0.0
        };

        Ok(serde_json::json!({
            "cache_statistics": {
                "total_cache_entries": cache.len(),
                "cache_hits": stats.cache_hits,
                "cache_misses": stats.cache_misses,
                "cache_hit_rate": cache_hit_rate
            },
            "performance": {
                "total_calculations": stats.total_calculations,
                "avg_calculation_time_ms": stats.avg_calculation_time_ms
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_manager::Task;
    use std::collections::HashMap;

    #[test]
    fn test_statistics_manager_creation() {
        let manager = StatisticsManager::new(None);
        let metrics = manager.get_performance_metrics().unwrap();
        assert!(
            metrics["cache_statistics"]["total_cache_entries"]
                .as_u64()
                .unwrap()
                == 0
        );
    }

    #[test]
    fn test_section_stats_calculation() {
        let manager = StatisticsManager::new(None);
        let mut section_data = HashMap::new();

        let mut task = Task::new("Test task".to_string(), None);
        task.status = TaskStatus::Done;
        section_data.insert("task1".to_string(), task);

        let stats = manager.calculate_section_stats(&section_data).unwrap();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.completion_rate, 100.0);
    }

    #[test]
    fn test_task_update_recording() {
        let manager = StatisticsManager::new(None);

        let update = TaskUpdate {
            section: "test".to_string(),
            task_id: "task1".to_string(),
            old_status: Some(TaskStatus::Todo),
            new_status: TaskStatus::Done,
            timestamp: Utc::now(),
            change_type: ChangeType::StatusUpdated,
        };

        assert!(manager.record_task_update(update).is_ok());
    }
}
