pub mod diesel_user_repository;
pub mod factories;
pub mod instrumentation;
pub mod jsonapi_resource;
pub mod models;
pub mod pool;
pub mod query_helpers;
pub mod repositories;

pub mod schema;

pub use diesel_user_repository::DieselUserRepository;
pub use factories::{
    AccountFactory, AuditLogFactory, ContactFactory, EntityFactory, Factory, OrganizationFactory,
    UserFactory,
};
pub use models::{
    Account, AccountUpdate, Address, AddressUpdate, AuditLog, Brand, BrandUpdate, Category,
    CategoryUpdate, ClockworkQuery, ClockworkRequest, Contact, ContactUpdate, Customer,
    CustomerUpdate, ImageJob, MediaModel, NewAccount, NewAddress, NewAuditLog, NewBrand,
    NewCategory, NewClockworkQuery, NewClockworkRequest, NewContact, NewCustomer, NewImageJob,
    NewMedia, NewOrder, NewOrderItem, NewOrganization, NewProduct, NewUser, Order, OrderItem,
    OrderItemUpdate, OrderUpdate, Organization, OrganizationUpdate, Product, ProductUpdate,
    UpdateImageJob, User, UserUpdate,
};
pub use pool::{establish_connection_pool, run_migrations, DbConnection, DbPool};
pub use repositories::{
    AccountRepository, AddressRepository, AuditLogRepository, BrandRepository, CategoryRepository,
    ContactRepository, CustomerRepository, ImageJobRepository, MediaRepository,
    OrderItemRepository, OrderRepository, OrganizationRepository, ProductRepository,
    UserRepository,
};

pub use diesel::prelude::*;
pub use diesel_async::RunQueryDsl;

pub use query_helpers::{get_conn, Paginatable, Paginated, PaginationHelper};

pub use jsonapi_resource::{
    DeserializeJsonApi, JsonApiRepository, JsonApiResource, ToNewModel, ToUpdateModel,
};
