use crate::clock::Clock;
pub use crate::eta::EtaData;
use crate::output::Output;
use crate::style::*;
use core::cell::RefCell;
use core::fmt::{self, Write};
use core::time::Duration;

#[cfg(feature = "std")]
use crate::{clock::StdClock, output::Stdout};

macro_rules! define_progress_bar {
    ($($generics:tt)*) => {
        /// A terminal progress bar.
        ///
        /// It renders to an [`Output`] (stdout by default) and uses a [`Clock`] for ETA estimation.
        /// Use [`ProgressBar::with_output`] to render somewhere else, e.g. a UART on `no_std` targets.
        ///
        /// The action text is stored as an `A`, so the caller decides how it is allocated: a [`String`]
        /// by default with `std`, or e.g. a `&'static str` or a `heapless::String` without allocator.
        pub struct ProgressBar<$($generics)*> {
            max: usize,
            progress: usize,
            width: usize,
            action: Option<A>,
            action_color: Color,
            action_style: Style,
            progress_style: ProgressStyle,
            action_width: usize,
            eta: Option<EtaData>,
            colors: bool,
            output: RefCell<W>,
            clock: C,
        }
    };
}

/// Colors are enabled by default, unless disabled by the `NO_COLOR` environment variable.
///
/// See <https://no-color.org/>: colors are disabled when `NO_COLOR` is present and not empty.
fn default_colors() -> bool {
    #[cfg(feature = "std")]
    return std::env::var_os("NO_COLOR").as_deref().is_none_or(std::ffi::OsStr::is_empty);
    #[cfg(not(feature = "std"))]
    return true;
}

#[cfg(feature = "std")]
define_progress_bar!(W = Stdout, C = StdClock, A = String);
#[cfg(not(feature = "std"))]
define_progress_bar!(W, C, A);

/// Displays through a closure, so a whole line reaches the output as a single `write_fmt` call.
struct DisplayFn<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result>(F);

impl<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result> fmt::Display for DisplayFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

/// Right-aligns `text`, padded or truncated to exactly `width` characters.
fn write_sized(out: &mut impl Write, text: &str, width: usize) -> fmt::Result {
    write!(out, "{text:>width$.width$}")
}

#[cfg(feature = "std")]
impl ProgressBar {
    /// Creates a progress bar with the total number of actions.  
    /// You will need to call inc method when an action is completed and the bar progress will be incremented by 1.  
    /// Don't print things with print macro while the bar is running; use the print_info method instead.  
    /// Colors and styles are disabled when the `NO_COLOR` environment variable is set (see [ProgressBar::set_colors]).  
    /// 
    /// # Example
    /// 
    /// ```
    /// use progress_bar::{pb::ProgressBar, Color, Style};
    /// use std::{thread, time};
    /// 
    /// // if you have 81 pages to load
    /// let mut progress_bar = ProgressBar::new(81);
    /// progress_bar.set_action("Loading", Color::Blue, Style::Bold);
    ///
    /// for i in 0..81 {
    ///     // load page
    ///     thread::sleep(time::Duration::from_millis(10));
    /// 
    ///     // log the result
    ///     if i == 14 {
    ///         progress_bar.print_info("Failed", "https://zefzef.zef", Color::Red, Style::Blink);
    ///     } else {
    ///         progress_bar.print_info("Success", "https://example.com", Color::Red, Style::Blink);
    ///     }
    ///     
    ///     // increase the progress by 1
    ///     progress_bar.inc();
    /// }
    /// progress_bar.print_final_info("Loading", "Load complete", Color::LightGreen, Style::Bold);
    /// // Or, to leave the progress bar at 100%:
    /// // progress_bar.finalize();
    /// ```
    pub fn new(max: usize) -> Self {
        ProgressBar::with_output(max, Stdout, StdClock::default())
    }

