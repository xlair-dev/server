use crate::repository::{
    music::{MockMusicRepository, MusicRepository},
    record::{MockRecordRepository, RecordRepository},
    user::{MockUserRepository, UserRepository},
};

pub mod music;
pub mod record;
pub mod user;

pub trait Repositories {
    type UserRepositoryImpl: UserRepository;
    type RecordRepositoryImpl: RecordRepository;
    type MusicRepositoryImpl: MusicRepository;

    fn user(&self) -> &Self::UserRepositoryImpl;
    fn record(&self) -> &Self::RecordRepositoryImpl;
    fn music(&self) -> &Self::MusicRepositoryImpl;
}

pub struct MockRepositories {
    pub user: MockUserRepository,
    pub record: MockRecordRepository,
    pub music: MockMusicRepository,
}

impl Repositories for MockRepositories {
    type UserRepositoryImpl = MockUserRepository;
    type RecordRepositoryImpl = MockRecordRepository;
    type MusicRepositoryImpl = MockMusicRepository;

    fn user(&self) -> &Self::UserRepositoryImpl {
        &self.user
    }

    fn record(&self) -> &Self::RecordRepositoryImpl {
        &self.record
    }

    fn music(&self) -> &Self::MusicRepositoryImpl {
        &self.music
    }
}
