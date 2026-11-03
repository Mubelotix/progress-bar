# progress_bar

A simple and elegant crate for displaying progress bars in the terminal.
It can estimate and display the remaining time, and includes logging utilities that integrate cleanly with your progress bar output.

⚠️ Note: This crate currently supports Unix-based systems only. It can also run on `no_std` targets, see [`no_std` / embedded usage](#no_std--embedded-usage).

## Features

- Display a clean terminal progress bar
- Show the current action to the left of the progress bar
- Output log messages above the progress bar
- Estimate and display remaining time
- Integrate seamlessly with Rust’s `log` crate
- Respect the [`NO_COLOR`](https://no-color.org/) environment variable
- `no_std` and allocation-free, for embedded targets

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

![image displaying the output of the code above](https://raw.githubusercontent.com/Mubelotix/cli_progress_bar/refs/heads/master/media/progress_bar_example1.png "Output")

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

### `no_std` / embedded usage

The crate is `no_std` compatible and doesn't need an allocator: disable the default `std` feature.

```toml
progress_bar = { version = "*", default-features = false }
```

Without `std`, the global progress bar and the logger are unavailable: create a `ProgressBar` with
`ProgressBar::with_output`, giving it anything implementing `core::fmt::Write` (a UART, an RTT channel...) and a clock.
Any `Fn() -> Duration` is a clock, so you can plug your hardware timer in. Use `NoClock` if you don't need ETA.

The bar keeps the action text as given to `set_action`, in a type you choose: `&'static str` for literals,
a `heapless::String`, or `alloc::string::String` if you have an allocator.

```rust
use core::{fmt, time::Duration};
use progress_bar::{pb::ProgressBar, Color, Output, Style};

struct Uart;

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // send `s` over the wire
        Ok(())
    }
}

impl Output for Uart {
    // optional: flush the TX buffer once the bar has been drawn
    fn flush(&mut self) {}
}

fn uptime() -> Duration {
    // read your hardware timer here
    Duration::ZERO
}

let mut progress_bar = ProgressBar::<_, _, &'static str>::with_output(64, Uart, uptime as fn() -> Duration);
progress_bar.enable_eta(0.2);
progress_bar.set_action("Flashing", Color::Blue, Style::Bold);

for sector in 0..64 {
    // write sector
    if sector == 14 {
        // `print_info` accepts anything implementing `Display`, no allocation needed
        progress_bar.print_info("Retry", format_args!("sector {sector}"), Color::Yellow, Style::Normal);
    }
    progress_bar.inc();
}

progress_bar.finalize();
```

License: MIT
