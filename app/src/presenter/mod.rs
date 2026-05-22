pub mod user_presenter;

pub use user_presenter::{present_users, present_users_list, UserListPresenter, UserPresenter};

use serde::Serialize;

pub trait Presenter<T, O>
where
    O: Serialize,
{
    fn present(&self, entity: &T) -> O;

    fn present_collection(&self, entities: &[T]) -> Vec<O> {
        entities.iter().map(|e| self.present(e)).collect()
    }
}
