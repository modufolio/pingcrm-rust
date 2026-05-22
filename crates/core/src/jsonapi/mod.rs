pub mod configurator;
pub mod controller;
pub mod document;
pub mod error;
pub mod generic_config;
pub mod input_normalizer;
pub mod openapi_generator;
pub mod pagination;
pub mod query;
pub mod query_builder;
pub mod request;
pub mod request_deserializer;
pub mod resource;
pub mod response_builder;

pub mod serializer;
pub mod types;
pub mod validation;

pub use configurator::{
    EntityConfig, FetchStrategy, JsonApiConfigurator, Operations, RelationshipCardinality,
    RelationshipMeta,
};
pub use controller::{JsonApiController, JsonApiState, RegisteredEntity, ResourceHandler};
pub use document::{JsonApiData, JsonApiDocument};
pub use error::ErrorObject;
pub use generic_config::{
    Cardinality, DynamicRelationshipConfig, DynamicResourceConfig, DynamicResourceConfigBuilder,
    RelationshipBuilder, RelationshipConfig, ResourceConfig,
};
pub use input_normalizer::{InputNormalizer, NormalizedPayload, RelationshipValue, RequestFormat};
pub use openapi_generator::OpenApiGenerator;
pub use pagination::{PaginatedResponse, Pagination, PaginationLinks, PaginationMeta};
pub use query::{
    FilterCondition, FilterOperator, PageParams, QueryParams, SearchStrategy, SortDirection,
};
pub use query_builder::{
    ApplyJsonApiFilter, ApplyJsonApiPagination, ApplyJsonApiSort, PaginatedResult,
};
pub use request::JsonApiRequest;
pub use request_deserializer::{
    deserialize_and_validate_simple, deserialize_simple, DeserializeJsonApi,
};
pub use resource::{ResourceIdentifier, ResourceObject};
pub use response_builder::{
    apply_sparse_fieldsets, build_index_response, build_mutation_response, build_show_response,
    build_show_response_with_params,
};

pub use serializer::JsonApiSerializer;
pub use types::{
    JsonApiDocument as JsonApiDocumentType, JsonApiError, JsonApiErrorSource, JsonApiLinks,
    JsonApiRelationship, JsonApiRelationshipData, JsonApiResource, JsonApiResourceIdentifier,
};
pub use validation::{field_errors_to_jsonapi, validation_errors_to_jsonapi, ValidationErrorsExt};