    /// Same as [ProgressBar::new] but enabled ETA display.
    /// 
    /// Smoothing factor: A value between `0.1` and `0.3` works well.
    /// Higher value makes the estimate react faster to recent changes; lower makes it less jumpy.
    pub fn new_with_eta(smoothing_factor: f64, max: usize) -> Self {
        let mut progress_bar = ProgressBar::new(max);
        progress_bar.enable_eta(smoothing_factor);
        progress_bar
    }
}

#[cfg(feature = "std")]
impl<C: Clock> ProgressBar<Stdout, C> {
    /// Creates a progress bar using `clock` instead of the system clock for ETA estimation,
    /// e.g. to make the ETA deterministic in tests.
    ///
    /// Any `Fn() -> Duration` is a clock:
    ///
    /// ```
    /// use progress_bar::pb::ProgressBar;
    /// use std::{cell::Cell, time::Duration};
    ///
    /// let time = Cell::new(Duration::ZERO);
    /// let mut progress_bar = ProgressBar::with_clock(100, || time.get());
    /// progress_bar.enable_eta(0.2);
    /// time.set(Duration::from_secs(2));
    /// progress_bar.set_progress(10); // ETA 18s
    /// ```
    pub fn with_clock(max: usize, clock: C) -> Self {
        ProgressBar::with_output(max, Stdout, clock)
    }
}

impl<W: Output, C: Clock, A: AsRef<str>> ProgressBar<W, C, A> {
    /// Creates a progress bar rendering to `output` and using `clock` for ETA estimation.
    ///
    /// This is the constructor to use on `no_std` targets.
    /// Pass [`NoClock`](crate::NoClock) if you don't need ETA.
    ///
    /// The type of the action text, `A`, usually has to be given explicitly:
    /// it is the type in which [`set_action`](ProgressBar::set_action) stores its argument.
    ///
    /// ```
    /// use progress_bar::{pb::ProgressBar, Color, NoClock, Style};
    ///
    /// let mut progress_bar = ProgressBar::<_, _, &'static str>::with_output(10, String::new(), NoClock);
    /// progress_bar.set_action("Loading", Color::Blue, Style::Bold);
    /// progress_bar.inc();
    /// assert!(progress_bar.into_output().contains("1/10"));
    /// ```
    pub fn with_output(max: usize, output: W, clock: C) -> Self {
        ProgressBar {
            max,
            progress: 0,
            width: 50,
            action: None,
            action_color: Color::Black,
            action_style: Style::Normal,
            progress_style: ProgressStyle::Number,
            action_width: 12,
            eta: None,
            colors: default_colors(),
            output: RefCell::new(output),
            clock,
        }
    }

    /// Consumes the progress bar and returns its output.
    pub fn into_output(self) -> W {
        self.output.into_inner()
    }

    pub fn progress(&self) -> usize {
        self.progress
    }

    pub fn max(&self) -> usize {
        self.max
    }

    /// Set the width of the progress bar in characters in console (default: 50)
    pub fn set_width(&mut self, w: usize) {
        self.width = w;
        self.display();
    }

    /// Set the width of the action in characters in the console (default: 12)
    pub fn set_action_width(&mut self, w: usize) {
        self.action_width = w;
        self.display();
    }

    /// Set the style of the progress indicator to the right of the progress bar (default: [ProgressStyle::Number])
    pub fn set_progress_style(&mut self, style: ProgressStyle) {
        self.progress_style = style;
        self.display();
    }

    #[deprecated(note = "Use set_progress instead")]
    pub fn set_progression(&mut self, p: usize) {
        self.set_progress(p)
    }

    /// Set the progres
    pub fn set_progress(&mut self, p: usize) {
        if let Some(eta) = &mut self.eta {
            if p < self.progress {
                eta.reset(p, self.clock.now());
            } else if self.progress < p {
                eta.update(p, self.clock.now());
            }
        }
        self.progress = p;
        self.display();
    }

    /// Set the maximum progress
    pub fn set_max(&mut self, m: usize) {
        self.max = m;
        self.display();
    }

    /// Increment the progress by 1
    pub fn inc(&mut self) {
        self.set_progress(self.progress + 1);
    }

