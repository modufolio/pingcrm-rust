use crate::database::models::*;
use crate::database::pool::DbPool;
use crate::database::schema::media;
use appkit_core::jsonapi::{
    FilterCondition, FilterOperator, PaginatedResult, QueryParams, SortDirection,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct MediaRepository {
    pool: DbPool,
}

impl MediaRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        media_id: i32,
    ) -> Result<Option<MediaModel>, diesel::result::Error> {
        use crate::database::schema::media::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        media
            .filter(id.eq(media_id))
            .first::<MediaModel>(&mut conn)
            .await
            .optional()
    }

    pub async fn find_by_filename(
        &self,
        media_filename: &str,
    ) -> Result<Option<MediaModel>, diesel::result::Error> {
        use crate::database::schema::media::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        media
            .filter(filename.eq(media_filename))
            .first::<MediaModel>(&mut conn)
            .await
            .optional()
    }

    pub async fn create(&self, new_media: NewMedia) -> Result<MediaModel, diesel::result::Error> {
        use crate::database::schema::media::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(media)
            .values(&new_media)
            .get_result::<MediaModel>(&mut conn)
            .await
    }

    pub async fn delete(&self, media_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::media::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(media.filter(id.eq(media_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        media_id: i32,
        media_update: MediaUpdate,
    ) -> Result<MediaModel, diesel::result::Error> {
        use crate::database::schema::media::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(media.filter(id.eq(media_id)))
            .set(&media_update)
            .get_result::<MediaModel>(&mut conn)
            .await
    }

    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<MediaModel>, diesel::result::Error> {
        self.find_with_params(params).await
    }

    pub async fn find_with_params(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<MediaModel>, diesel::result::Error> {
        use crate::database::schema::media::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let mut count_query = media.into_boxed();
        for filter in &params.filters {
            count_query = Self::apply_media_filter(count_query, filter);
        }

        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = media.into_boxed();
        for filter in &params.filters {
            query = Self::apply_media_filter(query, filter);
        }

        query = Self::apply_media_sort(query, &params.sort);

        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<MediaModel>(&mut conn).await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    fn apply_media_filter(
        query: media::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        filter: &FilterCondition,
    ) -> media::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::media::dsl::*;

        match filter.field.as_str() {
            "filename" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(filename.eq(value)),
                        FilterOperator::Neq => query.filter(filename.ne(value)),
                        FilterOperator::Like => query.filter(filename.like(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "mime_type" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(mime_type.eq(value)),
                        FilterOperator::Neq => query.filter(mime_type.ne(value)),
                        FilterOperator::Like => query.filter(mime_type.like(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "is_public" => {
                if let Some(ref value) = filter.value {
                    if let Ok(bool_value) = value.parse::<bool>() {
                        match filter.operator {
                            FilterOperator::Eq => query.filter(is_public.eq(bool_value)),
                            FilterOperator::Neq => query.filter(is_public.ne(bool_value)),
                            _ => query,
                        }
                    } else {
                        query
                    }
                } else {
                    query
                }
            }
            "uploaded_by" => {
                if let Some(ref value) = filter.value {
                    if let Ok(id_value) = value.parse::<i32>() {
                        match filter.operator {
                            FilterOperator::Eq => query.filter(uploaded_by.eq(id_value)),
                            FilterOperator::Neq => query.filter(uploaded_by.ne(id_value)),
                            _ => query,
                        }
                    } else {
                        query
                    }
                } else {
                    match filter.operator {
                        FilterOperator::Null => query.filter(uploaded_by.is_null()),
                        FilterOperator::NotNull => query.filter(uploaded_by.is_not_null()),
                        _ => query,
                    }
                }
            }
            _ => query,
        }
    }

    fn apply_media_sort(
        query: media::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        sorts: &[(String, SortDirection)],
    ) -> media::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::media::dsl::*;

        let mut sorted_query = query;

        for (field, direction) in sorts {
            sorted_query = match (field.as_str(), direction) {
                ("filename", SortDirection::Ascending) => {
                    sorted_query.then_order_by(filename.asc())
                }
                ("filename", SortDirection::Descending) => {
                    sorted_query.then_order_by(filename.desc())
                }
                ("file_size", SortDirection::Ascending) => {
                    sorted_query.then_order_by(file_size.asc())
                }
                ("file_size", SortDirection::Descending) => {
                    sorted_query.then_order_by(file_size.desc())
                }
                ("created_at", SortDirection::Ascending) => {
                    sorted_query.then_order_by(created_at.asc())
                }
                ("created_at", SortDirection::Descending) => {
                    sorted_query.then_order_by(created_at.desc())
                }
                ("updated_at", SortDirection::Ascending) => {
                    sorted_query.then_order_by(updated_at.asc())
                }
                ("updated_at", SortDirection::Descending) => {
                    sorted_query.then_order_by(updated_at.desc())
                }
                _ => sorted_query,
            };
        }

        sorted_query
    }

    pub async fn count(&self) -> Result<i64, diesel::result::Error> {
        use crate::database::schema::media::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        media.count().get_result(&mut conn).await
    }

    pub async fn find(&self, media_id: i32) -> Result<Option<MediaModel>, diesel::result::Error> {
        self.find_by_id(media_id).await
    }

    pub async fn find_all_ordered(&self) -> Result<Vec<MediaModel>, diesel::result::Error> {
        use crate::database::schema::media::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        media
            .order_by(created_at.desc())
            .load::<MediaModel>(&mut conn)
            .await
    }
}

impl From<DbPool> for MediaRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<MediaModel> for MediaRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<MediaModel>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<MediaModel>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(&self, new_item: NewMedia) -> Result<MediaModel, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(
        &self,
        id: i32,
        update: MediaUpdate,
    ) -> Result<MediaModel, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<MediaModel>, diesel::result::Error> {
        use crate::database::schema::media;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "uploaded_by" => {
                media::table
                    .filter(media::uploaded_by.eq_any(&ids))
                    .load::<MediaModel>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
