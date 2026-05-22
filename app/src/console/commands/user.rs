use crate::console::output::ConsoleOutput;
use crate::database::models::user::{NewUser, User};
use crate::database::pool::DbPool;
use crate::database::repositories::user_repository::UserRepository;
use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct UserAddCommand {
    #[arg(long, short = 'e')]
    pub email: Option<String>,

    #[arg(long, short = 'n')]
    pub name: Option<String>,

    #[arg(long, short = 'p')]
    pub password: Option<String>,

    #[arg(long, short = 'r', default_value = "user")]
    pub role: String,

    #[arg(long)]
    pub account_id: Option<i32>,
}

impl UserAddCommand {
    pub async fn execute(&self, pool: DbPool) -> Result<()> {
        ConsoleOutput::section("Add New User");

        let email = self
            .email
            .clone()
            .unwrap_or_else(|| ConsoleOutput::ask("Email address", None));

        let name = self
            .name
            .clone()
            .unwrap_or_else(|| ConsoleOutput::ask("Full name", None));

        let parts: Vec<&str> = name.split_whitespace().collect();
        let first_name = parts.first().unwrap_or(&"User").to_string();
        let last_name = parts
            .get(1..)
            .map(|parts| parts.join(" "))
            .unwrap_or_else(|| "".to_string());

        let password = self
            .password
            .clone()
            .unwrap_or_else(|| ConsoleOutput::ask("Password", None));

        let role = &self.role;

        if !email.contains('@') {
            ConsoleOutput::error("Invalid email format");
            return Ok(());
        }

        ConsoleOutput::task("Hashing password");
        let password_hash = appkit_core::security::hash_password(&password)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
        ConsoleOutput::done("Password hashed");

        let repo = UserRepository::new(pool);

        if let Some(_existing) = repo.find_by_email(&email).await? {
            ConsoleOutput::error(format!("User with email '{}' already exists", email));
            return Ok(());
        }

        ConsoleOutput::task("Creating user");

        let new_user = NewUser::new(
            email.clone(),
            password_hash,
            first_name.clone(),
            last_name.clone(),
        )
        .with_roles(vec![format!("ROLE_{}", role.to_uppercase())])
        .with_account(self.account_id.unwrap_or(1), true);

        let user = repo.create(new_user).await?;

        ConsoleOutput::done("User created");
        ConsoleOutput::newline();

        ConsoleOutput::info("User created successfully!");
        ConsoleOutput::list_item(format!("ID: {}", user.id));
        ConsoleOutput::list_item(format!("Email: {}", user.email));
        ConsoleOutput::list_item(format!("Name: {}", user.full_name()));
        ConsoleOutput::list_item(format!("Role: {}", role));
        ConsoleOutput::newline();

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct UserListCommand {
    #[arg(long)]
    pub account_id: Option<i32>,

    #[arg(long, short = 'l', default_value = "50")]
    pub limit: usize,

    #[arg(long, short = 'v')]
    pub verbose: bool,
}

impl UserListCommand {
    pub async fn execute(&self, pool: DbPool) -> Result<()> {
        ConsoleOutput::section("User List");

        let repo = UserRepository::new(pool);

        ConsoleOutput::task("Fetching users");

        let all_users = repo.find_all().await?;

        let filtered_users: Vec<User> = all_users
            .into_iter()
            .filter(|u| {
                if let Some(account_id) = self.account_id {
                    u.account_id == Some(account_id)
                } else {
                    true
                }
            })
            .take(self.limit)
            .collect();

        ConsoleOutput::done(format!("Found {} users", filtered_users.len()));
        ConsoleOutput::newline();

        if filtered_users.is_empty() {
            ConsoleOutput::warning("No users found");
            return Ok(());
        }

        if self.verbose {
            let mut table =
                crate::console::Table::new(vec!["ID", "Email", "Name", "Account ID", "Created"]);
            for user in filtered_users {
                table.add_row(vec![
                    user.id.to_string(),
                    user.email.clone(),
                    user.full_name(),
                    user.account_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    user.created_at.to_string(),
                ]);
            }
            table.render();
        } else {
            let mut table = crate::console::Table::new(vec!["ID", "Email", "Name"]);
            for user in filtered_users {
                table.add_row(vec![
                    user.id.to_string(),
                    user.email.clone(),
                    user.full_name(),
                ]);
            }
            table.render();
        }

        ConsoleOutput::newline();
        Ok(())
    }
}
