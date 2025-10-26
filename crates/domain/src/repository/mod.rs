use crate::repository::{
    record::{MockRecordRepository, RecordRepository},
    user::{MockUserRepository, UserRepository},
};

pub mod record;
pub mod user;

pub trait Repositories {
    type UserRepositoryImpl: UserRepository;
    type RecordRepositoryImpl: RecordRepository;

    fn user(&self) -> &Self::UserRepositoryImpl;
    fn record(&self) -> &Self::RecordRepositoryImpl;
}

pub struct MockRepositories {
    pub user: MockUserRepository,
    pub record: MockRecordRepository,
}

impl Repositories for MockRepositories {
    type UserRepositoryImpl = MockUserRepository;
    type RecordRepositoryImpl = MockRecordRepository;

    fn user(&self) -> &Self::UserRepositoryImpl {
        &self.user
    }

    fn record(&self) -> &Self::RecordRepositoryImpl {
        &self.record
    }
}
