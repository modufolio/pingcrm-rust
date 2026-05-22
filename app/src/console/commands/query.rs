use crate::console::output::ConsoleOutput;
use anyhow::Result;
use clap::Args;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Debug, Args)]
pub struct QuerySqlCommand {
    pub sql: String,

    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long, short = 'v')]
    pub verbose: bool,
}

impl QuerySqlCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Execute SQL Query");

        let database_url = self.database_url.clone().unwrap_or_else(|| {
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "appkit.db".to_string())
        });

        ConsoleOutput::info(format!("Database: {}", database_url));
        ConsoleOutput::info(format!("Query: {}", self.sql));
        ConsoleOutput::newline();

        let sql_lower = self.sql.to_lowercase();
        let is_write = sql_lower.starts_with("insert")
            || sql_lower.starts_with("update")
            || sql_lower.starts_with("delete")
            || sql_lower.starts_with("drop")
            || sql_lower.starts_with("alter")
            || sql_lower.starts_with("create");

        if is_write {
            ConsoleOutput::warning("This query will modify the database");
            let confirmed = ConsoleOutput::confirm("Continue?", false);
            if !confirmed {
                ConsoleOutput::info("Query cancelled");
                return Ok(());
            }
        }

        ConsoleOutput::task("Executing query");

        let pool = crate::database::establish_connection_pool(&database_url).await?;
        let mut conn = pool.get().await?;

        let result = diesel::sql_query(&self.sql).execute(&mut conn).await;

        match result {
            Ok(affected_rows) => {
                ConsoleOutput::done("Query executed successfully");
                ConsoleOutput::newline();

                if is_write {
                    ConsoleOutput::success(format!("Affected rows: {}", affected_rows));
                } else {
                    ConsoleOutput::info(format!(
                        "Query completed ({} rows affected)",
                        affected_rows
                    ));
                }
            }
            Err(e) => {
                ConsoleOutput::error(format!("Query failed: {}", e));

                if self.verbose {
                    ConsoleOutput::newline();
                    ConsoleOutput::info("Error details:");
                    ConsoleOutput::list_item(format!("{:?}", e));
                }
            }
        }

        ConsoleOutput::newline();
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct QueryTablesCommand {
    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long, short = 'v')]
    pub verbose: bool,
}

impl QueryTablesCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Database Tables");

        let database_url = self.database_url.clone().unwrap_or_else(|| {
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "appkit.db".to_string())
        });

        ConsoleOutput::info(format!("Database: {}", database_url));
        ConsoleOutput::task("Fetching tables");

        let pool = crate::database::establish_connection_pool(&database_url).await?;
        let mut conn = pool.get().await?;

        #[derive(QueryableByName, Debug)]
        struct TableInfo {
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
        }

        let tables: Vec<TableInfo> = diesel::sql_query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )
        .load(&mut conn)
        .await?;

        ConsoleOutput::done(format!("Found {} tables", tables.len()));
        ConsoleOutput::newline();

        if tables.is_empty() {
            ConsoleOutput::warning("No tables found in database");
            ConsoleOutput::info("Run 'migrate' to create the schema");
            return Ok(());
        }

        if self.verbose {
            for table in tables {
                ConsoleOutput::info(format!("Table: {}", table.name));

                #[derive(QueryableByName, Debug)]
                struct ColumnInfo {
                    #[diesel(sql_type = diesel::sql_types::Text)]
                    name: String,
                    #[diesel(sql_type = diesel::sql_types::Text)]
                    r#type: String,
                    #[diesel(sql_type = diesel::sql_types::Integer)]
                    notnull: i32,
                    #[diesel(sql_type = diesel::sql_types::Integer)]
                    pk: i32,
                }

                let columns: Vec<ColumnInfo> =
                    diesel::sql_query(format!("PRAGMA table_info({})", table.name))
                        .load(&mut conn)
                        .await
                        .unwrap_or_default();

                ConsoleOutput::list_item(format!("Columns: {}", columns.len()));
                for col in columns {
                    let mut attrs = vec![];
                    if col.pk == 1 {
                        attrs.push("PRIMARY KEY");
                    }
                    if col.notnull == 1 {
                        attrs.push("NOT NULL");
                    }

                    let attr_str = if attrs.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", attrs.join(", "))
                    };

                    ConsoleOutput::list_item(format!(
                        " - {}: {}{}",
                        col.name, col.r#type, attr_str
                    ));
                }

                ConsoleOutput::newline();
            }
        } else {
            ConsoleOutput::info("Tables:");
            for table in tables {
                ConsoleOutput::list_item(&table.name);
            }
            ConsoleOutput::newline();
            ConsoleOutput::info("Use --verbose to see column details");
        }

        ConsoleOutput::newline();
        Ok(())
    }
}
