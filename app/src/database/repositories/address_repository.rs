use crate::database::models::*;
use crate::database::pool::DbPool;
use appkit_core::jsonapi::{PaginatedResult, QueryParams};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct AddressRepository {
    pool: DbPool,
}

impl AddressRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        address_id: i32,
    ) -> Result<Option<Address>, diesel::result::Error> {
        use crate::database::schema::addresses::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        addresses
            .filter(id.eq(address_id))
            .first::<Address>(&mut conn)
            .await
            .optional()
    }

    pub async fn find_by_customer(
        &self,
        cust_id: i32,
    ) -> Result<Vec<Address>, diesel::result::Error> {
        use crate::database::schema::addresses::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        addresses
            .filter(customer_id.eq(cust_id))
            .load::<Address>(&mut conn)
            .await
    }

    pub async fn find_by_account(
        &self,
        acct_id: i32,
    ) -> Result<Vec<Address>, diesel::result::Error> {
        use crate::database::schema::addresses::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        addresses
            .filter(account_id.eq(acct_id))
            .load::<Address>(&mut conn)
            .await
    }

    pub async fn create(&self, new_address: NewAddress) -> Result<Address, diesel::result::Error> {
        use crate::database::schema::addresses::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(addresses)
            .values(&new_address)
            .get_result::<Address>(&mut conn)
            .await
    }

    pub async fn delete(&self, address_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::addresses::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(addresses.filter(id.eq(address_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        address_id: i32,
        address_update: AddressUpdate,
    ) -> Result<Address, diesel::result::Error> {
        use crate::database::schema::addresses::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(addresses.filter(id.eq(address_id)))
            .set(&address_update)
            .get_result::<Address>(&mut conn)
            .await
    }

    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Address>, diesel::result::Error> {
        use crate::database::schema::addresses::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let total = addresses.count().get_result::<i64>(&mut conn).await?;

        let offset = (params.page.number - 1) * params.page.size;
        let items = addresses
            .order(created_at.desc())
            .limit(params.page.size)
            .offset(offset)
            .load::<Address>(&mut conn)
            .await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }
}

impl From<DbPool> for AddressRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<Address> for AddressRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<Address>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Address>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(&self, new_item: NewAddress) -> Result<Address, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(
        &self,
        id: i32,
        update: AddressUpdate,
    ) -> Result<Address, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<Address>, diesel::result::Error> {
        use crate::database::schema::addresses;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "customer_id" => {
                addresses::table
                    .filter(addresses::customer_id.eq_any(&ids))
                    .load::<Address>(&mut conn)
                    .await
            }
            "account_id" => {
                addresses::table
                    .filter(addresses::account_id.eq_any(&ids))
                    .load::<Address>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
