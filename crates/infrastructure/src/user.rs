use domain::{
    entity::user::User,
    repository::user::{UserRepository, UserRepositoryError},
};
use sea_orm::DbConn;

pub struct UserRepositoryImpl {
    db: DbConn,
}

impl UserRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

impl UserRepository for UserRepositoryImpl {
    fn create(&self, user: User) -> Result<User, UserRepositoryError> {
        // TODO: implement
        unimplemented!()
    }
}
