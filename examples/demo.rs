//! The demo recorded in the README GIF (see `media/demo.tape`).

use progress_bar::*;
use std::{thread::sleep, time::Duration};

fn main() {
    let pages = [
        "https://example.com",
        "https://example.org/about",
        "https://example.net/blog",
        "https://zefzef.zef",
        "https://example.com/contact",
    ];

    init_progress_bar_with_eta(0.3, 60);
    set_progress_bar_action("Loading", Color::Blue, Style::Bold);
    set_eta_style(Color::DarkGray, Style::Italic);
    set_eta_update_period(Duration::from_millis(200));

    for i in 0..60 {
        sleep(Duration::from_millis(if i < 30 { 60 } else { 90 }));

        if i % 12 == 5 {
            let page = pages[i / 12];
            if page.contains("zefzef") {
                print_progress_bar_info("Failed", page, Color::Red, Style::Bold);
            } else {
                print_progress_bar_info("Loaded", page, Color::Green, Style::Normal);
            }
        }
        if i == 40 {
            set_progress_bar_action("Parsing", Color::Magenta, Style::Bold);
        }

        inc_progress_bar();
    }

    print_progress_bar_final_info("Done", "60 pages loaded, 1 failure", Color::LightGreen, Style::Bold);
}
