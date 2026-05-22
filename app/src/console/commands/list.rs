use anyhow::Result;
use clap::Args;
use colored::*;

#[derive(Debug, Args)]
pub struct ListCommand {
    pub filter: Option<String>,

    #[arg(long)]
    pub raw: bool,
}

impl ListCommand {
    pub async fn execute(&self) -> Result<()> {
        let use_colors = !self.raw && atty::is(atty::Stream::Stdout);

        if use_colors {
            println!("{}", "AppKit Console".bright_green().bold());
            println!("{}", "==============".bright_green());
        } else {
            println!("AppKit Console");
            println!("==============");
        }
        println!();

        let command_groups = self.get_command_groups();

        let filtered_groups: Vec<_> = if let Some(filter) = &self.filter {
            command_groups
                .into_iter()
                .map(|(namespace, commands)| {
                    let filtered_commands: Vec<_> = commands
                        .into_iter()
                        .filter(|(name, _)| {
                            namespace.to_lowercase().contains(&filter.to_lowercase())
                                || name.to_lowercase().contains(&filter.to_lowercase())
                        })
                        .collect();
                    (namespace, filtered_commands)
                })
                .filter(|(_, commands)| !commands.is_empty())
                .collect()
        } else {
            command_groups
        };

        if filtered_groups.is_empty() {
            if use_colors {
                println!(
                    "{}",
                    format!(
                        "No commands found matching '{}'",
                        self.filter.as_ref().unwrap()
                    )
                    .yellow()
                );
            } else {
                println!(
                    "No commands found matching '{}'",
                    self.filter.as_ref().unwrap()
                );
            }
            return Ok(());
        }

        for (namespace, commands) in filtered_groups {
            if use_colors {
                println!("{}", namespace.yellow().bold());
            } else {
                println!("{}", namespace);
            }

            for (name, description) in commands {
                let full_name = if namespace == "Global" {
                    name.to_string()
                } else {
                    format!("{}:{}", namespace.to_lowercase(), name)
                };

                if use_colors {
                    println!(" {:<30} {}", full_name.green(), description.bright_black());
                } else {
                    println!(" {:<30} {}", full_name, description);
                }
            }
            println!();
        }

        Ok(())
    }

    fn get_command_groups(&self) -> Vec<(String, Vec<(String, String)>)> {
        vec![
            (
                "user".to_string(),
                vec![
                    (
                        "add".to_string(),
                        "Add a new user to the database".to_string(),
                    ),
                    (
                        "list".to_string(),
                        "List all users in the database".to_string(),
                    ),
                ],
            ),
            (
                "database".to_string(),
                vec![
                    (
                        "migrate".to_string(),
                        "Run pending database migrations (alias: m)".to_string(),
                    ),
                    (
                        "seed".to_string(),
                        "Seed the database with test data (alias: db:seed)".to_string(),
                    ),
                ],
            ),
            (
                "fixtures".to_string(),
                vec![(
                    "load".to_string(),
                    "Load data fixtures into the database".to_string(),
                )],
            ),
            (
                "schema".to_string(),
                vec![
                    (
                        "create".to_string(),
                        "Create the database schema".to_string(),
                    ),
                    (
                        "drop".to_string(),
                        "Drop the database schema (DESTRUCTIVE!)".to_string(),
                    ),
                    (
                        "validate".to_string(),
                        "Validate the database schema".to_string(),
                    ),
                ],
            ),
            (
                "migration".to_string(),
                vec![
                    (
                        "generate".to_string(),
                        "Generate a new migration file (alias: make:migration)".to_string(),
                    ),
                    (
                        "list".to_string(),
                        "List all available migrations (alias: migrations)".to_string(),
                    ),
                    ("status".to_string(), "Show migration status".to_string()),
                    (
                        "diff".to_string(),
                        "Generate migration from schema diff (planned)".to_string(),
                    ),
                ],
            ),
            (
                "query".to_string(),
                vec![
                    ("sql".to_string(), "Execute arbitrary SQL query".to_string()),
                    ("tables".to_string(), "Show database tables".to_string()),
                ],
            ),
            (
                "router".to_string(),
                vec![(
                    "debug".to_string(),
                    "Display all registered routes (alias: routes)".to_string(),
                )],
            ),
            (
                "global".to_string(),
                vec![
                    (
                        "serve".to_string(),
                        "Start the development server (alias: server)".to_string(),
                    ),
                    (
                        "db:info".to_string(),
                        "Display database information".to_string(),
                    ),
                    (
                        "list".to_string(),
                        "List all available commands".to_string(),
                    ),
                    ("help".to_string(), "Display help for a command".to_string()),
                ],
            ),
        ]
    }
}
