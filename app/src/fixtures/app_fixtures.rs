use crate::database::models::{Contact, Organization, User};

use crate::fixtures::Fixture;
use crate::seeder::{
    AuditLogFactory, ContactFactory, EntityFactory, OrganizationFactory, UserFactory,
};
use anyhow::Result;
use async_trait::async_trait;

pub struct AppFixtures {
    pub load_users: bool,
    pub load_organizations: bool,
    pub load_contacts: bool,
    pub load_audit_logs: bool,
    pub random_user_count: usize,
    pub organization_count: usize,
    pub contact_count: usize,
    pub activity_log_count: usize,
}

impl AppFixtures {
    pub fn new() -> Self {
        Self {
            load_users: true,
            load_organizations: true,
            load_contacts: true,
            load_audit_logs: true,
            random_user_count: 20,
            organization_count: 10,
            contact_count: 30,
            activity_log_count: 50,
        }
    }

    pub fn users_only() -> Self {
        Self {
            load_users: true,
            load_organizations: false,
            load_contacts: false,
            load_audit_logs: false,
            random_user_count: 20,
            organization_count: 0,
            contact_count: 0,
            activity_log_count: 0,
        }
    }

    pub fn with_random_users(mut self, count: usize) -> Self {
        self.random_user_count = count;
        self
    }

    pub fn with_organizations(mut self, count: usize) -> Self {
        self.organization_count = count;
        self
    }

    pub fn with_contacts(mut self, count: usize) -> Self {
        self.contact_count = count;
        self
    }

    pub fn with_activity_logs(mut self, count: usize) -> Self {
        self.activity_log_count = count;
        self
    }

    async fn load_users(&self, factory: &EntityFactory) -> Result<Vec<User>> {
        let mut created_users = Vec::new();

        println!("Creating account...");

        let account = factory.create_account("Acme Corporation").await?;
        let account_id = account.id;
        println!("      ✓ Account: {} ({})", account.name, account.id);

        println!("Creating users...");

        let super_admin = factory
            .create_user(
                UserFactory::new()
                    .with_email("admin@example.com")
                    .with_name("Super Admin")
                    .with_password("admin123")
                    .with_role("ROLE_SUPER_ADMIN".to_string())
                    .with_status("active".to_string())
                    .with_account(account_id)
                    .with_owner(true),
            )
            .await?;
        println!(
            "      ✓ Super Admin: {} {} ({}) [owner]",
            super_admin.first_name, super_admin.last_name, super_admin.email
        );
        created_users.push(super_admin.clone());

        let admin1 = factory
            .create_user(
                UserFactory::new()
                    .with_email("manager@example.com")
                    .with_name("Sarah Manager")
                    .with_password("password123")
                    .with_role("ROLE_ADMIN".to_string())
                    .with_account(account_id),
            )
            .await?;
        println!(
            "Admin: {} {} ({})",
            admin1.first_name, admin1.last_name, admin1.email
        );
        created_users.push(admin1.clone());

        let admin2 = factory
            .create_user(
                UserFactory::new()
                    .with_email("admin.john@example.com")
                    .with_name("John Administrator")
                    .with_password("password123")
                    .with_role("ROLE_ADMIN".to_string())
                    .with_account(account_id),
            )
            .await?;
        println!(
            "      ✓ Admin: {} {} ({})",
            admin2.first_name, admin2.last_name, admin2.email
        );
        created_users.push(admin2);

        let named_users = vec![
            ("john.doe@example.com", "John Doe"),
            ("jane.smith@example.com", "Jane Smith"),
            ("bob.wilson@example.com", "Bob Wilson"),
            ("alice.johnson@example.com", "Alice Johnson"),
            ("charlie.brown@example.com", "Charlie Brown"),
        ];

        for (email, name) in named_users {
            let user = factory
                .create_user(
                    UserFactory::new()
                        .with_email(email)
                        .with_name(name)
                        .with_password("password123")
                        .with_role("ROLE_USER".to_string())
                        .with_account(account_id),
                )
                .await?;
            println!(
                "      ✓ User: {} {} ({})",
                user.first_name, user.last_name, user.email
            );
            created_users.push(user);
        }

        if self.random_user_count > 0 {
            println!("Creating {} random users...", self.random_user_count);
            for i in 0..self.random_user_count {
                let user_factory = if i % 5 == 0 {
                    UserFactory::new()
                        .with_role("ROLE_ADMIN".to_string())
                        .with_account(account_id)
                } else {
                    UserFactory::new().with_account(account_id)
                };
                let user = factory.create_user(user_factory).await?;
                created_users.push(user);
            }
            println!("      ✓ Created {} random users", self.random_user_count);
        }

        let _disabled_user = factory
            .create_user(
                UserFactory::new()
                    .with_email("disabled@example.com")
                    .with_name("Disabled User")
                    .with_password("password123")
                    .with_status("disabled".to_string())
                    .with_account(account_id),
            )
            .await?;
        println!("      ✓ Disabled User: disabled@example.com");

        let _locked_user = factory
            .create_user(
                UserFactory::new()
                    .with_email("locked@example.com")
                    .with_name("Locked User")
                    .with_password("password123")
                    .with_status("locked".to_string())
                    .with_account(account_id),
            )
            .await?;
        println!("✓ Locked User: locked@example.com");

        let _twofa_user = factory
            .create_user(
                UserFactory::new()
                    .with_email("secure@example.com")
                    .with_name("Secure User")
                    .with_password("password123")
                    .with_two_factor("JBSWY3DPEHPK3PXP".to_string())
                    .with_account(account_id),
            )
            .await?;
        println!("✓ 2FA User: secure@example.com");

        Ok(created_users)
    }

