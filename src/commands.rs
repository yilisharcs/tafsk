use std::env;
use std::fmt::Display;
use std::process::Command;
use std::str::FromStr;

use lexopt::prelude::*;

use crate::store::Store;

#[derive(Debug, PartialEq)]
pub enum Status {
        Open,
        Closed,
}

impl Display for Status {
        #[rustfmt::skip]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                        Status::Open   => write!(f, "OPEN"),
                        Status::Closed => write!(f, "CLOSED"),
                }
        }
}

impl FromStr for Status {
        type Err = lexopt::Error;

        #[rustfmt::skip]
        fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                        "OPEN"   => Ok(Status::Open),
                        "CLOSED" => Ok(Status::Closed),
                        &_       => unreachable!(),
                }
        }
}

#[derive(Debug)]
pub struct Task {
        pub tags:     Vec<String>,
        pub title:    String,
        pub status:   Status,
        pub priority: u8,
}

#[derive(Default)]
pub struct ListArgs {
        pub show_closed: bool,
}

pub fn handle_list_arg(args: &mut ListArgs, arg: lexopt::Arg) -> Result<(), lexopt::Error> {
        match arg {
                Short('c') | Long("closed") => args.show_closed = true,
                _ => return Err(arg.unexpected()),
        }
        Ok(())
}

impl Task {
        pub fn add(parser: &mut lexopt::Parser) -> Result<(), lexopt::Error> {
                let mut priority: u8 = 10;
                let mut edit_mode = false;
                let mut payload: Vec<String> = Vec::new();

                while let Some(arg) = parser.next()? {
                        match arg {
                                #[rustfmt::skip]
                                Short('h') | Long("help") => {
                                        println!("tafsk-add");
                                        println!();
                                        println!("USAGE:");
                                        println!("    tafsk add [FLAGS] <TITLE>");
                                        println!();
                                        println!("FLAGS:");
                                        println!("    -h, --help           Prints help information");
                                        println!("    -p, --priority <N>   Set priority (0-255, default: 10)");
                                        println!("    -e, --edit           Open editor after creating");
                                        return Ok(());
                                },
                                Short('p') | Long("priority") => {
                                        priority = parser.value()?.parse()?
                                },
                                Short('e') | Long("edit") => edit_mode = true,
                                Value(val) => payload.push(val.string()?),
                                _ => return Err(arg.unexpected()),
                        }
                }

                let mut title = Vec::new();
                let mut tags = Vec::new();

                for item in payload {
                        if let Some(tag) = item.strip_prefix('+') {
                                if !tag.is_empty() {
                                        tags.push(tag.to_string());
                                }
                        } else {
                                title.push(item);
                        }
                }

                if title.is_empty() {
                        return Err(lexopt::Error::Custom(
                                "Missing required argument: Task title".to_string().into(),
                        ));
                }

                let task = Task {
                        title: title.join(" "),
                        tags,
                        status: Status::Open,
                        priority,
                };

                let store = Store::new().map_err(|e| lexopt::Error::Custom(e.into()))?;
                let content = task.render();
                let id = store
                        .create_task(&content)
                        .map_err(|e| lexopt::Error::Custom(e.into()))?;

                println!("Created task {}", id);

                if edit_mode {
                        let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
                        let path = store.root.join(&id).join("TASK.md");

                        let status = Command::new(&editor)
                                .arg(&path)
                                .status()
                                .map_err(|e| lexopt::Error::Custom(e.into()))?;

                        if !status.success() {
                                return Err(lexopt::Error::Custom(
                                        format!("Editor exited with error: {}", status).into(),
                                ));
                        }
                }

                Ok(())
        }

