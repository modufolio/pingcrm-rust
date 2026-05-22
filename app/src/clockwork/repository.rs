use crate::database::models::{
    ClockworkQuery, ClockworkRequest, NewClockworkQuery, NewClockworkRequest,
};
use crate::database::pool::DbPool;
use crate::database::schema::{clockwork_queries, clockwork_requests};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub struct ClockworkRepository {
    pool: DbPool,
}

impl ClockworkRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_request(
        &self,
        new_request: NewClockworkRequest,
    ) -> Result<ClockworkRequest, diesel::result::Error> {
        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(clockwork_requests::table)
            .values(&new_request)
            .get_result::<ClockworkRequest>(&mut conn)
            .await
    }

    pub async fn create_query(
        &self,
        new_query: NewClockworkQuery,
    ) -> Result<ClockworkQuery, diesel::result::Error> {
        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(clockwork_queries::table)
            .values(&new_query)
            .get_result::<ClockworkQuery>(&mut conn)
            .await
    }

    pub async fn find_request_by_id(
        &self,
        request_id: &str,
    ) -> Result<Option<ClockworkRequest>, diesel::result::Error> {
        use crate::database::schema::clockwork_requests::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        clockwork_requests
            .filter(id.eq(request_id))
            .first::<ClockworkRequest>(&mut conn)
            .await
            .optional()
    }

    pub async fn get_queries_for_request(
        &self,
        request_id_param: &str,
    ) -> Result<Vec<ClockworkQuery>, diesel::result::Error> {
        use crate::database::schema::clockwork_queries::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        clockwork_queries
            .filter(request_id.eq(request_id_param))
            .order(id.asc())
            .load::<ClockworkQuery>(&mut conn)
            .await
    }

    pub async fn get_latest_request(
        &self,
    ) -> Result<Option<ClockworkRequest>, diesel::result::Error> {
        use crate::database::schema::clockwork_requests::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        clockwork_requests
            .order(time.desc())
            .first::<ClockworkRequest>(&mut conn)
            .await
            .optional()
    }

    pub async fn get_last_request_ids(
        &self,
        limit: i64,
    ) -> Result<Vec<String>, diesel::result::Error> {
        use crate::database::schema::clockwork_requests::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        clockwork_requests
            .select(id)
            .order(time.desc())
            .limit(limit)
            .load::<String>(&mut conn)
            .await
    }

    pub async fn get_next_requests(
        &self,
        after_id: &str,
        limit: i64,
    ) -> Result<Vec<ClockworkRequest>, diesel::result::Error> {
        use crate::database::schema::clockwork_requests::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let ref_request = clockwork_requests
            .filter(id.eq(after_id))
            .first::<ClockworkRequest>(&mut conn)
            .await
            .optional()?;

        let ref_time = match ref_request {
            Some(r) => r.time,
            None => return Ok(Vec::new()),
        };

        clockwork_requests
            .filter(time.gt(ref_time))
            .order(time.asc())
            .limit(limit)
            .load::<ClockworkRequest>(&mut conn)
            .await
    }

    pub async fn get_previous_requests(
        &self,
        before_id: &str,
        limit: i64,
    ) -> Result<Vec<ClockworkRequest>, diesel::result::Error> {
        use crate::database::schema::clockwork_requests::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let ref_request = clockwork_requests
            .filter(id.eq(before_id))
            .first::<ClockworkRequest>(&mut conn)
            .await
            .optional()?;

        let ref_time = match ref_request {
            Some(r) => r.time,
            None => return Ok(Vec::new()),
        };

        clockwork_requests
            .filter(time.lt(ref_time))
            .order(time.desc())
            .limit(limit)
            .load::<ClockworkRequest>(&mut conn)
            .await
    }

    pub async fn cleanup_old_requests(
        &self,
        keep_last: i64,
    ) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::clockwork_requests::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let total_count: i64 = clockwork_requests.count().get_result(&mut conn).await?;

        if total_count <= keep_last {
            return Ok(0);
        }

        let ids_to_keep: Vec<String> = clockwork_requests
            .select(id)
            .order(time.desc())
            .limit(keep_last)
            .load(&mut conn)
            .await?;

        let deleted_queries = diesel::delete(
            clockwork_queries::table.filter(clockwork_queries::request_id.ne_all(&ids_to_keep)),
        )
        .execute(&mut conn)
        .await?;

        let deleted_requests = diesel::delete(clockwork_requests.filter(id.ne_all(&ids_to_keep)))
            .execute(&mut conn)
            .await?;

        tracing::debug!(
            "Cleaned up {} old requests and {} queries",
            deleted_requests,
            deleted_queries
        );
        Ok(deleted_requests)
    }

    pub async fn count(&self) -> Result<i64, diesel::result::Error> {
        use crate::database::schema::clockwork_requests::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        clockwork_requests.count().get_result(&mut conn).await
    }
}