    async fn load_organizations(
        &self,
        factory: &EntityFactory,
        account_id: i32,
    ) -> Result<Vec<Organization>> {
        let mut created_organizations = Vec::new();

        println!("Creating organizations...");

        let named_orgs = vec![
            ("Acme Corporation", "info@acme.com", "+1-555-0100"),
            (
                "Tech Solutions Inc",
                "contact@techsolutions.com",
                "+1-555-0101",
            ),
            ("Global Industries", "hello@globalind.com", "+1-555-0102"),
            ("Innovation Labs", "info@innovationlabs.com", "+1-555-0103"),
            ("Enterprise Systems", "contact@entsys.com", "+1-555-0104"),
        ];

        for (name, email, phone) in &named_orgs {
            let org = factory
                .create_organization(
                    OrganizationFactory::new()
                        .with_name(*name)
                        .with_email(*email)
                        .with_phone(*phone)
                        .with_account(account_id),
                )
                .await?;
            println!(
                "✓ Organization: {} ({})",
                org.name,
                org.email.as_ref().unwrap_or(&"".to_string())
            );
            created_organizations.push(org);
        }

        if self.organization_count > named_orgs.len() {
            let random_count = self.organization_count - named_orgs.len();
            println!("Creating {} random organizations...", random_count);
            for _ in 0..random_count {
                let org = factory
                    .create_organization(OrganizationFactory::new().with_account(account_id))
                    .await?;
                created_organizations.push(org);
            }
            println!("      ✓ Created {} random organizations", random_count);
        }

        Ok(created_organizations)
    }

    async fn load_contacts(
        &self,
        factory: &EntityFactory,
        account_id: i32,
        organizations: &[Organization],
    ) -> Result<Vec<Contact>> {
        let mut created_contacts = Vec::new();

        println!("Creating contacts...");

        let named_contacts = vec![
            ("John", "Doe", "john.doe@example.com", "+1-555-1001"),
            ("Jane", "Smith", "jane.smith@example.com", "+1-555-1002"),
            ("Bob", "Johnson", "bob.johnson@example.com", "+1-555-1003"),
            (
                "Alice",
                "Williams",
                "alice.williams@example.com",
                "+1-555-1004",
            ),
            (
                "Charlie",
                "Brown",
                "charlie.brown@example.com",
                "+1-555-1005",
            ),
        ];

        for (i, (first_name, last_name, email, phone)) in named_contacts.iter().enumerate() {
            let org_id = if !organizations.is_empty() {
                Some(organizations[i % organizations.len()].id)
            } else {
                None
            };

            let contact = factory
                .create_contact(
                    ContactFactory::new()
                        .with_first_name(*first_name)
                        .with_last_name(*last_name)
                        .with_email(*email)
                        .with_phone(*phone)
                        .with_account(account_id)
                        .with_organization(org_id.unwrap_or(0)),
                )
                .await?;

            let org_name = org_id
                .and_then(|id| organizations.iter().find(|o| o.id == id))
                .map(|o| o.name.as_str())
                .unwrap_or("No org");

            println!(
                "Contact: {} {} ({}) - {}",
                contact.first_name,
                contact.last_name,
                contact.email.as_ref().unwrap_or(&"".to_string()),
                org_name
            );
            created_contacts.push(contact);
        }

        if self.contact_count > named_contacts.len() {
            let random_count = self.contact_count - named_contacts.len();
            println!("Creating {} random contacts...", random_count);
            for i in 0..random_count {
                let org_id = if !organizations.is_empty() {
                    Some(organizations[i % organizations.len()].id)
                } else {
                    None
                };

                let mut contact_factory = ContactFactory::new().with_account(account_id);

                if let Some(oid) = org_id {
                    contact_factory = contact_factory.with_organization(oid);
                }

                let contact = factory.create_contact(contact_factory).await?;
                created_contacts.push(contact);
            }
            println!("      ✓ Created {} random contacts", random_count);
        }

        Ok(created_contacts)
    }