        pub fn done(parser: &mut lexopt::Parser) -> Result<(), lexopt::Error> {
                let mut ids = Vec::new();
                while let Some(arg) = parser.next()? {
                        match arg {
                                Value(val) => {
                                        let id: usize = val.parse()?;
                                        ids.push(id);
                                },
                                #[rustfmt::skip]
                                Short('h') | Long("help") => {
                                        println!("tafsk-done");
                                        println!();
                                        println!("USAGE:");
                                        println!("    tafsk done [ID]...");
                                        println!();
                                        println!("FLAGS:");
                                        println!("    -h, --help       Prints help information");
                                        println!();
                                        println!("ARGS:");
                                        println!("    <ID>...          One or more task IDs to mark as closed");
                                        return Ok(());
                                },
                                _ => return Err(arg.unexpected()),
                        }
                }

                if ids.is_empty() {
                        println!("Usage: done <ID>...");
                        return Ok(());
                }

                let store = Store::new().map_err(|e| lexopt::Error::Custom(e.into()))?;
                let tasks = store
                        .list_tasks()
                        .map_err(|e| lexopt::Error::Custom(e.into()))?;

                for id in ids {
                        if id == 0 || id > tasks.len() {
                                return Err(lexopt::Error::Custom(
                                        "No matches.".to_string().into(),
                                ));
                        }

                        let (folder_name, task) = &tasks[id - 1];

                        if task.status == Status::Closed {
                                return Err(lexopt::Error::Custom(
                                        format!(
                                                "Task [{}] '{}' is already CLOSED.",
                                                id, task.title
                                        )
                                        .into(),
                                ));
                        }

                        let path = store.root.join(folder_name).join("TASK.md");
                        let content = std::fs::read_to_string(&path)
                                .map_err(|e| lexopt::Error::Custom(e.into()))?;

                        let new_lines: Vec<String> = content
                                .lines()
                                .map(|line| {
                                        if line == "status: OPEN" {
                                                "status: CLOSED".to_string()
                                        } else {
                                                line.to_string()
                                        }
                                })
                                .collect();

                        // Join with newlines and append a final newline to match typical file behavior
                        let new_content = new_lines.join("\n") + "\n";

                        std::fs::write(&path, new_content)
                                .map_err(|e| lexopt::Error::Custom(e.into()))?;

                        println!("Completed task [{}] '{}'.", id, task.title);
                }
                Ok(())
        }

        pub fn init(parser: &mut lexopt::Parser) -> Result<(), lexopt::Error> {
                let mut timezone = None;

                loop {
                        // Negative offsets shouldn't be recognized as flags
                        if let Some(mut raw) = parser.try_raw_args() {
                                if let Some(arg) = raw.peek() {
                                        let s = arg.to_string_lossy();
                                        if s.starts_with('-')
                                                && s.chars()
                                                        .nth(1)
                                                        .map_or(false, |c| c.is_ascii_digit())
                                        {
                                                let val = raw.next().unwrap();
                                                timezone = Some(val.to_string_lossy().into_owned());
                                                continue;
                                        }
                                }
                        }

                        let match_arg = match parser.next()? {
                                Some(arg) => arg,
                                None => break,
                        };

                        match match_arg {
                                Value(val) => timezone = Some(val.string()?),
                                #[rustfmt::skip]
                                Short('h') | Long("help") => {
                                        println!("tafsk-init");
                                        println!();
                                        println!("USAGE:");
                                        println!("    tafsk init [TIMEZONE]");
                                        println!();
                                        println!("ARGS:");
                                        println!("    <TIMEZONE>       Timezone offset (e.g., +09:30, -05:00)");
                                        println!("                     Defaults to +00:00 (UTC)");
                                        println!();
                                        println!("FLAGS:");
                                        println!("    -h, --help       Prints help information");
                                        return Ok(());
                                },
                                _ => return Err(match_arg.unexpected()),
                        }
                }

                let timezone = timezone.unwrap_or_else(|| "+00:00".to_string());

                let store = Store::new().map_err(|e| lexopt::Error::Custom(e.into()))?;
                store.update_config(&timezone)
                        .map_err(|e| lexopt::Error::Custom(e.into()))?;

                println!(
                        "Initialized task store at {} with timezone {}",
                        store.root.display(),
                        timezone
                );
                Ok(())
        }

