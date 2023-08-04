#![allow(dead_code, unused_variables)]

use std::{
    io::{self, Write},
    str::FromStr
};
use super::taskmanager::*;

extern crate crossterm;
use crossterm::terminal;

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
#[derive(Debug)]
struct ParseCommandError;

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

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Command::None)
    }
}

#[derive(Default, Clone)]
struct Block {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    title: String,
    content: Vec<String>,
}

impl Block {
    fn new(x: usize, y: usize, width: usize, height: usize, title: &str) -> Block {
        Block { x, y, width, height, title: title.to_string(), content: Vec::new() }
    }

    fn draw<W: Write>(&self, handle: &mut W) -> Result<(), io::Error> {
        // Move the cursor to the starting position
        write!(handle, "\x1B[{};{}H", self.y, self.x)?;

        // Draw the top border
        let mut formatted_title = self.title.clone();
        formatted_title.push_str("-".repeat(self.width - self.title.len() - 1).as_str());
        write!(handle, "-{:<width$}", &formatted_title, width = self.width - 2)?;

        for i in 1..self.height-1 {
            let gap = self.width - 2;
            write!(handle, "\x1B[{};{}H", self.y + i, self.x)?;
            write!(
                handle,
                "|{:<width$}|",
                match self.content.get(i-1) {
                    Some(t) => {
                        let mut res = t.clone().trim().to_string();
                        res.truncate(self.width - 2);
                        res
                    },
                    None => { String::from("") },
                },
                width = self.width - 2
            )?;
        }

        // Draw the bottom border
        write!(handle, "\x1B[{};{}H", self.y + self.height - 1, self.x)?;
        for _ in 0..self.width { handle.write_all(b"-")?; }

        Ok(())
    }
}

pub struct TUI {
    pub tm: TaskManager,
    quit: bool,
    err_hist: Vec<String>,
    cmd_hist: Vec<String>,
    blocks: Vec<Block>,
    width: usize,
    height: usize,
}
impl TUI {
    pub fn new() -> TUI {
        let (cols, rows) = terminal::size().unwrap();
        TUI {
            tm: TaskManager::new(),
            quit: false,
            err_hist: Vec::new(),
            cmd_hist: Vec::new(),
            blocks: vec![
                Block::new(0,                   1,                     (cols*1/3-1) as usize, (rows*1/2) as usize, "ToDo"),
                Block::new((cols*1/3) as usize, 1,                     (cols*1/3)   as usize, (rows*1/2) as usize, "Doing"),
                Block::new((cols*2/3) as usize, 1,                     (cols*1/3)   as usize, (rows*1/2) as usize, "Done"),
                Block::new(0,                   (rows*1/2+1) as usize, (cols*1/2-1) as usize, (rows*1/3) as usize, "Errors"),
                Block::new((cols*1/2) as usize, (rows*1/2+1) as usize, (cols*1/2-1) as usize, (rows*1/3) as usize, "Commands"),
            ],
            width: cols as usize,
            height: rows as usize,
        }
    }

    fn display(&mut self) {
        println!("~~~");
        self.tm.log_tasks(SortBy::None);
        println!("~~~");
    }

    fn draw_ui<W: Write>(&self, handle: &mut W) -> Result<(), io::Error> {
        // Clear the screen
        // handle.write_all(b"\x1B[2J")?;
        for y in 0..self.height {
            write!(handle, "\x1B[{};{}H", y, 0)?;
            write!(handle, "{:width$}", "", width = self.width)?;
        }

        // Move the cursor to the top-left corner
        handle.write_all(b"\x1B[H")?;

        for block in self.blocks.iter() {
            block.draw(handle)?;
        }

        // Draw Prompt
        handle.write_all(b"\n")?;
        handle.write_all(b"> ")?;

        handle.flush()?;

        Ok(())
    }

    pub fn run(&mut self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();

        while !self.quit {
            let mut input = String::new();

            self.blocks[0].content = self.tm
                .filter_task_status(Status::ToDo)
                .iter()
                .map(|e| format!("{}. {}", e.id(), e))
                .collect();
            self.blocks[1].content = self.tm
                .filter_task_status(Status::Doing)
                .iter()
                .map(|e| format!("{}. {}", e.id(), e))
                .collect();
            self.blocks[2].content = self.tm
                .filter_task_status(Status::Done)
                .iter()
                .map(|e| format!("{}. {}", e.id(), e))
                .collect();
            self.blocks[3].content = self.err_hist.clone();
            self.blocks[4].content = self.cmd_hist.clone();

            match self.draw_ui(&mut handle) {
                Ok(_) => { },
                Err(e) => { println!("{}", e); },
            }

            // print!("> ");
            std::io::stdin().read_line(&mut input).expect("couldn't read stdin");
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
            Command::Exit => {
                self.quit = true;
                self.tm.save().unwrap();
            },
            Command::None => { },
        };

        Ok(())
    }

    fn process_input(&mut self, input: &str) -> Result<Command, String> {
        // separate input in command and arguments
        let mut binding = input.split(':');
        let cmd = binding.next().unwrap_or("");

        if cmd == "" { return Ok(Command::None); }

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
