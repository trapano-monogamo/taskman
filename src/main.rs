mod taskmanager;
mod tasktui;
mod queue;

use tasktui::TUI;

use std::{
    path::{Path, PathBuf},
    env,
};

use dirs::home_dir;

#[allow(unused_assignments)]
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut home_binding: PathBuf = PathBuf::new();
    let save_file: &Path = match args.get(1) {
        Some(arg) => { Path::new(arg) },
        None => {
            home_binding = home_dir().unwrap().join("taskman.json");
            Path::new(home_binding.as_path())
        },
    };
                
    TUI::new(Box::new(save_file)).run();
}
