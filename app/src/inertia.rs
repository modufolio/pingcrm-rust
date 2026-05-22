use serde_json::Value;
use tower_sessions::Session;

use crate::app::App;
use crate::database::{AccountRepository, DbPool};
use appkit_core::inertia::{AccountData, SharedProps, UserData};
use appkit_core::security::user::User;

pub struct DefaultProps;

impl DefaultProps {

    pub async fn create(user: Option<&User>, session: &Session, state: &App) -> SharedProps {
        let mut props = SharedProps::new();

        if let Some(user) = user {
            props = props.with_user(user_to_user_data(user, state.db_pool.clone()).await);
        }

        if let Ok(Some(msg)) = session.get::<String>("flash_success").await {
            props = props.with_success(msg);
            let _ = session.remove::<String>("flash_success").await;
        }
        if let Ok(Some(msg)) = session.get::<String>("flash_error").await {
            props = props.with_error(msg);
            let _ = session.remove::<String>("flash_error").await;
        }

        props
    }

    pub async fn merge(
        user: Option<&User>,
        session: &Session,
        state: &App,
        page_props: Value,
    ) -> Value {
        Self::create(user, session, state).await.merge_with(page_props)
    }
}

pub async fn user_to_user_data(user: &User, pool: DbPool) -> UserData {
    let account = if let Some(account_id) = user.account_id {
        let account_repo = AccountRepository::new(pool);
        match account_repo.find_by_id(account_id).await {
            Ok(Some(acc)) => AccountData {
                id: acc.id,
                name: acc.name,
            },
            _ => AccountData::default(),
        }
    } else {
        AccountData::default()
    };

    UserData {
        id: user.id.to_string(),
        email: user.email.clone(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        account,
        role: Some(format!("ROLE_{:?}", user.role).to_uppercase()),
        two_factor_enabled: user.two_factor_enabled,
    }
}
