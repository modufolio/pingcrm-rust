use crate::config::Operations;
use crate::database::pool::DbPool;
use crate::database::{JsonApiRepository, JsonApiResource};

use appkit_core::error::AppError;
use appkit_core::jsonapi::{
    build_index_response, build_show_response_with_params, DeserializeJsonApi, QueryParams,
    ResourceHandler, ResourceObject,
};
use async_trait::async_trait;
use serde_json::Value;
use std::marker::PhantomData;
use std::sync::Arc;
use validator::Validate;

type CreateConverter<T> = Arc<dyn Fn(&Value) -> Result<T, AppError> + Send + Sync>;

type UpdateConverter<T> = Arc<dyn Fn(&Value) -> Result<T, AppError> + Send + Sync>;

pub struct GenericResourceHandler<T: JsonApiResource> {
    pub db_pool: DbPool,
    pub operations: Operations,
    create_converter: Option<CreateConverter<T::NewModel>>,
    update_converter: Option<UpdateConverter<T::UpdateModel>>,
    _phantom: PhantomData<T>,
}

impl<T: JsonApiResource> GenericResourceHandler<T> {
    pub fn new(db_pool: DbPool, operations: Operations) -> Self {
        Self {
            db_pool,
            operations,
            create_converter: None,
            update_converter: None,
            _phantom: PhantomData,
        }
    }

    pub fn with_create_dto<DTO>(mut self) -> Self
    where
        DTO: DeserializeJsonApi + Validate + crate::database::ToNewModel<T::NewModel> + 'static,
    {
        self.create_converter = Some(Arc::new(move |payload| {
            let dto = DTO::deserialize_and_validate(payload, "application/vnd.api+json", T::TYPE)?;

            Ok(dto.to_new_model())
        }));

        self
    }

    pub fn with_update_dto<DTO>(mut self) -> Self
    where
        DTO: DeserializeJsonApi
            + Validate
            + crate::database::ToUpdateModel<T::UpdateModel>
            + 'static,
    {
        self.update_converter = Some(Arc::new(move |payload| {
            let dto = DTO::deserialize_and_validate(payload, "application/vnd.api+json", T::TYPE)?;

            Ok(dto.to_update_model())
        }));

        self
    }

    fn repo(&self) -> T::Repository {
        T::Repository::from(self.db_pool.clone())
    }

    fn resource_type_str(&self) -> &'static str {
        T::TYPE
    }
}

