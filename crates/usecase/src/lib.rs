pub mod user;

pub struct Usecases<R: domain::repository::Repositories> {
    pub user: user::UserUsecase<R>,
}
