use std::collections::hash_map::DefaultHasher;
use std::hash::{
        Hash,
        Hasher,
};
use std::path::PathBuf;
use std::str::FromStr;
use std::{
        fs,
        io,
};

use crate::commands::Task;
use crate::datetime::DateTime;

pub struct Store {
        pub root: PathBuf,
}

impl Store {
        pub fn new() -> io::Result<Self> {
                let mut current_dir = std::env::current_dir()?;
                loop {
                        let tasks_dir = current_dir.join("tasks");
                        if tasks_dir.exists() {
                                return Ok(Self { root: tasks_dir });
                        }
                        if !current_dir.pop() {
                                break;
                        }
                }

                Ok(Self {
                        root: std::env::current_dir()?.join("tasks"),
                })
        }

        /// Ensures the root directory and configuration exist.
        /// Returns the configured timezone offset in seconds.
        fn init(&self) -> io::Result<i32> {
                fs::create_dir_all(&self.root)?;

                let config_path = self.root.join(".config");
                if config_path.exists() {
                        let content = fs::read_to_string(&config_path)?;

                        for line in content.lines() {
                                let trimmed = line.trim();
                                if trimmed.is_empty() || trimmed.starts_with('#') {
                                        continue;
                                }
                                return parse_offset(trimmed).ok_or_else(|| {
                                        io::Error::new(
                                                io::ErrorKind::InvalidData,
                                                format!("Invalid timezone in config: {}", trimmed),
                                        )
                                });
                        }

                        Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Config file found but contains no timezone",
                        ))
                } else {
                        let utc_offset = 0;
                        let timezone = "+00:00";

                        let content = [
                                "# Configuration for tafsk store",
                                "# Timezone offset (e.g. +09:30, -05:00)",
                                timezone,
                        ];

                        fs::write(&config_path, content.join("\n"))?;
                        Ok(utc_offset)
                }
        }

        /// Updates the timezone configuration.
        pub fn update_config(&self, timezone: &str) -> io::Result<()> {
                if parse_offset(timezone).is_none() {
                        return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "Invalid timezone format. Expected +HH:MM or -HH:MM",
                        ));
                }

                if !self.root.exists() {
                        fs::create_dir_all(&self.root)?;
                }

                let config_path = self.root.join(".config");
                let content = [
                        "# Configuration for tafsk store",
                        "# Timezone offset (e.g. +09:30, -05:00)",
                        timezone,
                ];

                fs::write(&config_path, content.join("\n"))?;
                Ok(())
        }

        /// Saves a new task to disk.
        /// Returns the folder name of the newly created task.
        pub fn create_task(&self, content: &str) -> io::Result<String> {
                let offset = self.init()?;

                let mut hasher = DefaultHasher::new();
                content.hash(&mut hasher);
                let hash_val = hasher.finish();
                let hash_str = format!("{:x}", hash_val);
                let short_hash = &hash_str[..8];

                let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                // .as_secs();

                let timestamp = DateTime::new(now.as_secs(), offset).format();
                let folder_name = format!("{}.{}", timestamp, short_hash);

                let task_dir = self.root.join(&folder_name);
                let task_file = task_dir.join("TASK.md");

                fs::create_dir_all(&task_dir)?;
                fs::write(&task_file, content)?;

                Ok(folder_name)
        }

        /// Lists all tasks in the store, sorted by folder name.
        pub fn list_tasks(&self) -> io::Result<Vec<(String, Task)>> {
                let mut tasks = Vec::new();

                // If root doesn't exist, just return empty list
                if !self.root.exists() {
                        return Ok(tasks);
                }

                let entries = fs::read_dir(&self.root)?;

                for entry in entries {
                        let entry = entry?;
                        let path = entry.path();

                        if path.is_dir()
                                && let Some(folder_name) = path.file_name().and_then(|s| s.to_str())
                                && !folder_name.starts_with('.')
                        {
                                let task_file = path.join("TASK.md");
                                if task_file.exists() {
                                        let content = fs::read_to_string(&task_file)?;
                                        match Task::from_str(&content) {
                                                Ok(task) => {
                                                        tasks.push((folder_name.to_string(), task));
                                                },
                                                Err(e) => {
                                                        eprintln!(
                                                                "Warning: Failed to parse task in {}: {}",
                                                                folder_name, e
                                                        );
                                                },
                                        }
                                }
                        }
                }

                tasks.sort_by(|a, b| a.0.cmp(&b.0));
                Ok(tasks)
        }
}

fn parse_offset(s: &str) -> Option<i32> {
        const TZ: &str = "+HH:MM";
        if s.len() != TZ.len() {
                return None;
        }

        const COLON_SEP: char = ':';
        if s.chars().nth(3)? != COLON_SEP {
                return None;
        }

        let sign = s.chars().next()?;
        #[rustfmt::skip]
        let hours: i32   = s[1..3].parse().ok()?;
        let minutes: i32 = s[4..6].parse().ok()?;

        const SECS_IN_HOUR: i32 = 3600;
        const SECS_IN_MINUTE: i32 = 60;

        let offset_in_secs = (hours * SECS_IN_HOUR) + (minutes * SECS_IN_MINUTE);
        #[rustfmt::skip]
        let final_offset = if sign == '-' { -offset_in_secs } else { offset_in_secs };

        Some(final_offset)
}
