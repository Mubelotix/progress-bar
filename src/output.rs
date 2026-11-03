//! Output sinks the progress bar can render to.

use core::fmt;

/// Something a [`ProgressBar`](crate::pb::ProgressBar) can write to.
///
/// Any [`fmt::Write`] implementor can be used (a UART, an RTT channel, a `String`...):
/// implement this trait for it, overriding [`Output::flush`] if your sink is buffered.
///
/// ```
/// use core::fmt;
/// use progress_bar::Output;
///
/// struct Uart;
///
/// impl fmt::Write for Uart {
///     fn write_str(&mut self, s: &str) -> fmt::Result {
///         // send `s` over the wire
///         Ok(())
///     }
/// }
///
/// impl Output for Uart {}
/// ```
pub trait Output: fmt::Write {
    /// Flushes buffered output, called after each full redraw of the bar.
    fn flush(&mut self) {}
}

#[cfg(feature = "std")]
impl Output for std::string::String {}

/// Writes to the process' standard output. This is the default output.
///
/// It goes through `print!` rather than `io::stdout()`, so that `cargo test` captures the output.
#[cfg(feature = "std")]
#[derive(Debug, Default, Clone, Copy)]
pub struct Stdout;

#[cfg(feature = "std")]
impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        std::print!("{s}");
        Ok(())
    }

    // A single `print!` takes the stdout lock once, so a line can't be interleaved with other threads' output.
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        std::print!("{args}");
        Ok(())
    }
}

#[cfg(feature = "std")]
impl Output for Stdout {
    fn flush(&mut self) {
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
}
