# progress_bar

![animated demo of a progress bar with an ETA and logs printed above it](https://raw.githubusercontent.com/Mubelotix/progress-bar/refs/heads/master/media/demo.gif "Demo")

A simple and elegant crate for displaying progress bars in the terminal.
It can estimate and display the remaining time, and includes logging utilities that integrate cleanly with your progress bar output.

⚠️ Note: This crate currently supports Unix-based systems only.

## Features

- Display a clean terminal progress bar
- Show the current action to the left of the progress bar
- Output log messages above the progress bar
- Estimate and display remaining time
- Integrate seamlessly with Rust’s `log` crate
- Respect the [`NO_COLOR`](https://no-color.org/) environment variable

## Example

### Using the global progress bar

The crate offers a global progress bar interface for convenience, so you don't have to manually manage `ProgressBar` instances.

```rust
use progress_bar::*;
use std::{thread::sleep, time::Duration};

// if you have 81 pages to load
init_progress_bar(81);
set_progress_bar_action("Loading", Color::Blue, Style::Bold);

for i in 0..81 {
    // load page
    sleep(Duration::from_millis(100));

    // log the result
    if i == 14 {
        print_progress_bar_info("Failed", "to load https://zefzef.zef", Color::Red, Style::Normal);
    } else if i == 41 {
        print_progress_bar_info("Success", "loading https://example.com", Color::Green, Style::Bold);
    }

    // increase the progress by 1
    inc_progress_bar();
}

finalize_progress_bar();
```

### Logging with the progress bar

Calls to print while a progress bar is active will mess with the output.
To avoid this, you can use the `print_progress_bar_info` function to log messages while the progress bar is active.

#### Basic usage

```rust
use progress_bar::*;

init_progress_bar(100);

print_progress_bar_info("Loading", "website https://example.com", Color::Blue, Style::Bold);
print_progress_bar_info("Failed", "to load https://zefzef.zef", Color::Red, Style::Normal);
```

If you find this syntax too verbose, the crate allows you to use the progress bar as a logger directly.

Enable to `logger` feature to use this functionality.

```toml
# Add the logger feature in your Cargo.toml
progress_bar = { version = "*", features = ["logger"] }
```

```rust
use progress_bar::*;
use log::*;

init_logger().unwrap();
init_progress_bar(100);

info!("Loading website https://example.com");
warn!("Failed to load https://zefzef.zef");
```

#### Integrating with `env_logger`

It is also possible to set another logger to handle calls to logging functions, while 

```rust
use progress_bar::*;
use env_logger::Env;
use log::*;

let inner = env_logger::Builder::from_env(Env::default()).build();
init_logger_with_inner(inner).unwrap();

info!("Loading website https://example.com");
warn!("Failed to load https://zefzef.zef");
```

<!-- 
TODO: Show the fallback logger
-->

License: MIT
