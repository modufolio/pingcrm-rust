use crate::console::output::ConsoleOutput;
use anyhow::Result;
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct ServeCommand {
    #[arg(long, short = 'H', default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, short = 'p', default_value = "3000")]
    pub port: u16,

    #[arg(long, short = 'w')]
    pub watch: bool,

    #[arg(long, short = 'e', default_value = "dev")]
    pub env: String,
}

impl ServeCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Development Server");

        ConsoleOutput::info(format!(
            "Starting server at http://{}:{}",
            self.host, self.port
        ));
        ConsoleOutput::info(format!("Environment: {}", self.env));
        ConsoleOutput::newline();

        std::env::set_var("HOST", &self.host);
        std::env::set_var("PORT", self.port.to_string());
        std::env::set_var("APP_ENV", &self.env);

        if self.watch {
            ConsoleOutput::info("Auto-reload enabled (using cargo-watch)");
            ConsoleOutput::info("Press Ctrl+C to stop");
            ConsoleOutput::newline();

            let status = Command::new("cargo")
                .args(&["watch", "-x", "run --bin app-server"])
                .env("HOST", &self.host)
                .env("PORT", self.port.to_string())
                .env("APP_ENV", &self.env)
                .status();

            match status {
                Ok(_) => Ok(()),
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        ConsoleOutput::error("cargo-watch not found");
                        ConsoleOutput::info("Install with: cargo install cargo-watch");
                        ConsoleOutput::info("Or run without --watch flag");
                    } else {
                        ConsoleOutput::error(format!("Failed to start server: {}", e));
                    }
                    Ok(())
                }
            }
        } else {
            ConsoleOutput::info("Press Ctrl+C to stop");
            ConsoleOutput::newline();

            let status = Command::new("cargo")
                .args(&["run", "--bin", "app-server"])
                .env("HOST", &self.host)
                .env("PORT", self.port.to_string())
                .env("APP_ENV", &self.env)
                .status()?;

            if !status.success() {
                ConsoleOutput::error("Server exited with error");
            }

            Ok(())
        }
    }
}
