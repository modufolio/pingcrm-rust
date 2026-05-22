use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::models::{ImageJob, NewImageJob, UpdateImageJob};
use crate::database::pool::DbPool;
use crate::database::schema::image_jobs;

pub struct ImageJobRepository {
    pool: DbPool,
}

impl ImageJobRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_filename(
        &self,
        filename: &str,
        disk: &str,
    ) -> Result<Option<ImageJob>, diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();

        image_jobs::table
            .filter(image_jobs::filename.eq(filename))
            .filter(image_jobs::disk.eq(disk))
            .first::<ImageJob>(&mut conn)
            .await
            .optional()
    }

    pub async fn create(&self, new_job: NewImageJob) -> Result<ImageJob, diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();

        diesel::insert_into(image_jobs::table)
            .values(&new_job)
            .returning(ImageJob::as_returning())
            .get_result(&mut conn)
            .await
    }

    pub async fn mark_processed(&self, id: i32) -> Result<(), diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();

        let now = Utc::now().naive_utc();
        let update = UpdateImageJob {
            status: Some("processed".to_string()),
            processed_at: Some(now),
            accessed_at: Some(now),
            access_count: None,
            error_message: None,
            updated_at: now,
        };

        diesel::update(image_jobs::table.find(id))
            .set(&update)
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn mark_accessed(&self, id: i32) -> Result<(), diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();

        let job: ImageJob = image_jobs::table.find(id).first(&mut conn).await?;

        let now = Utc::now().naive_utc();
        let update = UpdateImageJob {
            status: None,
            processed_at: None,
            accessed_at: Some(now),
            access_count: Some(job.access_count + 1),
            error_message: None,
            updated_at: now,
        };

        diesel::update(image_jobs::table.find(id))
            .set(&update)
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn mark_failed(&self, id: i32, error: &str) -> Result<(), diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();

        let update = UpdateImageJob {
            status: Some("failed".to_string()),
            processed_at: None,
            accessed_at: None,
            access_count: None,
            error_message: Some(error.to_string()),
            updated_at: Utc::now().naive_utc(),
        };

        diesel::update(image_jobs::table.find(id))
            .set(&update)
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn find_pending(&self, limit: i64) -> Result<Vec<ImageJob>, diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();

        image_jobs::table
            .filter(image_jobs::status.eq("pending"))
            .order(image_jobs::created_at.asc())
            .limit(limit)
            .load::<ImageJob>(&mut conn)
            .await
    }

    pub async fn delete_old_accessed(&self, days: i64) -> Result<usize, diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();
        let cutoff = (Utc::now() - chrono::Duration::days(days)).naive_utc();

        diesel::delete(image_jobs::table)
            .filter(image_jobs::accessed_at.lt(cutoff))
            .filter(image_jobs::status.eq("processed"))
            .execute(&mut conn)
            .await
    }
}

impl From<DbPool> for ImageJobRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}
