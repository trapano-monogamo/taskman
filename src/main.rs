mod taskmanager;
mod tasktui;

fn main() {
    let mut tui = tasktui::TUI::new();
    tui.run();
}
