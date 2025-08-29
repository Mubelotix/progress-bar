#[test]
fn test() {
    use progress_bar::*;
    use std::thread::sleep;
    use std::time::Duration;

    // if you have 81 pages to load
    init_progress_bar_with_eta(81);
    set_progress_bar_action("Loading", Color::Blue, Style::Bold);

    for i in 0..81 {
        // load page
        sleep(Duration::from_millis(2));

        // log the result
        if i == 14 {
            print_progress_bar_info(
                "Failed",
                "to load https://zefzef.zef",
                Color::Red,
                Style::Normal,
            );
        } else if i == 41 {
            print_progress_bar_info(
                "Success",
                "loading https://example.com",
                Color::Green,
                Style::Bold,
            );
        }

        // increase the progress by 1
        inc_progress_bar();
    }

    finalize_progress_bar();
}
