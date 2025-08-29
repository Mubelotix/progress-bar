use std::sync::{LazyLock, Mutex};
use crate::{pb::ProgressBar, style::{Color, Style}, ProgressStyle};

pub static CURRENT_PROGRESS_BAR: LazyLock<Mutex<Option<ProgressBar>>> = LazyLock::new(|| Mutex::new(None));

pub fn has_progress_bar() -> bool {
    CURRENT_PROGRESS_BAR.lock().unwrap().is_some()
}

pub fn set_progress_bar(progress_bar: ProgressBar) {
    *CURRENT_PROGRESS_BAR.lock().unwrap() = Some(progress_bar);
}

pub fn init_progress_bar(max: usize) {
    let progress_bar = ProgressBar::new(max);
    set_progress_bar(progress_bar);
}

pub fn init_progress_bar_with_eta(max: usize) {
    let progress_bar = ProgressBar::new_with_eta(max);
    set_progress_bar(progress_bar);
}

#[deprecated(note = "Use set_progress_bar_progress instead")]
pub fn set_progress_bar_progression(progress: usize) {
    set_progress_bar_progress(progress);
}

pub fn set_progress_bar_progress(progress: usize) {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.set_progress(progress),
        None => eprintln!("ERROR: Unable to set progress bar progress (no progress bar)"),
    }
}

pub fn inc_progress_bar() {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.inc(),
        None => eprintln!("ERROR: Unable to increase progress bar progress (no progress bar)"),
    }
}

pub fn set_progress_bar_width(width: usize) {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.set_width(width),
        None => eprintln!("ERROR: Unable to set progress bar width (no progress bar)"),
    }
}

pub fn set_action_width(width: usize) {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.set_action_width(width),
        None => eprintln!("ERROR: Unable to set progress bar width (no progress bar)"),
    }
}

pub fn set_progress_bar_max(max: usize) {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.set_max(max),
        None => eprintln!("ERROR: Unable to set progress bar max (no progress bar)"),
    }
}

/// Warning: This resets progress to 0
pub fn enable_eta() {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.enable_eta(),
        None => eprintln!("ERROR: Unable to set progress bar max (no progress bar)"),
    }
}

pub fn disable_eta() {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.disable_eta(),
        None => eprintln!("ERROR: Unable to set progress bar max (no progress bar)"),
    }
}

pub fn set_progress_style(style: ProgressStyle) {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.set_progress_style(style),
        None => eprintln!("ERROR: Unable to set progress bar max (no progress bar)"),
    }
}

pub fn print_progress_bar_info(info_name: &str, text: &str, info_color: Color, info_style: Style) {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.print_info(info_name, text, info_color, info_style),
        None => eprintln!("ERROR: Unable to print progress bar info (no progress bar)"),
    }
}

pub fn set_progress_bar_action(action: &str, color: Color, style: Style) {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.set_action(action, color, style),
        None => eprintln!("ERROR: Unable to set progress bar action (no progress bar)"),
    }
}

pub fn print_progress_bar_final_info(info_name: &str, text: &str, info_color: Color, info_style: Style) {
    match *CURRENT_PROGRESS_BAR.lock().unwrap() {
        Some(ref mut progress_bar) => progress_bar.print_final_info(info_name, text, info_color, info_style),
        None => eprintln!("ERROR: Unable to print progress bar final info (no progress bar)"),
    }
}

pub fn finalize_progress_bar() {
    match CURRENT_PROGRESS_BAR.lock().unwrap().take() {
        Some(mut progress_bar) => progress_bar.finalize(),
        None => eprintln!("ERROR: Unable to finalize progress bar (no progress bar)"),
    }
}
