use crate::database::pool::DbPool;

use appkit_core::jsonapi::ResourceObject;
use serde_json::Value;
use std::collections::HashMap;

pub use appkit_core::jsonapi::{FetchStrategy, RelationshipCardinality, RelationshipMeta};

#[async_trait::async_trait]
pub trait JsonApiResource: Sized + Send + Sync + Clone + 'static {
    const TYPE: &'static str;

    type Repository: JsonApiRepository<Self> + From<DbPool>;

    type NewModel;

    type UpdateModel;

    fn id(&self) -> String;

    fn table_name() -> &'static str;

    fn field_names() -> &'static [&'static str];

    fn attributes(&self) -> Vec<(&'static str, Value)>;

    fn relationships() -> Vec<RelationshipMeta> {
        vec![]
    }

    fn repository(pool: DbPool) -> Self::Repository;

    fn to_resource(&self) -> ResourceObject {
        let mut res = ResourceObject::new(Self::TYPE, self.id());
        for (key, value) in self.attributes() {
            res = res.set_attribute(key, value);
        }
        res
    }

    fn to_resource_with_relationships(&self) -> ResourceObject {
        self.to_resource()
    }

    async fn load_related_to_one(
        _pool: &DbPool,
        _resource: &Self,
        _relationship_meta: &appkit_core::jsonapi::RelationshipMeta,
        _id: i32,
        _relationship: &str,
    ) -> Result<Value, appkit_core::error::AppError> {
        Err(appkit_core::error::AppError::NotImplemented)
    }

    async fn load_related_to_many(
        _pool: &DbPool,
        _resource: &Self,
        _relationship_meta: &appkit_core::jsonapi::RelationshipMeta,
        _id: i32,
        _relationship: &str,
    ) -> Result<Value, appkit_core::error::AppError> {
        Err(appkit_core::error::AppError::NotImplemented)
    }

    async fn load_related_to_one_batch(
        pool: &DbPool,
        resources: &[&Self],
        relationship_meta: &appkit_core::jsonapi::RelationshipMeta,
        relationship: &str,
    ) -> Result<HashMap<i32, ResourceObject>, appkit_core::error::AppError> {
        let mut result = HashMap::new();
        for resource in resources {
            let id = resource.id().parse::<i32>().unwrap_or(0);
            match Self::load_related_to_one(pool, resource, relationship_meta, id, relationship)
                .await
            {
                Ok(value) => {
                    if let Some(data) = value.get("data") {
                        if !data.is_null() {
                            if let Ok(obj) = serde_json::from_value(data.clone()) {
                                result.insert(id, obj);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(result)
    }

    async fn load_related_to_many_batch(
        pool: &DbPool,
        resources: &[&Self],
        relationship_meta: &appkit_core::jsonapi::RelationshipMeta,
        relationship: &str,
    ) -> Result<HashMap<i32, Vec<ResourceObject>>, appkit_core::error::AppError> {
        let mut result = HashMap::new();
        for resource in resources {
            let id = resource.id().parse::<i32>().unwrap_or(0);
            match Self::load_related_to_many(pool, resource, relationship_meta, id, relationship)
                .await
            {
                Ok(value) => {
                    if let Some(data) = value.get("data") {
                        if let Some(array) = data.as_array() {
                            let objects: Vec<ResourceObject> = array
                                .iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect();
                            result.insert(id, objects);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(result)
    }
}

#[async_trait::async_trait]
pub trait JsonApiRepository<T>: Clone + Send + Sync + 'static
where
    T: JsonApiResource,
{
    async fn find_by_id(&self, id: i32) -> Result<Option<T>, diesel::result::Error>;

    async fn paginate(
        &self,
        params: &appkit_core::jsonapi::QueryParams,
    ) -> Result<appkit_core::jsonapi::PaginatedResult<T>, diesel::result::Error>;

    async fn create(&self, new_item: T::NewModel) -> Result<T, diesel::result::Error>;

    async fn update(&self, id: i32, update: T::UpdateModel) -> Result<T, diesel::result::Error>;

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error>;

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<T>, diesel::result::Error>;
}

pub trait DeserializeJsonApi: Sized {
    fn from_json_api(payload: &Value) -> Result<Self, appkit_core::error::AppError>;
}

pub trait ToNewModel<T> {
    fn to_new_model(&self) -> T;
}

pub trait ToUpdateModel<T> {
    fn to_update_model(&self) -> T;
}
