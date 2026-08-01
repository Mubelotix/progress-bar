use crate::style::*;
use std::io;
use std::io::Write;
use std::time::Instant;

pub struct EtaData {
    smoothing_factor: f64,
    last_progress: usize,
    last_time: Instant,
    speed: f64,
    color: Option<Color>,
    style: Option<Style>,
}

pub struct ProgressBar {
    max: usize,
    progress: usize,
    width: usize,
    action: String,
    action_color: Color,
    action_style: Style,
    progress_style: ProgressStyle,
    action_width: usize,
    eta: Option<EtaData>,
}

impl ProgressBar {
    /// Creates a progress bar with the total number of actions.  
    /// You will need to call inc method when an action is completed and the bar progress will be incremented by 1.  
    /// Don't print things with print macro while the bar is running; use the print_info method instead.  
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
            action: String::new(),
            action_color: Color::Black,
            action_style: Style::Normal,
            progress_style: ProgressStyle::Number,
            action_width: 12,
            eta: None,
        }
    }

    /// Same as [ProgressBar::new] but enabled ETA display.
    /// 
    /// Smoothing factor: A value between `0.1` and `0.3` works well.
    /// Higher value makes the estimate react faster to recent changes; lower makes it less jumpy.
    pub fn new_with_eta(smoothing_factor: f64, max: usize) -> Self {
        ProgressBar {
            eta: Some(EtaData {
                smoothing_factor,
                last_progress: 0,
                last_time: Instant::now(),
                speed: 0.0,
                color: None,
                style: None,
            }),
            ..ProgressBar::new(max)
        }
    }

    fn set_good_size(&self, text: &str) -> String {
        let text_len = text.len();
        if text_len == self.action_width {
            text.to_string()
        } else if text_len > self.action_width {
            text[..self.action_width].to_string()
        } else {
            let mut text = text.to_string();
            while text.len() < self.action_width {
                text.insert(0, ' ');
            }
            text
        }
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
                if delta_time >= 1.0 {
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
        self.eta = Some(EtaData {
            smoothing_factor,
            last_progress: 0,
            last_time: Instant::now(),
            speed: 0.0,
            color: None,
            style: None,
        });
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

    /// Set the global action displayed before the progress bar.
    pub fn set_action(&mut self, a: &str, c: Color, s: Style) {
        self.action = self.set_good_size(a);
        self.action_color = c;
        self.action_style = s;
        self.display();
    }

    /// Log something, without display update
    pub fn print_final_info(&mut self, info_name: &str, text: &str, info_color: Color, info_style: Style) {
        let info_name = self.set_good_size(info_name);
        println!("{}{}{}\x1B[0m {}\x1B[K", info_style, info_color, info_name, text);
        self.progress = 0;
    }

    /// Log something
    pub fn print_info(&mut self, info_name: &str, text: &str, info_color: Color, info_style: Style) {
        let info_name = self.set_good_size(info_name);
        println!("{}{}{}\x1B[0m {}\x1B[K", info_style, info_color, info_name, text);
        self.display();
    }

    /// Display the bar
    pub fn display(&self) {
        print!("{}{}{}\x1B[0m\x1B[K", self.action_style, self.action_color, self.action);

        print!(" [");
        for i in 0..self.width {
            if i*self.max/self.width < self.progress {
                if (i+1)*self.max/self.width >= self.progress {
                    print!(">");
                } else {
                    print!("=");
                }
            } else {
                print!(" ");
            }
        }

        print!("] ");
        match self.progress_style {
            ProgressStyle::Number => print!("{}/{}", self.progress, self.max),
            ProgressStyle::Percentage => if self.max != 0 {
                    print!("{}%", (self.progress as f64 / self.max as f64 * 100.0) as usize);
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

            #[allow(overlapping_range_endpoints)]
            #[allow(clippy::match_overlapping_arm)]
            let eta_str = match remaining_ms {
                0..=3_000 => format!("{}ms", remaining_ms),
                3_001..=SECS_110 => format!("{}s", remaining_time.ceil() as usize),
                SECS_110..=MINS_110 => format!("{} minutes", (remaining_time / 60.).ceil() as usize),
                MINS_110..=HOURS_46 => format!("{} hours", (remaining_time / 3600.).ceil() as usize),
                _ => format!("{} days", (remaining_time / 86400.).ceil() as usize),
            };

            let color_code = eta.color.map(|c| c.to_string()).unwrap_or_default();
            let style_code = eta.style.map(|s| s.to_string()).unwrap_or_default();

            if eta.color.is_some() || eta.style.is_some() {
                print!(" {color_code}{style_code}(ETA {eta_str})\x1B[0m");
            } else {
                print!(" (ETA {eta_str})");
            }
        }
        print!("\n\x1B[1A");

        #[allow(unused_must_use)]
        { io::stdout().flush(); }
    }
    
    /// Mark the end of the progress bar - updates will make a 'new' bar
    pub fn finalize(&mut self) {
        self.progress = 0;
        self.eta = None;
        println!();
    }
}
