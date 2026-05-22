use crate::fixtures::Fixture;
use crate::seeder::{AuditLogFactory, EntityFactory};
use anyhow::Result;
use async_trait::async_trait;

pub struct AuditLogFixtures {
    pub create_activity_logs: bool,
    pub activity_log_count: usize,
}

impl AuditLogFixtures {
    pub fn new() -> Self {
        Self {
            create_activity_logs: true,
            activity_log_count: 50,
        }
    }

    pub fn with_activity_logs(mut self, count: usize) -> Self {
        self.create_activity_logs = true;
        self.activity_log_count = count;
        self
    }

    pub fn without_activity_logs(mut self) -> Self {
        self.create_activity_logs = false;
        self
    }
}

impl Default for AuditLogFixtures {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fixture for AuditLogFixtures {
    async fn load(&self, factory: &EntityFactory) -> Result<()> {
        println!("    Creating authentication logs...");

        for i in 0..10 {
            factory
                .create_audit_log(
                    AuditLogFactory::new()
                        .with_entity("Authentication".to_string())
                        .with_action("Login".to_string())
                        .with_status("Success".to_string())
                        .with_ip(format!("192.168.1.{}", 100 + i))
                        .with_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)".to_string()),
                )
                .await?;
        }
        println!("    ✓ Created 10 successful login logs");

        for i in 0..5 {
            factory
                .create_audit_log(
                    AuditLogFactory::new()
                        .with_entity("Authentication".to_string())
                        .with_action("Login".to_string())
                        .with_status("Failed".to_string())
                        .with_ip(format!("192.168.1.{}", 200 + i))
                        .with_details("Invalid credentials".to_string()),
                )
                .await?;
        }
        println!("    ✓ Created 5 failed login logs");

        println!("    Creating system event logs...");
        let system_events = vec![
            (
                "Configuration",
                "Update Settings",
                "Success",
                "Updated email settings",
            ),
            (
                "Configuration",
                "Update Settings",
                "Success",
                "Updated security settings",
            ),
            ("System", "Backup", "Success", "Database backup completed"),
            (
                "System",
                "Backup",
                "Failed",
                "Database backup failed: disk full",
            ),
            (
                "Security",
                "Password Reset",
                "Success",
                "Password reset email sent",
            ),
            (
                "Security",
                "2FA Enable",
                "Success",
                "Two-factor authentication enabled",
            ),
        ];

        for (event_type, action, status, details) in system_events {
            factory
                .create_audit_log(
                    AuditLogFactory::new()
                        .with_entity(event_type.to_string())
                        .with_action(action.to_string())
                        .with_status(status.to_string())
                        .with_details(details.to_string())
                        .with_ip("192.168.1.100".to_string()),
                )
                .await?;
        }
        println!("    ✓ Created {} system event logs", 6);

        if self.create_activity_logs {
            println!(
                "    Creating {} random activity logs...",
                self.activity_log_count
            );
            for i in 0..self.activity_log_count {
                let action = match i % 6 {
                    0 => "Create",
                    1 => "Update",
                    2 => "Delete",
                    3 => "View",
                    4 => "Export",
                    _ => "Import",
                };
                factory
                    .create_audit_log(
                        AuditLogFactory::new()
                            .with_entity("User Activity".to_string())
                            .with_action(action.to_string())
                            .with_status("Success".to_string()),
                    )
                    .await?;
            }
            println!(
                "    ✓ Created {} random activity logs",
                self.activity_log_count
            );
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "AuditLogFixtures"
    }

    fn description(&self) -> &str {
        "Create test audit logs (authentication, system events, activity)"
    }

    fn order(&self) -> i32 {
        20
    }
}
