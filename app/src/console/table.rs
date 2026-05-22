use atty;

use colored::*;
use std::fmt::Display;

pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    use_colors: bool,
}

impl Table {
    pub fn new(headers: Vec<impl Into<String>>) -> Self {
        Self {
            headers: headers.into_iter().map(|h| h.into()).collect(),
            rows: Vec::new(),
            use_colors: atty::is(atty::Stream::Stdout),
        }
    }

    pub fn add_row(&mut self, columns: Vec<impl Display>) {
        let row: Vec<String> = columns.iter().map(|c| c.to_string()).collect();
        self.rows.push(row);
    }

    pub fn disable_colors(&mut self) {
        self.use_colors = false;
    }

    fn calculate_widths(&self) -> Vec<usize> {
        let mut widths: Vec<usize> = self.headers.iter().map(|h| h.len()).collect();

        for row in &self.rows {
            for (i, col) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(col.len());
                }
            }
        }

        widths
    }

    pub fn render(&self) {
        let widths = self.calculate_widths();

        self.print_header(&widths);

        self.print_separator(&widths);

        for row in &self.rows {
            self.print_row(row, &widths);
        }
    }

    fn print_header(&self, widths: &[usize]) {
        let mut header_line = String::new();
        for (i, header) in self.headers.iter().enumerate() {
            if i > 0 {
                header_line.push_str(" ");
            }
            header_line.push_str(&format!("{:<width$}", header, width = widths[i]));
        }

        if self.use_colors {
            println!("{}", header_line.bright_white().bold());
        } else {
            println!("{}", header_line);
        }
    }

    fn print_separator(&self, widths: &[usize]) {
        let mut separator_line = String::new();
        for (i, &width) in widths.iter().enumerate() {
            if i > 0 {
                separator_line.push_str(" ");
            }
            separator_line.push_str(&"-".repeat(width));
        }

        if self.use_colors {
            println!("{}", separator_line.bright_black());
        } else {
            println!("{}", separator_line);
        }
    }

    fn print_row(&self, row: &[String], widths: &[usize]) {
        let mut row_line = String::new();
        for (i, col) in row.iter().enumerate() {
            if i > 0 {
                row_line.push_str(" ");
            }
            let width = widths.get(i).copied().unwrap_or(20);
            row_line.push_str(&format!("{:<width$}", col, width = width));
        }
        println!("{}", row_line);
    }
}

impl Table {
    pub fn quick_render(headers: Vec<impl Into<String>>, rows: Vec<Vec<impl Display>>) {
        let mut table = Table::new(headers);
        for row in rows {
            table.add_row(row);
        }
        table.render();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let mut table = Table::new(vec!["ID", "Name", "Status"]);
        table.add_row(vec!["1", "User One", "Active"]);
        table.add_row(vec!["2", "User Two", "Inactive"]);
        table.disable_colors();
        table.render();
    }

    #[test]
    fn test_dynamic_widths() {
        let mut table = Table::new(vec!["Short", "Medium Length", "X"]);
        table.add_row(vec!["A very long value here", "Short", "Y"]);
        table.disable_colors();

        let widths = table.calculate_widths();
        assert_eq!(widths[0], "A very long value here".len());
        assert_eq!(widths[1], "Medium Length".len());
        assert_eq!(widths[2], 1);
    }
}
