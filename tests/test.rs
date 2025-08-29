use progress_bar::pb::ProgressBar;
use progress_bar::style::{Color, Style};
use std::thread;
use std::time;

#[test]
fn test() {
    let mut test = ProgressBar::new_with_eta(100);
    test.set_action("Loading", Color::Blue, Style::Bold);

    for i in 0..100 {
        test.inc();
        if i == 14 {
            test.print_info("Failed", "to load a page", Color::Red, Style::Blink);
        } else if i == 48 {
            test.print_info(
                "Found",
                "something interessant",
                Color::LightGreen,
                Style::Normal,
            );
        } else if i == 75 {
            test.print_info(
                "Warning",
                "empty page here",
                Color::Yellow,
                Style::Underlined,
            );
        }
        thread::sleep(time::Duration::from_millis(2));
    }
    test.finalize();

    println!("Normal print macro");

    let mut test = ProgressBar::new(100);
    test.set_action("Loading", Color::Blue, Style::Bold);

    for i in 0..100 {
        test.inc();
        if i == 14 {
            test.print_info("Failed", "to load a page", Color::Red, Style::Blink);
        } else if i == 48 {
            test.print_info(
                "Found",
                "something interessant",
                Color::LightGreen,
                Style::Normal,
            );
        } else if i == 75 {
            test.print_info(
                "Warning",
                "empty page here",
                Color::Yellow,
                Style::Underlined,
            );
        }
        thread::sleep(time::Duration::from_millis(2));
    }
    test.print_final_info("Loading", "Load complete", Color::LightGreen, Style::Bold);
}
