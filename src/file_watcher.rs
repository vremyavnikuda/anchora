use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
}
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    pub file_patterns: Vec<String>,
    pub ignored_dirs: Vec<String>,
    pub max_file_size: u64,
    pub debounce_timeout: u64,
}
impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            file_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.ts".to_string(),
                "**/*.js".to_string(),
                "**/*.py".to_string(),
                "**/*.java".to_string(),
                "**/*.cpp".to_string(),
                "**/*.c".to_string(),
                "**/*.h".to_string(),
                "**/*.hpp".to_string(),
                "**/*.cc".to_string(),
                "**/*.cxx".to_string(),
                "**/*.go".to_string(),
                "**/*.php".to_string(),
                "**/*.rb".to_string(),
                "**/*.swift".to_string(),
                "**/*.kt".to_string(),
                "**/*.scala".to_string(),
                "**/*.cs".to_string(),
                "**/*.fs".to_string(),
                "**/*.vb".to_string(),
                "**/*.dart".to_string(),
                "**/*.elm".to_string(),
                "**/*.hs".to_string(),
                "**/*.ml".to_string(),
                "**/*.clj".to_string(),
                "**/*.ex".to_string(),
                "**/*.exs".to_string(),
                "**/*.erl".to_string(),
                "**/*.jl".to_string(),
                "**/*.r".to_string(),
                "**/*.m".to_string(),
                "**/*.mm".to_string(),
                "**/*.pl".to_string(),
                "**/*.pm".to_string(),
                "**/*.lua".to_string(),
                "**/*.sh".to_string(),
                "**/*.ps1".to_string(),
                "**/*.bat".to_string(),
                "**/*.cmd".to_string(),
                "**/*.jsx".to_string(),
                "**/*.tsx".to_string(),
                "**/*.vue".to_string(),
                "**/*.svelte".to_string(),
                "**/*.sql".to_string(),
                "**/*.yaml".to_string(),
                "**/*.yml".to_string(),
                "**/*.toml".to_string(),
                "**/*.ini".to_string(),
                "**/*.cfg".to_string(),
                "**/*.conf".to_string(),
                "**/*.dockerfile".to_string(),
                "**/*.tf".to_string(),
                "**/*.hcl".to_string(),
                "**/*.json".to_string(),
                "**/*.xml".to_string(),
                "**/*.html".to_string(),
                "**/*.css".to_string(),
                "**/*.scss".to_string(),
                "**/*.sass".to_string(),
                "**/*.less".to_string(),
                "**/*.md".to_string(),
                "**/*.rst".to_string(),
                "**/*.tex".to_string(),
            ],
            ignored_dirs: vec![
                "target".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                ".vscode".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "__pycache__".to_string(),
            ],
            max_file_size: 10 * 1024 * 1024,
            debounce_timeout: 500,
        }
    }
}
pub struct FileWatcher {
    config: WatcherConfig,
    _event_tx: mpsc::UnboundedSender<FileEvent>,
    _watcher: RecommendedWatcher,
}
impl FileWatcher {
    pub fn new(
        workspace_path: &Path,
        config: WatcherConfig,
    ) -> anyhow::Result<(Self, mpsc::UnboundedReceiver<FileEvent>)> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let tx_clone = event_tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if let Some(file_event) = Self::process_notify_event(event) {
                        let _ = tx_clone.send(file_event);
                    }
                }
            },
            Config::default(),
        )?;
        watcher.watch(workspace_path, RecursiveMode::Recursive)?;
        println!("Started watching directory: {:?}", workspace_path);
        let file_watcher = Self {
            config,
            _event_tx: event_tx,
            _watcher: watcher,
        };
        Ok((file_watcher, event_rx))
    }
    fn process_notify_event(event: Event) -> Option<FileEvent> {
        use notify::EventKind;
        match event.kind {
            EventKind::Create(_) => {
                if let Some(path) = event.paths.first() {
                    Some(FileEvent::Created(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Modify(_) => {
                if let Some(path) = event.paths.first() {
                    Some(FileEvent::Modified(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Remove(_) => {
                if let Some(path) = event.paths.first() {
                    Some(FileEvent::Deleted(path.clone()))
                } else {
                    None
                }
            }
            EventKind::Other => {
                if event.paths.len() == 2 {
                    Some(FileEvent::Renamed {
                        from: event.paths[0].clone(),
                        to: event.paths[1].clone(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn should_process_file(&self, file_path: &Path) -> bool {
        if let Ok(metadata) = std::fs::metadata(file_path) {
            if metadata.len() > self.config.max_file_size {
                return false;
            }
        }
        for ignored_dir in &self.config.ignored_dirs {
            if file_path.components().any(|component| {
                component.as_os_str().to_str().unwrap_or("") == ignored_dir
            }) {
                return false;
            }
        }
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            for pattern in &self.config.file_patterns {
                if Self::matches_pattern(file_name, pattern) {
                    return true;
                }
            }
        }
        false
    }
    fn matches_pattern(file_name: &str, pattern: &str) -> bool {
        if pattern == "**/*" {
            return true;
        }
        if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            if suffix.starts_with("*.") {
                let extension = &suffix[2..];
                return file_name.ends_with(&format!(".{}", extension));
            }
        }
        if pattern.starts_with("*.") {
            let extension = &pattern[2..];
            return file_name.ends_with(&format!(".{}", extension));
        }
        file_name == pattern
    }
    pub fn get_stats(&self) -> WatcherStats {
        WatcherStats {
            file_patterns_count: self.config.file_patterns.len(),
            ignored_dirs_count: self.config.ignored_dirs.len(),
            max_file_size: self.config.max_file_size,
            debounce_timeout: self.config.debounce_timeout,
        }
    }
}
#[derive(Debug, Clone)]
pub struct WatcherStats {
    pub file_patterns_count: usize,
    pub ignored_dirs_count: usize,
    pub max_file_size: u64,
    pub debounce_timeout: u64,
}
pub struct EventDebouncer {
    timeout: Duration,
    pending_events: std::collections::HashMap<PathBuf, FileEvent>,
}
impl EventDebouncer {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            timeout: Duration::from_millis(timeout_ms),
            pending_events: std::collections::HashMap::new(),
        }
    }
    pub async fn add_event(&mut self, event: FileEvent) -> Option<Vec<FileEvent>> {
        let path = match &event {
            FileEvent::Created(p) | FileEvent::Modified(p) | FileEvent::Deleted(p) => p.clone(),
            FileEvent::Renamed { to, .. } => to.clone(),
        };
        self.pending_events.insert(path, event);
        tokio::time::sleep(self.timeout).await;
        if !self.pending_events.is_empty() {
            let events: Vec<FileEvent> = self.pending_events.drain().map(|(_, event)| event).collect();
            Some(events)
        } else {
            None
        }
    }
    pub fn flush(&mut self) -> Vec<FileEvent> {
        self.pending_events.drain().map(|(_, event)| event).collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use tokio::time::timeout;
    #[test]
    fn test_watcher_config_default() {
        let config = WatcherConfig::default();
        assert!(!config.file_patterns.is_empty());
        assert!(config.file_patterns.contains(&"**/*.rs".to_string()));
        assert!(config.ignored_dirs.contains(&"target".to_string()));
    }
    #[test]
    fn test_should_process_file() {
        let config = WatcherConfig::default();
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();
        let (tx, _rx) = mpsc::unbounded_channel();
        let dummy_watcher = RecommendedWatcher::new(|_| {}, Config::default()).unwrap();     
        let file_watcher = FileWatcher {
            config,
            _event_tx: tx,
            _watcher: dummy_watcher,
        };
        assert!(file_watcher.should_process_file(&test_file));
        let target_file = temp_dir.path().join("target").join("test.rs");
        assert!(!file_watcher.should_process_file(&target_file));
    }
    #[test]
    fn test_matches_pattern() {
        assert!(FileWatcher::matches_pattern("test.rs", "*.rs"));
        assert!(FileWatcher::matches_pattern("test.rs", "**/*.rs"));
        assert!(!FileWatcher::matches_pattern("test.py", "*.rs"));
        assert!(FileWatcher::matches_pattern("anything", "**/*"));
    }
    #[tokio::test]
    async fn test_event_debouncer() {
        let mut debouncer = EventDebouncer::new(100);
        let event = FileEvent::Modified(PathBuf::from("test.rs"));
        let result = timeout(Duration::from_millis(200), debouncer.add_event(event)).await;
        assert!(result.is_ok());
    }
}