        pub fn list(mut args: ListArgs, parser: &mut lexopt::Parser) -> Result<(), lexopt::Error> {
                while let Some(arg) = parser.next()? {
                        match arg {
                                Short('h') | Long("help") => {
                                        println!("tafsk-list");
                                        println!();
                                        println!("USAGE:");
                                        println!("    tafsk list [FLAGS]");
                                        println!();
                                        println!("FLAGS:");
                                        println!("    -h, --help       Prints help information");
                                        println!("    -c, --closed     Show closed tasks");
                                        return Ok(());
                                },
                                _ => handle_list_arg(&mut args, arg)?,
                        }
                }

                let store = Store::new().map_err(|e| lexopt::Error::Custom(e.into()))?;
                let tasks = store
                        .list_tasks()
                        .map_err(|e| lexopt::Error::Custom(e.into()))?;

                if tasks.is_empty() {
                        println!("No tasks found.");
                        return Ok(());
                }

                let mut indexed_tasks: Vec<_> = tasks
                        .into_iter()
                        .enumerate()
                        // indexes are lua-pilled
                        .map(|(i, (folder, task))| (i + 1, folder, task))
                        .collect();
                indexed_tasks.sort_by(|a, b| b.2.priority.cmp(&a.2.priority));

                for (display_idx, folder_name, task) in indexed_tasks {
                        if task.status == Status::Closed && !args.show_closed {
                                continue;
                        }

                        let tags = task.tags.join(",");
                        let path = store.root.join(&folder_name).join("TASK.md");

                        let mut parts = vec![];

                        parts.push(format!("[PRIORITY: {:>3}]", task.priority));
                        if task.status != Status::Open {
                                parts.push("[STATUS: CLOSED]".to_string())
                        }
                        if !tags.is_empty() {
                                parts.push(format!("[TAGS: {}]", tags))
                        }

                        println!(
                                "{}:7:{}: {} {}",
                                path.display(),
                                display_idx,
                                parts.join(" "),
                                task.title
                        );
                }

                Ok(())
        }

        fn render(&self) -> String {
                let lines = vec![
                        format!("---"),
                        format!("status: {}", self.status),
                        format!("priority: {}", self.priority),
                        format!("tags: [{}]", self.tags.join(", ")),
                        format!("---"),
                        format!(""),
                        format!("# {}", self.title),
                        format!(""),
                        format!("\n"), // Last item doesn't get newlines
                ];

                lines.join("\n")
        }
}

impl FromStr for Task {
        type Err = lexopt::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut status: Option<Status> = None;
                let mut priority: Option<u8> = None;
                let mut tags: Option<Vec<String>> = None;
                let mut title: Option<String> = None;

                let mut lines = s.lines();

                if lines.next() != Some("---") {
                        return Err(lexopt::Error::Custom(
                                "Task file missing front matter delimiter '---'"
                                        .to_string()
                                        .into(),
                        ));
                }

                for line in lines.by_ref().take_while(|l| *l != "---") {
                        let Some((k, v)) = line.split_once(": ") else {
                                continue;
                        };

                        match k.trim() {
                                "status" => status = Some(Status::from_str(v.trim())?),
                                "priority" => {
                                        priority = Some(v.trim().parse::<u8>().map_err(|_| {
                                                lexopt::Error::Custom(
                                                        format!("Invalid priority: '{v}'").into(),
                                                )
                                        })?);
                                },
                                "tags" => {
                                        let cleaned_value =
                                                v.trim().strip_prefix('[')
                                                        .and_then(|t| t.strip_suffix(']'))
                                                        .unwrap_or(v.trim());

                                        if cleaned_value.is_empty() {
                                                tags = Some(Vec::new());
                                        } else {
                                                tags = Some(cleaned_value
                                                        .split(',')
                                                        .map(|t| t.trim().to_string())
                                                        .collect());
                                        }
                                },
                                _ => { /* Ignore unknown front matter keys */ },
                        }
                }

                for line in lines {
                        // Skip empty lines between front matter and title
                        if line.trim().is_empty() {
                                continue;
                        }

                        if let Some(t) = line.strip_prefix("# ") {
                                title = Some(t.to_string());
                                break;
                        } else {
                                #[rustfmt::skip]
                                return Err(lexopt::Error::Custom(format!(
                                        "Expected title line starting with '# ', found: \"{line}\"",
                                ).into()));
                        }
                }

                #[rustfmt::skip]
                let task = Task {
                        title: title.ok_or_else(|| { lexopt::Error::Custom("Task file missing title".to_string().into()) })?,
                        priority: priority.ok_or_else(|| { lexopt::Error::Custom("Task file missing priority".to_string().into()) })?,
                        status: status.ok_or_else(|| { lexopt::Error::Custom("Task file missing status".to_string().into()) })?,
                        tags: tags.unwrap_or_default(),
                };

                Ok(task)
        }
}

pub fn print_global_help() {
        println!("tafsk {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("USAGE:");
        println!("    tafsk [SUBCOMMAND] [FLAGS]");
        println!();
        println!("FLAGS:");
        println!("    -h, --help       Prints help information");
        println!("    -V, --version    Prints version information");
        println!();
        println!("SUBCOMMANDS:");
        println!("    add              Create a new task");
        println!("    done             Mark a task as closed");
        println!("    init             Initialize store with timezone");
        println!("    list             List all tasks");
}
