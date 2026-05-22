use crate::database::pool::DbPool;
use crate::database::JsonApiResource;
use crate::handlers::generic_handler::GenericResourceHandler;
use appkit_core::jsonapi::{
    DeserializeJsonApi, EntityConfig, JsonApiConfigurator, Operations, ResourceHandler,
    SearchStrategy,
};
use std::collections::HashMap;
use std::sync::Arc;
use validator::Validate;

pub struct V1JsonApiConfig {
    pub configurator: JsonApiConfigurator,
    pub handlers: HashMap<String, Arc<dyn ResourceHandler + Send + Sync>>,
}

struct HandlerBuilder {
    db_pool: DbPool,
    handlers: HashMap<String, Arc<dyn ResourceHandler + Send + Sync>>,
}

impl HandlerBuilder {
    fn new(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            handlers: HashMap::new(),
        }
    }

    fn add<T, CreateDTO, UpdateDTO>(
        &mut self,
        resource_key: &str,
        operations: Operations,
    ) -> &mut Self
    where
        T: JsonApiResource + 'static,
        CreateDTO:
            DeserializeJsonApi + Validate + crate::database::ToNewModel<T::NewModel> + 'static,
        UpdateDTO: DeserializeJsonApi
            + Validate
            + crate::database::ToUpdateModel<T::UpdateModel>
            + 'static,
    {
        let handler = GenericResourceHandler::<T>::new(self.db_pool.clone(), operations)
            .with_create_dto::<CreateDTO>()
            .with_update_dto::<UpdateDTO>();

        self.handlers
            .insert(resource_key.to_string(), Arc::new(handler));
        self
    }

    fn build(self) -> HashMap<String, Arc<dyn ResourceHandler + Send + Sync>> {
        self.handlers
    }
}

pub fn configure_v1_resources(db_pool: DbPool) -> V1JsonApiConfig {
    use crate::database::models::{Account, Contact, Organization, User};
    use crate::requests::account::{CreateAccountRequest, UpdateAccountRequest};
    use crate::requests::contact::{CreateContactRequest, UpdateContactRequest};
    use crate::requests::organization::{CreateOrganizationRequest, UpdateOrganizationRequest};
    use crate::requests::user::{CreateUserRequest, UpdateUserRequest};

    let mut configurator = JsonApiConfigurator::new("/api/v1");

    let mut handlers = HandlerBuilder::new(db_pool);

    let accounts_ops = Operations::all();
    configurator.entity(
        "accounts",
        EntityConfig::new("accounts")
            .operations(accounts_ops)
            .filterable(["id", "name"])
            .sortable(["id", "name", "created_at"])
            .searchable([("name", SearchStrategy::Partial)])
            .has_many("users", "users", "account_id")
            .has_many("organizations", "organizations", "account_id")
            .has_many("contacts", "contacts", "account_id"),
    );
    handlers.add::<Account, CreateAccountRequest, UpdateAccountRequest>("accounts", accounts_ops);

    let contacts_ops = Operations::all();
    configurator.entity(
        "contacts",
        EntityConfig::new("contacts")
            .operations(contacts_ops)
            .filterable([
                "id",
                "first_name",
                "last_name",
                "email",
                "city",
                "organization_id",
                "account_id",
            ])
            .sortable(["id", "first_name", "last_name", "email", "city", "created_at"])
            .searchable([
                ("first_name", SearchStrategy::Partial),
                ("last_name", SearchStrategy::Partial),
                ("email", SearchStrategy::Exact),
                ("city", SearchStrategy::StartsWith),
            ])
            .belongs_to("organization", "organizations", "organization_id")
            .belongs_to("account", "accounts", "account_id"),
    );
    handlers.add::<Contact, CreateContactRequest, UpdateContactRequest>("contacts", contacts_ops);

    let orgs_ops = Operations::all();
    configurator.entity(
        "organizations",
        EntityConfig::new("organizations")
            .operations(orgs_ops)
            .filterable(["id", "name", "email", "city", "country", "account_id"])
            .sortable(["id", "name", "city", "country", "created_at"])
            .searchable([
                ("name", SearchStrategy::Partial),
                ("email", SearchStrategy::Exact),
                ("city", SearchStrategy::StartsWith),
                ("country", SearchStrategy::Exact),
            ])
            .belongs_to("account", "accounts", "account_id")
            .has_many("contacts", "contacts", "organization_id"),
    );
    handlers.add::<Organization, CreateOrganizationRequest, UpdateOrganizationRequest>(
        "organizations",
        orgs_ops,
    );

    let users_ops = Operations::all();
    configurator.entity(
        "users",
        EntityConfig::new("users")
            .operations(users_ops)
            .filterable([
                "id",
                "email",
                "first_name",
                "last_name",
                "account_status",
                "account_id",
            ])
            .sortable(["id", "email", "first_name", "last_name", "created_at"])
            .searchable([
                ("email", SearchStrategy::Exact),
                ("first_name", SearchStrategy::Partial),
                ("last_name", SearchStrategy::Partial),
            ])
            .belongs_to("account", "accounts", "account_id"),
    );
    handlers.add::<User, CreateUserRequest, UpdateUserRequest>("users", users_ops);

    V1JsonApiConfig {
        configurator,
        handlers: handlers.build(),
    }
}