#[async_trait]
impl<T: JsonApiResource> ResourceHandler for GenericResourceHandler<T> {
    fn resource_type(&self) -> &'static str {
        T::TYPE
    }

    fn allowed_fields(&self) -> &'static [&'static str] {
        T::field_names()
    }

    fn allowed_includes(&self) -> &'static [&'static str] {
        &[]
    }

    fn is_include_allowed(&self, include: &str) -> bool {
        T::relationships().iter().any(|rel| rel.name == include)
    }

    async fn index(&self, params: QueryParams) -> Result<Value, AppError> {
        if !self.operations.index {
            return Err(AppError::MethodNotAllowed(format!(
                "Index operation not allowed for resource '{}'",
                self.resource_type_str()
            )));
        }

        let allowed_fields = T::field_names();
        params.validate_filters(allowed_fields)?;
        params.validate_sorts(allowed_fields)?;
        params.validate_fields(T::TYPE, allowed_fields)?;

        let relationship_names: Vec<String> = T::relationships()
            .iter()
            .map(|rel| rel.name.clone())
            .collect();
        let relationship_refs: Vec<&str> = relationship_names.iter().map(|s| s.as_str()).collect();
        params.validate_includes(&relationship_refs)?;

        let repo = self.repo();

        let result = repo
            .paginate(&params)
            .await
            .map_err(|e| AppError::database_error(e))?;

        let resources: Vec<ResourceObject> =
            result.items.iter().map(|item| item.to_resource()).collect();

        let mut included: Vec<ResourceObject> = vec![];
        let mut included_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        if !params.includes.is_empty() {
            let includes = &params.includes;
            let relationships = T::relationships();

            let resource_refs: Vec<&T> = result.items.iter().collect();

            for include_name in includes {
                if let Some(rel_meta) = relationships.iter().find(|r| r.name == *include_name) {
                    match rel_meta.cardinality {
                        appkit_core::jsonapi::RelationshipCardinality::ToOne => {
                            match T::load_related_to_one_batch(
                                &self.db_pool,
                                &resource_refs,
                                rel_meta,
                                include_name,
                            )
                            .await
                            {
                                Ok(related_map) => {
                                    for resource_obj in related_map.values() {
                                        let key = format!(
                                            "{}:{}",
                                            resource_obj.resource_type,
                                            resource_obj.id.as_ref().unwrap_or(&"".to_string())
                                        );
                                        if !included_ids.contains(&key) {
                                            included_ids.insert(key);
                                            included.push(resource_obj.clone());
                                        }
                                    }
                                }
                                Err(AppError::NotImplemented) => {
                                    for item in &result.items {
                                        let item_id = item.id().parse::<i32>().unwrap_or(0);
                                        match T::load_related_to_one(
                                            &self.db_pool,
                                            item,
                                            rel_meta,
                                            item_id,
                                            include_name,
                                        )
                                        .await
                                        {
                                            Ok(related_value) => {
                                                if let Some(data) = related_value.get("data") {
                                                    if !data.is_null() {
                                                        if let Ok(resource_obj) =
                                                            serde_json::from_value::<ResourceObject>(
                                                                data.clone(),
                                                            )
                                                        {
                                                            let key = format!(
                                                                "{}:{}",
                                                                resource_obj.resource_type,
                                                                resource_obj
                                                                    .id
                                                                    .as_ref()
                                                                    .unwrap_or(&"".to_string())
                                                            );
                                                            if !included_ids.contains(&key) {
                                                                included_ids.insert(key);
                                                                included.push(resource_obj);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                        appkit_core::jsonapi::RelationshipCardinality::ToMany => {
                            match T::load_related_to_many_batch(
                                &self.db_pool,
                                &resource_refs,
                                rel_meta,
                                include_name,
                            )
                            .await
                            {
                                Ok(related_map) => {
                                    for resources in related_map.values() {
                                        for resource_obj in resources {
                                            let key = format!(
                                                "{}:{}",
                                                resource_obj.resource_type,
                                                resource_obj.id.as_ref().unwrap_or(&"".to_string())
                                            );
                                            if !included_ids.contains(&key) {
                                                included_ids.insert(key);
                                                included.push(resource_obj.clone());
                                            }
                                        }
                                    }
                                }
                                Err(AppError::NotImplemented) => {
                                    for item in &result.items {
                                        let item_id = item.id().parse::<i32>().unwrap_or(0);
                                        match T::load_related_to_many(
                                            &self.db_pool,
                                            item,
                                            rel_meta,
                                            item_id,
                                            include_name,
                                        )
                                        .await
                                        {
                                            Ok(related_value) => {
                                                if let Some(data) = related_value.get("data") {
                                                    if let Some(data_array) = data.as_array() {
                                                        for data_item in data_array {
                                                            if let Ok(resource_obj) =
                                                                serde_json::from_value::<
                                                                    ResourceObject,
                                                                >(
                                                                    data_item.clone()
                                                                )
                                                            {
                                                                let key = format!(
                                                                    "{}:{}",
                                                                    resource_obj.resource_type,
                                                                    resource_obj
                                                                        .id
                                                                        .as_ref()
                                                                        .unwrap_or(&"".to_string())
                                                                );
                                                                if !included_ids.contains(&key) {
                                                                    included_ids.insert(key);
                                                                    included.push(resource_obj);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                    }
                }
            }
        }

        Ok(build_index_response(
            resources,
            included,
            result.total,
            &params,
            &format!("/api/v1/{}", self.resource_type_str()),
        ))
    }

    async fn show(&self, id: i32, params: QueryParams) -> Result<Value, AppError> {
        if !self.operations.show {
            return Err(AppError::MethodNotAllowed(format!(
                "Show operation not allowed for resource '{}'",
                self.resource_type_str()
            )));
        }

        let allowed_fields = T::field_names();
        params.validate_fields(T::TYPE, allowed_fields)?;

        let relationship_names: Vec<String> = T::relationships()
            .iter()
            .map(|rel| rel.name.clone())
            .collect();
        let relationship_refs: Vec<&str> = relationship_names.iter().map(|s| s.as_str()).collect();
        params.validate_includes(&relationship_refs)?;

        let repo = self.repo();

        let resource = repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError::database_error(e))?
            .ok_or_else(|| {
                AppError::NotFound(format!("{} {} not found", self.resource_type_str(), id))
            })?;

        let mut included: Vec<ResourceObject> = vec![];

        if !params.includes.is_empty() {
            let includes = &params.includes;
            let relationships = T::relationships();

            for include_name in includes {
                if let Some(rel_meta) = relationships.iter().find(|r| r.name == *include_name) {
                    match rel_meta.cardinality {
                        appkit_core::jsonapi::RelationshipCardinality::ToOne => {
                            match T::load_related_to_one(
                                &self.db_pool,
                                &resource,
                                rel_meta,
                                id,
                                include_name,
                            )
                            .await
                            {
                                Ok(related_value) => {
                                    if let Some(data) = related_value.get("data") {
                                        if !data.is_null() {
                                            match serde_json::from_value::<ResourceObject>(
                                                data.clone(),
                                            ) {
                                                Ok(resource_obj) => {
                                                    included.push(resource_obj);
                                                }
                                                Err(_) => {}
                                            }
                                        }
                                    }
                                }
                                Err(AppError::NotImplemented) => {}
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        appkit_core::jsonapi::RelationshipCardinality::ToMany => {
                            match T::load_related_to_many(
                                &self.db_pool,
                                &resource,
                                rel_meta,
                                id,
                                include_name,
                            )
                            .await
                            {
                                Ok(related_value) => {
                                    if let Some(data) = related_value.get("data") {
                                        if let Some(data_array) = data.as_array() {
                                            for item in data_array {
                                                match serde_json::from_value::<ResourceObject>(
                                                    item.clone(),
                                                ) {
                                                    Ok(resource_obj) => {
                                                        included.push(resource_obj);
                                                    }
                                                    Err(_) => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(AppError::NotImplemented) => {}
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(build_show_response_with_params(
            resource.to_resource(),
            included,
            &params,
            &format!("/api/v1/{}/{}", self.resource_type_str(), id),
        ))
    }

    async fn create(&self, payload: Value) -> Result<Value, AppError> {
        if !self.operations.create {
            return Err(AppError::MethodNotAllowed(format!(
                "Create operation not allowed for resource '{}'",
                self.resource_type_str()
            )));
        }

        let new_model = if let Some(converter) = &self.create_converter {
            converter(&payload)?
        } else {
            return Err(AppError::NotImplemented);
        };

        let repo = self.repo();

        let created = repo
            .create(new_model)
            .await
            .map_err(|e| AppError::database_error(e))?;

        Ok(build_show_response_with_params(
            created.to_resource(),
            vec![],
            &Default::default(),
            &format!("/api/v1/{}/{}", self.resource_type_str(), created.id()),
        ))
    }

    async fn update(
        &self,
        id: i32,
        payload: Value,
        _params: QueryParams,
    ) -> Result<Value, AppError> {
        if !self.operations.update {
            return Err(AppError::MethodNotAllowed(format!(
                "Update operation not allowed for resource '{}'",
                self.resource_type_str()
            )));
        }

        let repo = self.repo();

        repo.find_by_id(id)
            .await
            .map_err(|e| AppError::database_error(e))?
            .ok_or_else(|| {
                AppError::NotFound(format!("{} {} not found", self.resource_type_str(), id))
            })?;

        let update_model = if let Some(converter) = &self.update_converter {
            converter(&payload)?
        } else {
            return Err(AppError::NotImplemented);
        };

        let updated = repo
            .update(id, update_model)
            .await
            .map_err(|e| AppError::database_error(e))?;

        Ok(build_show_response_with_params(
            updated.to_resource(),
            vec![],
            &Default::default(),
            &format!("/api/v1/{}/{}", self.resource_type_str(), id),
        ))
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        if !self.operations.delete {
            return Err(AppError::MethodNotAllowed(format!(
                "Delete operation not allowed for resource '{}'",
                self.resource_type_str()
            )));
        }

        let repo = self.repo();

        repo.find_by_id(id)
            .await
            .map_err(|e| AppError::database_error(e))?
            .ok_or_else(|| {
                AppError::NotFound(format!("{} {} not found", self.resource_type_str(), id))
            })?;

        repo.delete(id)
            .await
            .map_err(|e| AppError::database_error(e))?;

        Ok(())
    }

    async fn related(
        &self,
        id: i32,
        relationship: &str,
        _params: QueryParams,
    ) -> Result<Value, AppError> {
        use appkit_core::jsonapi::RelationshipCardinality;

        let allowed_relationships = T::relationships();
        let relationship_meta = allowed_relationships
            .iter()
            .find(|r| r.name == relationship)
            .ok_or_else(|| {
                AppError::BadRequest(format!(
                    "Relationship '{}' does not exist on resource '{}'",
                    relationship,
                    self.resource_type_str()
                ))
            })?;

        let repo = self.repo();
        let resource = repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError::database_error(e))?
            .ok_or_else(|| {
                AppError::NotFound(format!("{} {} not found", self.resource_type_str(), id))
            })?;

        match relationship_meta.cardinality {
            RelationshipCardinality::ToOne => {
                T::load_related_to_one(
                    &self.db_pool,
                    &resource,
                    relationship_meta,
                    id,
                    relationship,
                )
                .await
            }
            RelationshipCardinality::ToMany => {
                T::load_related_to_many(
                    &self.db_pool,
                    &resource,
                    relationship_meta,
                    id,
                    relationship,
                )
                .await
            }
        }
    }
}
