use colored::*;

use std::fmt::Display;

pub struct ConsoleOutput;

impl ConsoleOutput {
    fn use_colors() -> bool {
        atty::is(atty::Stream::Stdout)
    }

    pub fn success(message: impl Display) {
        if Self::use_colors() {
            println!("\n{} {}\n", "".green(), message.to_string().green());
        } else {
            println!("\n {}\n", message);
        }
    }

    pub fn error(message: impl Display) {
        if Self::use_colors() {
            eprintln!(
                "\n{} {}\n",
                "".red(),
                format!("Error: {}", message).red().bold()
            );
        } else {
            eprintln!("\nError: {}\n", message);
        }
    }

    pub fn warning(message: impl Display) {
        if Self::use_colors() {
            println!(
                "\n{} {}\n",
                " ".yellow(),
                format!("Warning: {}", message).yellow()
            );
        } else {
            println!("\nWarning: {}\n", message);
        }
    }

    pub fn info(message: impl Display) {
        if Self::use_colors() {
            println!("\n{} {}\n", "ℹ ".cyan(), message.to_string().cyan());
        } else {
            println!("\nℹ {}\n", message);
        }
    }

    pub fn section(title: impl Display) {
        println!();
        if Self::use_colors() {
            println!("{}", title.to_string().yellow().bold());
            println!("{}", "=".repeat(50).yellow());
        } else {
            println!("{}", title);
            println!("{}", "=".repeat(50));
        }
    }

    pub fn task(message: impl Display) {
        if Self::use_colors() {
            println!("{} {}...", "".blue(), message.to_string().blue());
        } else {
            println!(" {}...", message);
        }
    }

    pub fn done(message: impl Display) {
        if Self::use_colors() {
            println!(" {} {}", "".green(), message.to_string().green());
        } else {
            println!(" {}", message);
        }
    }

    pub fn list_item(message: impl Display) {
        println!(" • {}", message);
    }

    pub fn table_header(headers: &[&str]) {
        let separator = headers
            .iter()
            .map(|h| "-".repeat(h.len().max(20)))
            .collect::<Vec<_>>()
            .join(" ");

        println!();
        if Self::use_colors() {
            let header_line = headers.join(" ");
            println!("{}", header_line.bright_white().bold());
            println!("{}", separator.bright_black());
        } else {
            println!("{}", headers.join(" "));
            println!("{}", separator);
        }
    }

    pub fn table_row(columns: &[impl Display]) {
        let row = columns
            .iter()
            .map(|c| format!("{:<20}", c))
            .collect::<Vec<_>>()
            .join(" ");
        println!("{}", row);
    }

    pub fn newline() {
        println!();
    }

    pub fn line() {
        if Self::use_colors() {
            println!("{}", "-".repeat(80).bright_black());
        } else {
            println!("{}", "-".repeat(80));
        }
    }

    pub fn confirm(question: impl Display, default: bool) -> bool {
        use std::io::{self, Write};

        let prompt = if default {
            format!("{} [Y/n]: ", question)
        } else {
            format!("{} [y/N]: ", question)
        };

        if Self::use_colors() {
            print!("{}", prompt.yellow());
        } else {
            print!("{}", prompt);
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim().to_lowercase();

        if input.is_empty() {
            return default;
        }

        matches!(input.as_str(), "y" | "yes")
    }

    pub fn ask(question: impl Display, default: Option<&str>) -> String {
        use std::io::{self, Write};

        let prompt = if let Some(def) = default {
            format!("{} [{}]: ", question, def)
        } else {
            format!("{}: ", question)
        };

        if Self::use_colors() {
            print!("{}", prompt.cyan());
        } else {
            print!("{}", prompt);
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();

        if input.is_empty() {
            default.unwrap_or("").to_string()
        } else {
            input.to_string()
        }
    }

    pub fn progress(current: usize, total: usize, message: impl Display) {
        let percentage = (current as f64 / total as f64 * 100.0) as usize;
        let bar_length = 40;
        let filled = (bar_length * current) / total;
        let empty = bar_length - filled;

        let progress_bar = if Self::use_colors() {
            format!(
                "\r[{}{}] {}% - {} ({}/{})",
                "=".repeat(filled).green(),
                " ".repeat(empty),
                percentage.to_string().cyan(),
                message,
                current,
                total
            )
        } else {
            format!(
                "\r[{}{}] {}% - {} ({}/{})",
                "=".repeat(filled),
                " ".repeat(empty),
                percentage,
                message,
                current,
                total
            )
        };

        print!("{}", progress_bar);

        use std::io::Write;
        std::io::stdout().flush().unwrap();

        if current == total {
            println!();
        }
    }

    pub fn comment(message: impl Display) {
        if Self::use_colors() {
            println!("{}", message.to_string().bright_black());
        } else {
            println!("{}", message);
        }
    }

    pub fn note(message: impl Display) {
        if Self::use_colors() {
            println!(
                "\n{} {}\n",
                "!".yellow().bold(),
                message.to_string().yellow()
            );
        } else {
            println!("\n! {}\n", message);
        }
    }

    pub fn caution(message: impl Display) {
        if Self::use_colors() {
            println!("\n{} {}\n", "!".red().bold(), message.to_string().red());
        } else {
            println!("\n! {}\n", message);
        }
    }
}
