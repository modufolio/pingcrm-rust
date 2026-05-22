use crate::database::models::{Account, AuditLog, Contact, NewAccount, Organization, User};
use crate::database::pool::DbPool;
use crate::database::schema::{accounts, audit_logs, contacts, organizations, users};
use diesel_async::RunQueryDsl;

use super::audit_log_factory::AuditLogFactory;
use super::contact_factory::ContactFactory;
use super::factory_trait::Factory;
use super::organization_factory::OrganizationFactory;
use super::user_factory::UserFactory;

pub struct EntityFactory {
    pool: DbPool,
}

impl EntityFactory {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    pub async fn create_user(&self, factory: UserFactory) -> Result<User, diesel::result::Error> {
        let new_user = factory.build();

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(&mut conn)
            .await
    }

    pub async fn create_users(
        &self,
        factory: UserFactory,
        count: usize,
        customizer: Option<Box<dyn Fn(usize, UserFactory) -> UserFactory>>,
    ) -> Result<Vec<User>, diesel::result::Error> {
        let mut results = Vec::new();

        for i in 0..count {
            let current_factory = if let Some(ref customize) = customizer {
                customize(i, factory.clone())
            } else {
                factory.clone()
            };

            let user = self.create_user(current_factory).await?;
            results.push(user);
        }

        Ok(results)
    }

    pub async fn create_audit_log(
        &self,
        factory: AuditLogFactory,
    ) -> Result<AuditLog, diesel::result::Error> {
        let new_log = factory.build();

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(audit_logs::table)
            .values(&new_log)
            .get_result::<AuditLog>(&mut conn)
            .await
    }

    pub async fn create_audit_logs(
        &self,
        factory: AuditLogFactory,
        count: usize,
        customizer: Option<Box<dyn Fn(usize, AuditLogFactory) -> AuditLogFactory>>,
    ) -> Result<Vec<AuditLog>, diesel::result::Error> {
        let mut results = Vec::new();

        for i in 0..count {
            let current_factory = if let Some(ref customize) = customizer {
                customize(i, factory.clone())
            } else {
                factory.clone()
            };

            let log = self.create_audit_log(current_factory).await?;
            results.push(log);
        }

        Ok(results)
    }

    pub async fn create_account(
        &self,
        name: impl Into<String>,
    ) -> Result<Account, diesel::result::Error> {
        let new_account = NewAccount::new(name.into());

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .get_result::<Account>(&mut conn)
            .await
    }

    pub async fn create_organization(
        &self,
        factory: OrganizationFactory,
    ) -> Result<Organization, diesel::result::Error> {
        let new_org = factory.build();

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(organizations::table)
            .values(&new_org)
            .get_result::<Organization>(&mut conn)
            .await
    }

    pub async fn create_organizations(
        &self,
        factory: OrganizationFactory,
        count: usize,
        customizer: Option<Box<dyn Fn(usize, OrganizationFactory) -> OrganizationFactory>>,
    ) -> Result<Vec<Organization>, diesel::result::Error> {
        let mut results = Vec::new();

        for i in 0..count {
            let current_factory = if let Some(ref customize) = customizer {
                customize(i, factory.clone())
            } else {
                factory.clone()
            };

            let org = self.create_organization(current_factory).await?;
            results.push(org);
        }

        Ok(results)
    }

    pub async fn create_contact(
        &self,
        factory: ContactFactory,
    ) -> Result<Contact, diesel::result::Error> {
        let new_contact = factory.build();

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(contacts::table)
            .values(&new_contact)
            .get_result::<Contact>(&mut conn)
            .await
    }

    pub async fn create_contacts(
        &self,
        factory: ContactFactory,
        count: usize,
        customizer: Option<Box<dyn Fn(usize, ContactFactory) -> ContactFactory>>,
    ) -> Result<Vec<Contact>, diesel::result::Error> {
        let mut results = Vec::new();

        for i in 0..count {
            let current_factory = if let Some(ref customize) = customizer {
                customize(i, factory.clone())
            } else {
                factory.clone()
            };

            let contact = self.create_contact(current_factory).await?;
            results.push(contact);
        }

        Ok(results)
    }
}
