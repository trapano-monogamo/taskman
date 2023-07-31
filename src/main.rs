mod taskmanager;
mod tui;

fn main() {
    let mut tui = tui::TUI::new();
    tui.run();
}
