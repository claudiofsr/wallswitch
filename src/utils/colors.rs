use std::fmt::Display;

// https://gist.github.com/abritinthebay/d80eb99b2726c83feb0d97eab95206c4
// https://talyian.github.io/ansicolors/

// Ansi Colors & Styles:
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const BLACK: &str = "\x1b[30m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";

// ANSI Terminal Control Sequences
pub const CURSOR_TO_START: &str = "\r";
pub const ERASE_LINE_TO_END: &str = "\x1b[K";

pub trait Colors {
    // Styles
    fn bold(&self) -> String;
    fn dimmed(&self) -> String;

    // Colors
    fn black(&self) -> String;
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn blue(&self) -> String;
    fn magenta(&self) -> String;
    fn cyan(&self) -> String;
    fn white(&self) -> String;

    /// Wraps the text to be printed on a single line, clearing the previous content.
    /// Idiomatic way to handle progress updates.
    fn to_line_start(&self) -> String;
}

impl<T> Colors for T
where
    T: Display,
{
    fn bold(&self) -> String {
        format!("{BOLD}{self}{RESET}")
    }

    fn dimmed(&self) -> String {
        format!("{DIM}{self}{RESET}")
    }

    fn black(&self) -> String {
        format!("{BLACK}{self}{RESET}")
    }

    fn red(&self) -> String {
        format!("{RED}{self}{RESET}")
    }

    fn green(&self) -> String {
        format!("{GREEN}{self}{RESET}")
    }

    fn yellow(&self) -> String {
        format!("{YELLOW}{self}{RESET}")
    }

    fn blue(&self) -> String {
        format!("{BLUE}{self}{RESET}")
    }

    fn magenta(&self) -> String {
        format!("{MAGENTA}{self}{RESET}")
    }

    fn cyan(&self) -> String {
        format!("{CYAN}{self}{RESET}")
    }

    fn white(&self) -> String {
        format!("{WHITE}{self}{RESET}")
    }

    fn to_line_start(&self) -> String {
        format!("{CURSOR_TO_START}{self}{ERASE_LINE_TO_END}")
    }
}
