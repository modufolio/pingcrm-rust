use crate::database::models::*;
use crate::database::pool::DbPool;
use appkit_core::jsonapi::{PaginatedResult, QueryParams};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct AccountRepository {
    pool: DbPool,
}

impl AccountRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        account_id: i32,
    ) -> Result<Option<Account>, diesel::result::Error> {
        use crate::database::schema::accounts::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        accounts
            .filter(id.eq(account_id))
            .first::<Account>(&mut conn)
            .await
            .optional()
    }

    pub async fn create(&self, new_account: NewAccount) -> Result<Account, diesel::result::Error> {
        use crate::database::schema::accounts::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(accounts)
            .values(&new_account)
            .get_result::<Account>(&mut conn)
            .await
    }

    pub async fn find_with_params(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Account>, diesel::result::Error> {
        use crate::database::schema::accounts::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let count_query = accounts.into_boxed();
        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = accounts.into_boxed();
        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<Account>(&mut conn).await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    pub async fn count(&self) -> Result<i64, diesel::result::Error> {
        use crate::database::schema::accounts::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        accounts.count().get_result(&mut conn).await
    }

    pub async fn delete(&self, account_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::accounts::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(accounts.filter(id.eq(account_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        account_id: i32,
        account_update: AccountUpdate,
    ) -> Result<Account, diesel::result::Error> {
        use crate::database::schema::accounts::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(accounts.filter(id.eq(account_id)))
            .set(&account_update)
            .get_result::<Account>(&mut conn)
            .await
    }
}

impl From<DbPool> for AccountRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<Account> for AccountRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<Account>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Account>, diesel::result::Error> {
        self.find_with_params(params).await
    }

    async fn create(&self, new_item: NewAccount) -> Result<Account, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(
        &self,
        id: i32,
        update: AccountUpdate,
    ) -> Result<Account, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        _foreign_key: &str,
        _ids: Vec<i32>,
    ) -> Result<Vec<Account>, diesel::result::Error> {
        Ok(vec![])
    }
}
