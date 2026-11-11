//! `cargo test` only captures output written with `print!` and friends. Writing to `io::stdout()`
//! directly bypasses it, and the bar would then leak into the test output.

use progress_bar::{pb::ProgressBar, Color, Style};
use std::process::Command;

const CHILD: &str = "PROGRESS_BAR_CAPTURE_CHILD";

#[test]
fn draw_bar() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    let mut progress_bar = ProgressBar::new(3);
    progress_bar.set_action("LEAKED_ACTION", Color::Blue, Style::Bold);
    progress_bar.inc();
    progress_bar.print_info("LEAKED_INFO", "text", Color::Red, Style::Bold);
    progress_bar.print_final_info("LEAKED_FINAL", "text", Color::Red, Style::Bold);
    progress_bar.finalize();
}

#[test]
fn output_is_captured() {
    // Runs `draw_bar` in a child test process, with output capture enabled.
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["draw_bar", "--exact", "--test-threads=1"])
        .env(CHILD, "1")
        .output()
        .unwrap();
    assert!(child.status.success());
    let stdout = String::from_utf8_lossy(&child.stdout);
    assert!(!stdout.contains("LEAKED"), "the bar was not captured:\n{stdout}");
}
