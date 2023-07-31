#![allow(dead_code)]

use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    ToDo,
    Doing,
    Done,
}

#[derive(Debug, PartialEq, Eq)]
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

#[allow(dead_code)]
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
            "{} {} {}",
            match self.priority {
                Priority::Low => "[*  ]",
                Priority::Medium => "[** ]",
                Priority::High => "[***]",
            },
            self.title,
            match self.status {
                Status::ToDo => "(ToDo)",
                Status::Doing => "(Doing)",
                Status::Done => "(Done)",
            },
        )
    }
}

impl Task {
    pub fn new(id: u32, priority: Priority, title: &str, description: &str) -> Task {
        Task {
            id,
            title: title.to_owned(),
            description: description.to_owned(),
            priority,
            status: Status::ToDo,
        }
    }
}

pub struct TaskFactory;
impl TaskFactory {
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

pub struct TaskManager {
    tasks: Vec<Task>, 
}

impl TaskManager {
    pub fn new() -> TaskManager {
        TaskManager { tasks: Vec::new() }
    }

    pub fn new_task(&mut self, priority: Priority, title: &str) {
        self.tasks.push(
            Task::new(self.tasks.len() as u32, priority, title, "")
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

    // format a list with all tasks ([Priority] title)
    pub fn log_tasks(&mut self, sort_by: SortBy) {
        match sort_by {
            SortBy::Id => self.tasks.sort_by_key(|e| e.id),
            SortBy::Title => self.tasks.sort_by_key(|e| e.title.chars().nth(0)),
            SortBy::Priority => self.tasks.sort_by_key(|e| e.priority),
            SortBy::Status => self.tasks.sort_by_key(|e| e.status),
            SortBy::None => {},
        };
        for t in self.tasks.iter() {
            println!("{}. {}", t.id, t);
        }
    }

    // format a single task fully ([Priority] title \n description)
    // pub fn show_task(&mut self, task_selector: TaskSelector) -> String {
    //     let task: Option<&mut Task> = match task_selector {
    //         TaskSelector::Title(title) => self.get_task_by_title(title),
    //         TaskSelector::Id(id) => self.get_task_by_id(id),
    //     };
    //     if let Some(t) = task {
    //         return format!("{}. {}\n{}\n", t.id, t, t.description);
    //     } else {
    //         return String::from("");
    //     }
    // }
}
