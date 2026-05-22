pub mod access_control;
pub mod audit;
pub mod auth_context;
pub mod authenticator;
pub mod authorization;
pub mod csrf;
pub mod firewall;
pub mod password_hasher;
pub mod rate_limiting;
pub mod session;
pub mod token;
pub mod two_factor;
pub mod user;

pub use authenticator::{
    AuthenticationResult, Authenticator, AuthenticatorChain, BruteForceProtection,
    FormLoginAuthenticator, InMemoryBruteForceProtection, InMemoryRememberMeRepository,
    JwtAuthenticator, JwtClaims, RememberMeAuthenticator, RememberMeHelper, RememberMeRepository,
    RememberMeToken, SessionAuthenticator, DEFAULT_REMEMBER_ME_DURATION,
};

pub use access_control::{access_control_middleware, AccessControlConfig, AccessControlRule};
pub use audit::{AuditEventType, AuditLogEntry, AuditLogger};
pub use auth_context::AuthContext;
pub use csrf::CsrfTokenManager;
pub use firewall::{FirewallRule, FirewallService};
pub use password_hasher::{hash_password, needs_rehash, verify_password};
pub use rate_limiting::{rate_limit_middleware, RateLimitConfig, RateLimiter};
pub use session::{AppSessionStore, SessionManager, SessionStoreBuilder, SessionStoreConfig};
pub use token::{Token, TwoFactorToken, UsernamePasswordToken};
pub use two_factor::TotpManager;
pub use user::{CreateUserDto, LoginCredentials, UpdateUserDto, User, UserRole, UserStatus};

pub use audit::{AuditEventType as SecurityAuditEventType, AuditLogEntry as SecurityAuditLogEntry};
pub use user::User as SecurityUser;
