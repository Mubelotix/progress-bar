//! A module containing style and color enums.

use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Color {
    White,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    LightGray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "\x1B[97m"),
            Color::Black => write!(f, "\x1B[30m"),
            Color::Red => write!(f, "\x1B[31m"),
            Color::Green => write!(f, "\x1B[32m"),
            Color::Yellow => write!(f, "\x1B[33m"),
            Color::Blue => write!(f, "\x1B[34m"),
            Color::Magenta => write!(f, "\x1B[35m"),
            Color::Cyan => write!(f, "\x1B[36m"),
            Color::LightGray => write!(f, "\x1B[37m"),
            Color::DarkGray => write!(f, "\x1B[90m"),
            Color::LightRed => write!(f, "\x1B[91m"),
            Color::LightGreen => write!(f, "\x1B[92m"),
            Color::LightYellow => write!(f, "\x1B[93m"),
            Color::LightBlue => write!(f, "\x1B[94m"),
            Color::LightMagenta => write!(f, "\x1B[95m"),
            Color::LightCyan => write!(f, "\x1B[96m"),
        }
    }
}

pub enum Style {
    Normal,
    Bold,
    Dim,
    Italic,
    Underlined,
    Blink,
    Reverse,
    Hidden,
    StrikeThrough,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::Normal => write!(f, "\x1B[0m"),
            Style::Bold => write!(f, "\x1B[1m"),
            Style::Dim => write!(f, "\x1B[2m"),
            Style::Italic => write!(f, "\x1B[3m"),
            Style::Underlined => write!(f, "\x1B[4m"),
            Style::Blink => write!(f, "\x1B[5m"),
            Style::Reverse => write!(f, "\x1B[7m"),
            Style::Hidden => write!(f, "\x1B[8m"),
            Style::StrikeThrough => write!(f, "\x1B[9m"),
        }
    }
}

pub enum ProgressStyle {
    /// A number that shows how many actions have been finished and how many actions there are. Example: 23/57
    Number,
    /// A percentage of the completed work is shown. Example: 32%
    Percentage,
    /// No indication of progression is shown next to the progress bar.
    Empty,
}
