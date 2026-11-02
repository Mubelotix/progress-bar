use crate::style::*;
use std::ffi::OsStr;
use std::fmt::{self, Write};
use std::io::{self, Write as _};
use std::time::{Duration, Instant};

pub struct EtaData {
    smoothing_factor: f64,
    last_progress: usize,
    last_time: Instant,
    speed: f64,
    color: Option<Color>,
    style: Option<Style>,
    update_period: Duration,
}

impl EtaData {
    fn new(smoothing_factor: f64) -> Self {
        EtaData {
            smoothing_factor,
            last_progress: 0,
            last_time: Instant::now(),
            speed: 0.0,
            color: None,
            style: None,
            update_period: Duration::from_secs(1),
        }
    }
}

pub struct ProgressBar {
    max: usize,
    progress: usize,
    width: usize,
    action: Option<String>,
    action_color: Color,
    action_style: Style,
    progress_style: ProgressStyle,
    action_width: usize,
    eta: Option<EtaData>,
    colors: bool,
}

const RESET: &str = "\x1B[0m";

/// An SGR escape code (color or style), displayed only when colors are enabled.
struct Sgr<T>(T, bool);

impl<T: fmt::Display> fmt::Display for Sgr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.1 {
            self.0.fmt(f)
        } else {
            Ok(())
        }
    }
}

/// Writes to stdout, so that rendering doesn't need intermediate `String`s.
struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        io::stdout().write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

/// Rounds a non-negative float up, saturating at `usize::MAX`.
// `f64::ceil` isn't available in `core`, this will allow dropping `std`.
fn ceil_usize(x: f64) -> usize {
    let truncated = x as usize;
    if (truncated as f64) < x {
        truncated.saturating_add(1)
    } else {
        truncated
    }
}

/// Right-aligns `text`, padded or truncated to exactly `width` characters.
fn write_sized(out: &mut impl Write, text: &str, width: usize) -> fmt::Result {
    write!(out, "{text:>width$.width$}")
}

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
            colors: std::env::var_os("NO_COLOR").as_deref().is_none_or(OsStr::is_empty),
        }
    }

    /// Same as [ProgressBar::new] but enabled ETA display.
    /// 
    /// Smoothing factor: A value between `0.1` and `0.3` works well.
    /// Higher value makes the estimate react faster to recent changes; lower makes it less jumpy.
    pub fn new_with_eta(smoothing_factor: f64, max: usize) -> Self {
        ProgressBar {
            eta: Some(EtaData::new(smoothing_factor)),
            ..ProgressBar::new(max)
        }
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
                eta.last_progress = p;
                eta.last_time = Instant::now();
                eta.speed = 0.0;
            } else if self.progress < p {
                let now = Instant::now();
                let delta_time = now.duration_since(eta.last_time).as_secs_f64();

                // Only update EMA if a meaningful time window has elapsed
                if delta_time >= eta.update_period.as_secs_f64() {
                    let delta_progress = (p - eta.last_progress) as f64;
                    let instant_speed = delta_progress / delta_time;
                    if eta.speed == 0.0 {
                        eta.speed = instant_speed;
                    } else {
                        eta.speed = eta.smoothing_factor * instant_speed + (1.0 - eta.smoothing_factor) * eta.speed;
                    }
                    eta.last_progress = p;
                    eta.last_time = now;
                }
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
        self.eta = Some(EtaData::new(smoothing_factor));
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
    /// to a non-empty value. Use this to apply a user preference (like a `--color` flag) that takes precedence.
    pub fn set_colors(&mut self, enabled: bool) {
        self.colors = enabled;
        self.display();
    }

    fn sgr<T: fmt::Display>(&self, code: T) -> Sgr<T> {
        Sgr(code, self.colors)
    }

    /// Set the global action displayed before the progress bar.
    pub fn set_action(&mut self, a: &str, c: Color, s: Style) {
        self.action = Some(a.to_string());
        self.action_color = c;
        self.action_style = s;
        self.display();
    }

    fn write_info(&self, info_name: &str, text: impl fmt::Display, info_color: Color, info_style: Style) {
        let out = &mut Stdout;
        let _ = (|| {
            write!(out, "{}{}", self.sgr(info_style), self.sgr(info_color))?;
            write_sized(out, info_name, self.action_width)?;
            writeln!(out, "{} {text}\x1B[K", self.sgr(RESET))
        })();
    }

    /// Log something, without display update
    pub fn print_final_info(&mut self, info_name: &str, text: impl fmt::Display, info_color: Color, info_style: Style) {
        self.write_info(info_name, text, info_color, info_style);
        self.progress = 0;
        if let Some(eta) = &mut self.eta {
            eta.last_progress = 0;
            eta.last_time = Instant::now();
            eta.speed = 0.0;
        }
    }

    /// Log something
    pub fn print_info(&mut self, info_name: &str, text: impl fmt::Display, info_color: Color, info_style: Style) {
        self.write_info(info_name, text, info_color, info_style);
        self.display();
    }

    /// Display the bar
    pub fn display(&self) {
        let _ = self.render(&mut Stdout);
        let _ = io::stdout().flush();
    }

    fn render(&self, out: &mut impl Write) -> fmt::Result {
        write!(out, "{}{}", self.sgr(self.action_style), self.sgr(self.action_color))?;
        if let Some(action) = &self.action {
            write_sized(out, action, self.action_width)?;
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

        if let Some(eta) = &self.eta && self.max != 0 && eta.speed != 0.0 && self.progress != self.max {
            let remaining_progress = self.max.saturating_sub(self.progress) as f64;
            let remaining_time = remaining_progress / eta.speed;
            let remaining_ms = (remaining_time * 1000.0) as usize;

            const SECS_110: usize = 110 * 1000;
            const MINS_110: usize = 110 * 60 * 1000;
            const HOURS_46: usize = 46 * 60 * 60 * 1000;

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

            #[allow(overlapping_range_endpoints)]
            #[allow(clippy::match_overlapping_arm)]
            match remaining_ms {
                0..=3_000 => write!(out, "(ETA {}ms)", remaining_ms)?,
                3_001..=SECS_110 => write!(out, "(ETA {}s)", ceil_usize(remaining_time))?,
                SECS_110..=MINS_110 => write!(out, "(ETA {} minutes)", ceil_usize(remaining_time / 60.))?,
                MINS_110..=HOURS_46 => write!(out, "(ETA {} hours)", ceil_usize(remaining_time / 3600.))?,
                _ => write!(out, "(ETA {} days)", ceil_usize(remaining_time / 86400.))?,
            }

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
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceil() {
        assert_eq!(ceil_usize(0.0), 0);
        assert_eq!(ceil_usize(1.0), 1);
        assert_eq!(ceil_usize(1.2), 2);
        assert_eq!(ceil_usize(f64::INFINITY), usize::MAX);
    }

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
        let mut progress_bar = ProgressBar::new_with_eta(0.2, 100);
        progress_bar.set_eta_update_period(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(2));
        progress_bar.set_progress(50);
        progress_bar.print_final_info("Done", "first batch", Color::Green, Style::Bold);
        std::thread::sleep(Duration::from_millis(2));
        // used to underflow: the ETA still remembered progress 50
        progress_bar.set_progress(10);
    }

    #[test]
    fn no_color() {
        assert_eq!(Sgr(Color::Red, true).to_string(), "\x1B[31m");
        assert_eq!(Sgr(Color::Red, false).to_string(), "");
    }
}