    /// **Resets progress** and enables ETA (resets its style too).
    pub fn enable_eta(&mut self, smoothing_factor: f64) {
        self.progress = 0;
        self.eta = Some(EtaData::new(smoothing_factor, self.clock.now()));
    }

    /// Disables ETA
    pub fn disable_eta(&mut self) {
        self.eta = None;
    }

    /// Sets the ETA style
    pub fn set_eta_style(&mut self, color: Color, style: Style) {
        if let Some(eta) = &mut self.eta {
            eta.color = Some(color);
            eta.style = Some(style);
        }
    }

    /// Resets ETA style
    pub fn reset_eta_style(&mut self) {
        if let Some(eta) = &mut self.eta {
            eta.color = None;
            eta.style = None;
        }
    }

    /// Sets the update period for ETA calculations.
    /// This controls how frequently the speed is estimated, but ETA number will still be updated on every display.
    /// Default is 1 second.
    pub fn set_eta_update_period(&mut self, period: Duration) {
        if let Some(eta) = &mut self.eta {
            eta.update_period = period;
        }
    }

    /// Enables or disables colors and styles.
    ///
    /// By default, they are enabled unless the [`NO_COLOR`](https://no-color.org/) environment variable is set
    /// to a non-empty value (checked with the `std` feature only).
    /// Use this to apply a user preference (like a `--color` flag) that takes precedence.
    pub fn set_colors(&mut self, enabled: bool) {
        self.colors = enabled;
        self.display();
    }

    fn sgr<T: fmt::Display>(&self, code: T) -> Sgr<T> {
        Sgr(code, self.colors)
    }

    /// Set the global action displayed before the progress bar.
    ///
    /// The action is stored as an `A` (a [`String`] with [`ProgressBar::new`]), and padded or truncated
    /// to the action width when the bar is drawn.
    pub fn set_action(&mut self, a: impl Into<A>, c: Color, s: Style) {
        self.action = Some(a.into());
        self.action_color = c;
        self.action_style = s;
        self.display();
    }

    fn write_info(&self, info_name: &str, text: impl fmt::Display, info_color: Color, info_style: Style) {
        let line = DisplayFn(|f| {
            write!(f, "{}{}", self.sgr(info_style), self.sgr(info_color))?;
            write_sized(f, info_name, self.action_width)?;
            writeln!(f, "{} {text}\x1B[K", self.sgr(RESET))
        });
        let _ = write!(self.output.borrow_mut(), "{line}");
    }

    /// Log something, without display update
    pub fn print_final_info(&mut self, info_name: &str, text: impl fmt::Display, info_color: Color, info_style: Style) {
        self.write_info(info_name, text, info_color, info_style);
        self.output.get_mut().flush();
        self.progress = 0;
        if let Some(eta) = &mut self.eta {
            eta.reset(0, self.clock.now());
        }
    }

    /// Log something
    pub fn print_info(&mut self, info_name: &str, text: impl fmt::Display, info_color: Color, info_style: Style) {
        self.write_info(info_name, text, info_color, info_style);
        self.display();
    }

    /// Display the bar
    pub fn display(&self) {
        let out = &mut *self.output.borrow_mut();
        let _ = write!(out, "{}", DisplayFn(|f| self.render(f)));
        out.flush();
    }

