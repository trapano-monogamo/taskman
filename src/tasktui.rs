#![allow(dead_code, unused_variables)]

use std::{
    io::{self, Write},
    path::Path,
    str::FromStr
};
use super::taskmanager::*;
use super::queue::Queue;

extern crate crossterm;
use crossterm::terminal;

static HELP_MSG: &str =
r#"
<command> <arg1> <arg2> ...

List of commands:
* help
* list
* add "<title>" "<optional:description>" <optional:priority> <optional:status>
* remove <id>
* description <id> <description>
* priority <id> <new_priority>
* status <id> <new_status>
* exit
"#;

#[derive(Debug)]
struct CommandFailedError(String);
#[derive(Debug)]
struct ParseCommandError;

enum Command {
    Help,
    Show(u32),
    Add(String, String, Priority, Status),
    Description(u32, String),
    Remove(u32),
    Priority(u32, Priority),
    Status(u32, Status),
    Save,
    Quit,
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

pub struct TUI<'a> {
    pub tm: TaskManager<'a>,
    quit: bool,
    err_hist: Queue<String>,
    cmd_hist: Queue<String>,
    log_buf: Vec<String>,
    blocks: Vec<Block>,
    width: usize,
    height: usize,
}
impl<'a> TUI<'a> {
    pub fn new(save_file: Box<&'a Path>) -> TUI {
        let (cols, rows) = terminal::size().unwrap();
        let queue_cap = (rows*1/6) as usize - 2;
        let mut err_hist = Queue::<String>::new(queue_cap);
        TUI {
            tm: match TaskManager::new(save_file.clone()) {
                Ok(tm) => { tm },
                Err(e) => {
                    err_hist.push(e);
                    TaskManager::default(save_file)
                },
            },
            quit: false,
            err_hist,
            cmd_hist: Queue::new(queue_cap),
            log_buf: Vec::new(),
            blocks: vec![
                Block::new(0,                   1,                     (cols*1/3-1) as usize, (rows*1/2)   as usize, "ToDo"),
                Block::new((cols*1/3) as usize, 1,                     (cols*1/3)   as usize, (rows*1/2)   as usize, "Doing"),
                Block::new((cols*2/3) as usize, 1,                     (cols*1/3)   as usize, (rows*1/2)   as usize, "Done"),
                Block::new(0,                   (rows*1/2+1) as usize, (cols*1/2-1) as usize, (rows*1/6)   as usize, "Errors"),
                Block::new((cols*1/2) as usize, (rows*1/2+1) as usize, (cols*1/2-1) as usize, (rows*1/6)   as usize, "Commands"),
                Block::new(0,                   (rows*2/3)   as usize, (cols)       as usize, (rows*1/3-1) as usize, "Show"),
            ],
            width: cols as usize,
            height: rows as usize,
        }
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
                .map(|e| format!("{}", e))
                .collect();
            self.blocks[1].content = self.tm
                .filter_task_status(Status::Doing)
                .iter()
                .map(|e| format!("{}", e))
                .collect();
            self.blocks[2].content = self.tm
                .filter_task_status(Status::Done)
                .iter()
                .map(|e| format!("{}", e))
                .collect();
            self.blocks[3].content = self.err_hist.clone_elements();
            self.blocks[4].content = self.cmd_hist.clone_elements();
            self.blocks[5].content = self.log_buf.clone();

            match self.draw_ui(&mut handle) {
                Ok(_) => { },
                Err(e) => { println!("{}", e); },
            }

            std::io::stdin().read_line(&mut input).expect("couldn't read stdin");
            input = input.trim_end().to_owned();
            self.cmd_hist.push(input.to_owned());

            match self.process_input(&input) {
                Ok(cmd) => {
                    match self.execute_command(cmd) {
                        Ok(_) => { },
                        Err(e) => { self.err_hist.push(e) },
                    };
                },
                Err(e) => { self.err_hist.push(e); },
            };
        }
    }

    fn execute_command(&mut self, cmd: Command) -> Result<(), String> {
        match cmd {
            Command::Help => { println!("{}", HELP_MSG) },
            Command::Show(id) => {
                let task = self.tm
                    .get_task_by_id(id)
                    .ok_or(format!("could not find task with id '{}'...", id))?;
                let mut buffer = format!("{}", task.log());
                let mut ccount = 0;
                let mut indices = Vec::<usize>::new();
                let mut i = 0;
                for c in buffer.chars() {
                    if c == '\n' {
                        ccount = 0;
                    } else { ccount += 1; }
                    if ccount >= self.blocks
                        .last()
                        .ok_or(format!("couldn't find 'Log' ui block..."))?
                        .width - 2
                    {
                        indices.push(i);
                    }
                    i += 1;
                }
                for j in indices {
                    buffer.insert(j, '\n');
                }
                self.log_buf = buffer
                    .split('\n')
                    .map(|e| e.to_string())
                    .filter(|e| *e != "".to_string())
                    .collect();
            },
            Command::Add(title, description, priority, status) => { self.tm.new_task(&title, &description, priority, status) },
            Command::Description(id, description) => { }
            Command::Remove(id) => { self.tm.remove_task(TaskSelector::Id(id)) },
            Command::Priority(id, priority) => {
                self.tm
                    .change_task_priority(TaskSelector::Id(id), priority)
                    .ok().ok_or(format!("could not find task with id '{}'...", id))?;
            },
            Command::Status(id, status) => {
                self.tm
                    .change_task_status(TaskSelector::Id(id), status)
                    .ok().ok_or(format!("could not find task with id '{}'...", id))?;
            },
            Command::Save => {
                match self.tm.save() {
                    Ok(_) => { },
                    Err(e) => { return Err(format!("{}", e)); },
                };
            },
            Command::Quit => { self.quit = true; },
            Command::None => { },
        };

        Ok(())
    }

    fn process_input(&self, input: &str) -> Result<Command, String> {
        // tokenize input
        let mut tokens: Vec<String> = Vec::new();
        let mut inside_quote = false;
        let mut current_argument = String::new();

        for input_slice in input.split_whitespace() {
            if input_slice.starts_with('"') { inside_quote = true; }
            if inside_quote {
                // current_argument.push_str(" ");
                // current_argument.push_str(input_slice);
                current_argument = [current_argument, input_slice.to_string()].join(" ");
            } else {
                current_argument = input_slice.to_owned();
            }
            if input_slice.ends_with('"') { inside_quote = false; }
            if !inside_quote {
                tokens.push(current_argument.trim().to_owned());
                current_argument.clear();
            }
        }

        tokens.retain(|e| *e != "");

        let mut tokens = tokens.iter();
        match tokens.next() {
            Some(cmd) => {
                match cmd.as_str() {
                    "help" => {
                        if let Some(_) = tokens.next() {
                            return Err(format!("Unexpected arguments for command '{}'...", cmd));
                        } else { return Ok(Command::Help); }
                    },
                    "show" => {
                        let id = tokens
                            .next()
                            .ok_or(format!("Missing <task_id> argument..."))?
                            .parse::<u32>()
                            .ok().ok_or(format!("Invalid <task_id> argument..."))?;
                        return Ok(Command::Show(id));
                    },
                    "add" => {
                        let title = tokens
                            .next()
                            .ok_or(format!("Missing <title> argument..."))?
                            .trim_matches('"');
                        let description = match tokens.next() {
                            Some(s) => { s.trim_matches('"') },
                            None => { "" },
                        };
                        let priority = match tokens.next() {
                            Some(p) => {
                                Priority::from_str(p)
                                    .ok()
                                    .ok_or(format!("Invalid priority argument..."))?
                            },
                            None => { Priority::default() },
                        };
                        let status = match tokens.next() {
                            Some(s) => {
                                Status::from_str(s)
                                    .ok()
                                    .ok_or(format!("Invalid status argument..."))?
                            },
                            None => { Status::default() },
                        };

                        return Ok(Command::Add((*title).to_owned(), (*description).to_owned(), priority, status));
                    },
                    "description" => { return Err(format!("Command 'description' not implemented yet...")); },
                    "remove" => {
                        let id = tokens
                            .next()
                            .ok_or(format!("Missing <task_id> argument..."))?
                            .parse::<u32>()
                            .ok().ok_or(format!("Invalid <task_id> argument..."))?;
                        return Ok(Command::Remove(id));
                    },
                    "priority" => {
                        let id = tokens
                            .next()
                            .ok_or(format!("Missing <task_id> argument..."))?
                            .parse::<u32>()
                            .ok().ok_or(format!("Invalid <task_id> argument..."))?;
                        let priority = Priority::from_str(tokens
                                .next()
                                .ok_or(format!("Missing <new_priority> argument..."))?
                            ).ok().ok_or(format!("Invalid <new_priority> argument..."))?;
                        return Ok(Command::Priority(id, priority));
                    },
                    "status" => {
                        let id = tokens
                            .next()
                            .ok_or(format!("Missing <task_id> argument..."))?
                            .parse::<u32>()
                            .ok().ok_or(format!("Invalid <task_id> argument..."))?;
                        let status = Status::from_str(tokens
                                .next()
                                .ok_or(format!("Missing <new_status> argument..."))?
                            ).ok().ok_or(format!("Invalid <new_status> argument..."))?;
                        return Ok(Command::Status(id, status));
                    },
                    "save" => {
                        if let Some(_) = tokens.next() {
                            return Err(format!("Unexpected arguments for command '{}'...", cmd));
                        } else { return Ok(Command::Save); }
                    },
                    "quit" => {
                        if let Some(_) = tokens.next() {
                            return Err(format!("Unexpected arguments for command '{}'...", cmd));
                        } else { return Ok(Command::Quit); }
                    },
                    _ => { return Err(format!("Invalid command '{}'...", cmd)); },
                }
            },
            None => { return Ok(Command::None); },
        };
    }
}
