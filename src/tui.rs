#![allow(dead_code, unused_variables, unused_imports)]

use std::{io::Write, str::FromStr};
use crossterm;

pub use super::taskmanager::*;

static HELP_MSG: &str =
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
"#;

#[derive(Debug)]
struct CommandFailedError;

enum Command {
    Help,
    List(SortBy),
    Add(Priority, String),
    Remove(u32),
    Priority(u32, Priority),
    Status(u32, Status),
    Exit,
    None,
}

pub struct TUI {
    pub tm: TaskManager,
    quit: bool,
    err_hist: Vec<String>,
    cmd_hist: Vec<String>,
}
impl TUI {
    pub fn new() -> TUI {
        TUI {
            tm: TaskManager::new(),
            quit: false,
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
        while !self.quit {
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
                Ok(cmd) => {
                    match self.execute_command(cmd) {
                        Ok(_) => { },
                        Err(_) => { println!("[!] command failed..."); },
                    };
                },
                Err(e) => {
                    println!("[Error] {}", e);
                    self.err_hist.push(e);
                },
            };
        }
    }

    fn execute_command(&mut self, cmd: Command) -> Result<(), CommandFailedError> {
        match cmd {
            Command::Help => { println!("{}", HELP_MSG) },
            Command::List(sort_by) => { self.display() },
            Command::Add(priority, title) => { self.tm.new_task(priority, &title) },
            Command::Remove(id) => { self.tm.remove_task(TaskSelector::Id(id)) },
            Command::Priority(id, priority) => {
                self.tm
                    .change_task_priority(TaskSelector::Id(id), priority)
                    .ok().ok_or(CommandFailedError)?;
            },
            Command::Status(id, status) => {
                self.tm
                    .change_task_status(TaskSelector::Id(id), status)
                    .ok().ok_or(CommandFailedError)?;
            },
            Command::Exit => { self.quit = true; },
            Command::None => { },
        };

        Ok(())
    }

    fn process_input(&mut self, input: &str) -> Result<Command, String> {
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
                "help" => { return Ok(Command::Help); }

                "list" => { return Ok(Command::List(SortBy::None)); },

                "add" => {
                    let priority: Priority = match Priority::from_str(
                            &(*args.next().unwrap_or(&"low")).to_lowercase())
                    {
                        Ok(p) => p,
                        Err(e) => return Err(format!("invalid priority argument...")),
                    };
                    let title: &str = args
                        .next()
                        .ok_or(format!("task title is missing..."))?
                        .strip_suffix('\n')
                        .ok_or(format!("could not parse task title..."))?;

                    return Ok(Command::Add(priority, title.to_string()));
                },

                "remove" => {
                    let id = args
                            .next()
                            .ok_or(format!("task id missing..."))?
                            .strip_suffix('\n')
                            .ok_or(format!("could not read task id..."))?
                            .parse::<u32>()
                            .ok().ok_or(format!("could not parse task id..."))?;
                    return Ok(Command::Remove(id));
                },

                "description" => { return Ok(Command::None); },

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
                    return Ok(Command::Priority(id, priority));
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
                    return Ok(Command::Status(id, status));
                },

                "exit" => { return Ok(Command::Exit); },

                "" => { return Ok(Command::None); },

                _ => return Err(format!("'{}' is not a valid command...", cmd)),
            }
        } else {
            return Err(format!("invalid syntax..."));
        }
    }
}
