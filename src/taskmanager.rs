#![allow(dead_code)]

use std::{
    fmt::Display,
    str::FromStr,
    fs::OpenOptions,
    io::{Write, Read},
    path::Path,
};

use serde::{Serialize, Deserialize};



// ..:: Priority ::..

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Default for Priority {
    fn default() -> Self { Priority::Low }
}

pub struct ParsePriorityError;

impl FromStr for Priority {
    type Err=ParsePriorityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Err(ParsePriorityError),
        }
    }
}



// ..:: Status ::..

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    ToDo,
    Doing,
    Done,
}

impl Default for Status {
    fn default() -> Self { Status::ToDo }
}

pub struct ParseStatusError;

impl FromStr for Status {
    type Err=ParseStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "todo" => Ok(Self::ToDo),
            "doing" => Ok(Self::Doing),
            "done" => Ok(Self::Done),
            _ => Err(ParseStatusError),
        }
    }
}



// ..:: Task ::..

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    id: u32,
    title: String,
    description: String,
    priority: Priority,
    status: Status,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "{}. {} {}",
            self.id,
            match self.priority {
                Priority::Low => "[*  ]",
                Priority::Medium => "[** ]",
                Priority::High => "[***]",
            },
            self.title,
        )
    }
}

impl Task {
    pub fn new(id: u32, title: &str, description: &str, priority: Priority, status: Status) -> Task {
        Task {
            id,
            title: title.to_owned(),
            description: description.to_owned(),
            priority,
            status,
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn log(&self) -> String {
        format!("{}\n{}", self, self.description)
    }
}



pub enum TaskSelector {
    Title(&'static str),
    Id(u32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TaskNotFountError;



pub enum SortBy {
    Priority,
    Title,
    Id,
    Status,
    None,
}



// ..:: TaskManager ::..

pub struct TaskManager<'a> {
    tasks: Vec<Task>, 
    save_file: Box<&'a Path>,
}

impl<'a> TaskManager<'a> {
    pub fn new(save_file: Box<&'a Path>) -> Result<TaskManager, String> {
        let mut f = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(*save_file)
            .ok()
            .ok_or(format!("could not open file '{}'...", save_file
                           .to_str().unwrap_or("")))?;
        let mut buffer = String::new();
        f.read_to_string(&mut buffer)
            .ok().ok_or(format!("could not read file to buffer..."))?;
        drop(f);
        Ok(TaskManager {
            tasks: serde_json::from_str(&buffer)
                .ok().ok_or(format!("couldn't deserialize file content..."))?,
            save_file,
        })
    }

    pub fn default(save_file: Box<&'a Path>) -> TaskManager {
        TaskManager { tasks: Vec::new(), save_file }
    }

    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_tasks = serde_json::to_string(&self.tasks)?;

        let mut f = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(*self.save_file)?;

        f.write_all(serialized_tasks.as_bytes())?;

        drop(f);

        Ok(())
    }

    pub fn new_task(&mut self, title: &str, description: &str, priority: Priority, status: Status) {
        self.tasks.push(
            Task::new(self.tasks.len() as u32, title, description, priority, status)
        );
    }

    pub fn remove_task(&mut self, task_selector: TaskSelector) {
        match task_selector {
            TaskSelector::Title(title) => {
                let mut i = 0;
                while i < self.tasks.len() {
                    if self.tasks[i].title == title {
                        self.tasks.remove(i);
                    } else { i += 1; }
                }
            },
            TaskSelector::Id(id) => {
                let mut i = 0;
                while i < self.tasks.len() {
                    if self.tasks[i].id == id {
                        self.tasks.remove(i);
                    } else { i += 1; }
                }
            },
        };
    }

    pub fn get_task_by_title(&mut self, title: &str) -> Option<&mut Task> {
        for t in self.tasks.iter_mut() {
            if t.title == title {
                return Some(t);
            }
        }
        return None
    }
    pub fn get_task_by_id(&mut self, id: u32) -> Option<&mut Task> {
        for t in self.tasks.iter_mut() {
            if t.id == id {
                return Some(t);
            }
        }
        return None
    }

    pub fn change_task_status(&mut self, task_selector: TaskSelector, new_status: Status) -> Result<(), TaskNotFountError> {
        let task: Option<&mut Task> = match task_selector {
            TaskSelector::Title(title) => self.get_task_by_title(title),
            TaskSelector::Id(id) => self.get_task_by_id(id),
        };
        if let Some(t) = task {
            t.status = new_status;
        } else {
            return Err(TaskNotFountError);
        }
        Ok(())
    }

    pub fn change_task_priority(&mut self, task_selector: TaskSelector, new_priority: Priority) -> Result<(), TaskNotFountError> {
        let task: Option<&mut Task> = match task_selector {
            TaskSelector::Title(title) => self.get_task_by_title(title),
            TaskSelector::Id(id) => self.get_task_by_id(id),
        };
        if let Some(t) = task {
            t.priority = new_priority;
        } else {
            return Err(TaskNotFountError);
        }
        Ok(())
    }

    pub fn filter_task_status(&self, status: Status) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|e| e.status == status)
            .collect()
    }

    pub fn log_tasks<W: Write>(&mut self, handle: &mut W, sort_by: SortBy) {
        match sort_by {
            SortBy::Id => self.tasks.sort_by_key(|e| e.id),
            SortBy::Title => self.tasks.sort_by_key(|e| e.title.chars().nth(0)),
            SortBy::Priority => self.tasks.sort_by_key(|e| e.priority),
            SortBy::Status => self.tasks.sort_by_key(|e| e.status),
            SortBy::None => {},
        };
        for t in self.tasks.iter() {
            write!(handle, "{}. {}", t.id, t).unwrap();
        }
    }
}