    fn render(&self, out: &mut impl Write) -> fmt::Result {
        write!(out, "{}{}", self.sgr(self.action_style), self.sgr(self.action_color))?;
        if let Some(action) = &self.action {
            write_sized(out, action.as_ref(), self.action_width)?;
        }
        write!(out, "{}\x1B[K", self.sgr(RESET))?;

        out.write_str(" [")?;
        for i in 0..self.width {
            if i*self.max/self.width < self.progress {
                if (i+1)*self.max/self.width >= self.progress {
                    out.write_char('>')?;
                } else {
                    out.write_char('=')?;
                }
            } else {
                out.write_char(' ')?;
            }
        }

        out.write_str("] ")?;
        match self.progress_style {
            ProgressStyle::Number => write!(out, "{}/{}", self.progress, self.max)?,
            ProgressStyle::Percentage => if self.max != 0 {
                write!(out, "{}%", (self.progress as f64 / self.max as f64 * 100.0) as usize)?;
            }
            ProgressStyle::Empty => (),
        }

        if let Some(eta) = &self.eta && let Some(remaining) = eta.remaining(self.progress, self.max) {
            let styled = self.colors && (eta.color.is_some() || eta.style.is_some());
            out.write_char(' ')?;
            if styled {
                if let Some(style) = eta.style {
                    write!(out, "{style}")?;
                }
                if let Some(color) = eta.color {
                    write!(out, "{color}")?;
                }
            }

            write!(out, "(ETA {remaining})")?;

            if styled {
                out.write_str(RESET)?;
            }
        }
        out.write_str("\n\x1B[1A")
    }
    
    /// Mark the end of the progress bar - updates will make a 'new' bar
    pub fn finalize(&mut self) {
        self.progress = 0;
        self.eta = None;
        let out = self.output.get_mut();
        let _ = out.write_char('\n');
        out.flush();
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::NoClock;

    #[test]
    fn sized_text() {
        let mut out = String::new();
        write_sized(&mut out, "Loading", 12).unwrap();
        write_sized(&mut out, "A very long action", 12).unwrap();
        write_sized(&mut out, "Exactly12chr", 12).unwrap();
        write_sized(&mut out, "Échec", 3).unwrap();
        write_sized(&mut out, "Écriture télémétrie", 12).unwrap();
        assert_eq!(out, "     LoadingA very long Exactly12chrÉchÉcriture tél");
    }

    #[test]
    fn eta_after_final_info() {
        use core::cell::Cell;
        let time = Cell::new(Duration::ZERO);
        let mut progress_bar = ProgressBar::with_clock(100, || time.get());
        progress_bar.enable_eta(0.2);
        time.set(Duration::from_secs(2));
        progress_bar.set_progress(50);
        progress_bar.print_final_info("Done", "first batch", Color::Green, Style::Bold);
        time.set(Duration::from_secs(4));
        // used to underflow: the ETA still remembered progress 50
        progress_bar.set_progress(10);
    }

    #[test]
    fn colors_disabled() {
        let mut pb = ProgressBar::<_, _, &str>::with_output(4, String::new(), NoClock);
        pb.set_colors(false);
        pb.set_width(4);
        pb.set_action("Loading", Color::Blue, Style::Bold);
        pb.print_info("Hi", "there", Color::Red, Style::Normal);
        let out = pb.into_output();
        assert!(out.ends_with("          Hi there\x1B[K\n     Loading\x1B[K [    ] 0/4\n\x1B[1A"));
    }

    #[test]
    fn action_is_padded_and_truncated() {
        let mut pb: ProgressBar<_, _> = ProgressBar::with_output(4, String::new(), NoClock);
        pb.set_width(4);
        pb.set_action("A very long action", Color::Blue, Style::Bold);
        pb.inc();
        pb.set_colors(true);
        pb.print_info("Hi", "there", Color::Red, Style::Normal);
        let out = pb.into_output();
        assert!(out.contains("A very long \x1B[0m\x1B[K [>   ] 1/4"));
        assert!(out.contains("          Hi\x1B[0m there\x1B[K\n"));
    }

    #[test]
    fn eta_with_custom_clock() {
        use core::cell::Cell;
        let time = Cell::new(Duration::ZERO);
        let mut pb = ProgressBar::<_, _, &str>::with_output(100, String::new(), || time.get());
        pb.enable_eta(0.2);
        time.set(Duration::from_secs(2));
        pb.set_progress(10); // 5 items/s, 90 remaining
        assert!(pb.output.borrow().ends_with("10/100 (ETA 18s)\n\x1B[1A"));

        // a clock that didn't advance doesn't produce an infinite speed
        pb.set_progress(20);
        assert!(pb.into_output().ends_with("20/100 (ETA 16s)\n\x1B[1A"));
    }
}