    async fn load_audit_logs(&self, factory: &EntityFactory, users: &[User]) -> Result<()> {
        println!("Creating audit logs...");

        println!("Creating authentication logs...");
        for (i, user) in users.iter().take(10).enumerate() {
            factory
                .create_audit_log(
                    AuditLogFactory::new()
                        .with_entity("Authentication".to_string())
                        .with_user(user.id, user.email.clone())
                        .with_action("Login".to_string())
                        .with_status("Success".to_string())
                        .with_ip(format!("192.168.1.{}", 100 + i))
                        .with_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)".to_string()),
                )
                .await?;
        }
        println!("✓ Created 10 successful login logs");

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
        println!("      ✓ Created 5 failed login logs");

        if users.len() >= 3 {
            println!("Creating user management logs...");
            let super_admin = &users[0];
            let admin1 = &users[1];

            for (i, user) in users.iter().take(5).enumerate() {
                factory
                    .create_audit_log(
                        AuditLogFactory::new()
                            .with_entity("User Management".to_string())
                            .with_user(super_admin.id, super_admin.email.clone())
                            .with_action("Create User".to_string())
                            .with_status("Success".to_string())
                            .with_resource(format!("User:{}", user.id)),
                    )
                    .await?;

                if i < 3 {
                    factory
                        .create_audit_log(
                            AuditLogFactory::new()
                                .with_entity("User Management".to_string())
                                .with_user(admin1.id, admin1.email.clone())
                                .with_action("Update User".to_string())
                                .with_status("Success".to_string())
                                .with_resource(format!("User:{}", user.id)),
                        )
                        .await?;
                }
            }
            println!("✓ Created user management logs");
        }

        println!("Creating system event logs...");
        let system_events = vec![
            ("Configuration", "Update Settings", "Success"),
            ("System", "Backup", "Success"),
            ("Security", "Password Reset", "Success"),
            ("Security", "2FA Enable", "Success"),
        ];

        for (event_type, action, status) in system_events {
            factory
                .create_audit_log(
                    AuditLogFactory::new()
                        .with_entity(event_type.to_string())
                        .with_action(action.to_string())
                        .with_status(status.to_string()),
                )
                .await?;
        }
        println!("✓ Created system event logs");

        if self.activity_log_count > 0 {
            println!(
                "Creating {} random activity logs...",
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
                "      ✓ Created {} random activity logs",
                self.activity_log_count
            );
        }

        Ok(())
    }
}

impl Default for AppFixtures {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fixture for AppFixtures {
    async fn load(&self, factory: &EntityFactory) -> Result<()> {
        let mut created_users = Vec::new();
        let mut _created_organizations = Vec::new();
        let account_id = 1;

        if self.load_users {
            created_users = self.load_users(factory).await?;
        }

        if self.load_organizations {
            _created_organizations = self.load_organizations(factory, account_id).await?;
        }

        if self.load_contacts {
            let _created_contacts = self
                .load_contacts(factory, account_id, &_created_organizations)
                .await?;
        }

        if self.load_audit_logs {
            self.load_audit_logs(factory, &created_users).await?;
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "AppFixtures"
    }

    fn description(&self) -> &str {
        "Load all application fixtures (users, organizations, contacts, audit logs)"
    }

    fn order(&self) -> i32 {
        0
    }
}
