#![cfg(loom)]

use loom::sync::Arc;
use loom::thread;

#[derive(Clone, Debug)]
struct MockUser {
    id: i32,
    email: String,
}

#[derive(Clone)]
struct MockUserRepository {}

impl MockUserRepository {
    fn new() -> Self {
        Self {}
    }

    fn find_by_id(&self, id: i32) -> Option<MockUser> {
        if id == 1 {
            Some(MockUser {
                id: 1,
                email: "test@example.com".to_string(),
            })
        } else {
            None
        }
    }
}

struct MockJwtAuthenticator<R> {
    secret: String,
    user_repository: R,
}

impl<R> MockJwtAuthenticator<R>
where
    R: Clone,
{
    fn new(secret: String, user_repository: R) -> Self {
        Self {
            secret,
            user_repository,
        }
    }
}

struct MockSessionAuthenticator<R> {
    user_repository: R,
}

impl<R> MockSessionAuthenticator<R>
where
    R: Clone,
{
    fn new(user_repository: R) -> Self {
        Self { user_repository }
    }
}

enum AuthenticatorType<R>
where
    R: Clone,
{
    Jwt(MockJwtAuthenticator<R>),
    Session(MockSessionAuthenticator<R>),
}

struct AuthenticatorChain<R>
where
    R: Clone,
{
    authenticators: Vec<AuthenticatorType<R>>,
}

impl<R> AuthenticatorChain<R>
where
    R: Clone,
{
    fn new() -> Self {
        Self {
            authenticators: Vec::new(),
        }
    }

    fn add_jwt(mut self, jwt: MockJwtAuthenticator<R>) -> Self {
        self.authenticators.push(AuthenticatorType::Jwt(jwt));
        self
    }

    fn add_session(mut self, session: MockSessionAuthenticator<R>) -> Self {
        self.authenticators
            .push(AuthenticatorType::Session(session));
        self
    }

    fn authenticate(&self, _request_id: usize) -> Option<i32> {
        for authenticator in &self.authenticators {
            match authenticator {
                AuthenticatorType::Jwt(_jwt) => {
                    return Some(1);
                }
                AuthenticatorType::Session(_session) => {
                    continue;
                }
            }
        }
        None
    }
}

#[test]
fn test_authenticator_chain_concurrent_reads() {
    loom::model(|| {
        let user_repo = MockUserRepository::new();

        let chain = AuthenticatorChain::new()
            .add_jwt(MockJwtAuthenticator::new(
                "test_secret".to_string(),
                user_repo.clone(),
            ))
            .add_session(MockSessionAuthenticator::new(user_repo));

        let chain = Arc::new(chain);

        let chain1 = Arc::clone(&chain);
        let chain2 = Arc::clone(&chain);

        let t1 = thread::spawn(move || {
            let result = chain1.authenticate(1);
            assert!(result.is_some());
        });

        let t2 = thread::spawn(move || {
            let result = chain2.authenticate(2);
            assert!(result.is_some());
        });

        t1.join().unwrap();
        t2.join().unwrap();
    });
}

#[test]
fn test_authenticator_chain_many_concurrent_reads() {
    loom::model(|| {
        let user_repo = MockUserRepository::new();

        let chain = AuthenticatorChain::new()
            .add_jwt(MockJwtAuthenticator::new(
                "secret".to_string(),
                user_repo.clone(),
            ))
            .add_session(MockSessionAuthenticator::new(user_repo));

        let chain = Arc::new(chain);

        let handles: Vec<_> = (0..3)
            .map(|i| {
                let chain = Arc::clone(&chain);
                thread::spawn(move || {
                    let result = chain.authenticate(i);
                    assert!(result.is_some());
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[test]
fn test_generic_repository_clone_safety() {
    loom::model(|| {
        let repo = MockUserRepository::new();

        let repo1 = repo.clone();
        let repo2 = repo.clone();

        let t1 = thread::spawn(move || {
            let user = repo1.find_by_id(1);
            assert!(user.is_some());
        });

        let t2 = thread::spawn(move || {
            let user = repo2.find_by_id(1);
            assert!(user.is_some());
        });

        t1.join().unwrap();
        t2.join().unwrap();
    });
}

#[test]
fn test_authenticator_enum_dispatch_safety() {
    loom::model(|| {
        let user_repo = MockUserRepository::new();

        let jwt = MockJwtAuthenticator::new("secret".to_string(), user_repo.clone());
        let session = MockSessionAuthenticator::new(user_repo);

        let auth1 = Arc::new(AuthenticatorType::Jwt(jwt));
        let auth2 = Arc::new(AuthenticatorType::Session(session));

        let a1 = Arc::clone(&auth1);
        let a2 = Arc::clone(&auth2);

        let t1 = thread::spawn(move || match &*a1 {
            AuthenticatorType::Jwt(_) => {}
            _ => panic!("Wrong authenticator type"),
        });

        let t2 = thread::spawn(move || match &*a2 {
            AuthenticatorType::Session(_) => {}
            _ => panic!("Wrong authenticator type"),
        });

        t1.join().unwrap();
        t2.join().unwrap();
    });
}

#[test]
fn test_zero_cost_generic_safety() {
    loom::model(|| {
        let repo = MockUserRepository::new();
        let jwt = MockJwtAuthenticator::new("secret".to_string(), repo);

        let jwt = Arc::new(jwt);

        let jwt1 = Arc::clone(&jwt);
        let jwt2 = Arc::clone(&jwt);

        let t1 = thread::spawn(move || {
            let _secret = &jwt1.secret;
        });

        let t2 = thread::spawn(move || {
            let _secret = &jwt2.secret;
        });

        t1.join().unwrap();
        t2.join().unwrap();
    });
}
