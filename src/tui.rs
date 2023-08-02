#![allow(dead_code, unused_variables, unused_imports)]

use std::{io::Write, str::FromStr};
use crossterm;

pub use super::taskmanager::*;

struct CommandFailedError;

enum Command {
    Help,
    List(SortBy),
    Add(Priority, String),
    Remove(u32),
    Priority(u32, Priority),
    Status(u32, Status),
    Exit,
}

pub struct TUI {
    pub tm: TaskManager,
    err_hist: Vec<String>,
    cmd_hist: Vec<String>,
}
impl TUI {
    pub fn new() -> TUI {
        TUI {
            tm: TaskManager::new(),
            err_hist: Vec::new(),
            cmd_hist: Vec::new(),
        }
    }
    fn display(&mut self) {
        println!("~~~");
        self.tm.log_tasks(SortBy::None);
        println!("~~~");
    }

    pub fn run(&mut self) {
        loop {
            let mut input = String::from("");

            print!("> ");
            std::io::stdout().flush().expect("Failed to flush stdout...");

            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            println!("");
            std::io::stdout().flush().expect("Failed to flush stdout...");

            self.cmd_hist.push(input.to_owned());

            match self.process_input(&input) {
                Ok(_) => { },
                Err(e) => {
                    println!("[Error] {}", e);
                    self.err_hist.push(e);
                },
            };
        }
    }

    fn execute_command(&mut self, cmd: Command) -> Result<(), CommandFailedError> {
        match cmd {
            Command::Help => { },
            Command::List(sort_by) => { },
            Command::Add(priority, title) => { },
            Command::Remove(id) => { },
            Command::Priority(id, priority) => { },
            Command::Status(id, status) => { },
            Command::Exit => { },
        }

        Ok(())
    }

    fn process_input(&mut self, input: &str) -> Result<(), String> {
        // separate input in command and arguments
        let mut binding = input.split(':');
        let cmd = binding.next().unwrap_or("");

        if let Some(arguments) = binding.next() {
            // separate arguments by commas.
            // for each command, retrieve the elements in the args iterator
            // and parse them to pass arguments to the TaskManager functions
            let args_binding = arguments.split(',').collect::<Vec<&str>>();
            let mut args = args_binding.iter();
            match cmd {
                "help" => {
                    println!(
r#"
Every command follows this syntax:
the name of the command, a semicolon (always) and an optional list of arguments depending on the command.
<command>:<arg1,arg2,...>

List of commands:
* help:
* list:
* add:<priority>,<title>
* remove:<id>
* description:<id>,<description>
* priority:<id>,<new_priority>
* status:<id>,<new_status>
* exit:
"#
                        );
                }
                "list" => {
                    self.display();
                },
                "add" => {
                    let priority: &str = &(*args.next().unwrap_or(&"low")).to_lowercase();
                    let title: &str = args
                        .next()
                        .ok_or(format!("task title is missing..."))?
                        .strip_suffix('\n')
                        .ok_or(format!("could not parse task title..."))?;

                    self.tm.new_task(
                            match Priority::from_str(priority) {
                                Ok(p) => p,
                                Err(e) => return Err(format!("invalid priority argument...")),
                            },
                            title,
                        );
                },
                "remove" => {
                    let id = args
                            .next()
                            .ok_or(format!("task id missing..."))?
                            .strip_suffix('\n')
                            .ok_or(format!("could not read task id..."))?
                            .parse::<u32>()
                            .ok().ok_or(format!("could not parse task id..."))?;
                    self.tm.remove_task(TaskSelector::Id(id));
                },
                "description" => { },           // <--------------
                "priority" => {
                    let id = args
                        .next()
                        .ok_or(format!("task id is missing..."))?
                        .parse::<u32>()
                        .ok().ok_or(format!("could not parse task id..."))?;
                    let priority = match Priority::from_str(args
                                                            .next()
                                                            .ok_or(format!("priority argument is missing..."))?
                                                            .strip_suffix('\n')
                                                            .ok_or(format!("could not read priority argument..."))?)
                    {
                        Ok(p) => p,
                        Err(_) => return Err(format!("invalid priority argument...")),
                    };
                    self.tm
                        .change_task_priority(TaskSelector::Id(id), priority)
                        .ok().ok_or(format!("could not find task with id '{}'", id))?;
                },
                "status" => {
                    let id = args
                        .next()
                        .ok_or(format!("task id is missing..."))?
                        .parse::<u32>()
                        .ok().ok_or(format!("could not parse task id..."))?;
                    let status = match Status::from_str(args
                                                        .next()
                                                        .ok_or(format!("status argument is missing..."))?
                                                        .strip_suffix('\n')
                                                        .ok_or(format!("could not read status argument..."))?)
                    {
                        Ok(s) => s,
                        Err(_) => return Err(format!("invalid status argument...")),
                    };
                    self.tm
                        .change_task_status(TaskSelector::Id(id), status)
                        .ok().ok_or(format!("could not find task with id '{}'", id))?;
                },
                "exit" => {
                    // save tasks to file and exit
                },
                "" => { /* do nothing */ },
                _ => return Err(format!("'{}' is not a valid command...", cmd)),
            }
        } else {
            return Err(format!("command arguments are missing..."));
        }

        Ok(())
    }
}
