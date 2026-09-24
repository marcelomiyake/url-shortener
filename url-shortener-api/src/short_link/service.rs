use std::{future::Future, sync::Arc};

use chrono::{DateTime, Utc};

use super::domain::{DestinationUrl, ShortCode};

const FIRST_CODE_LENGTH: usize = 12;
const MAX_CODE_LENGTH: usize = 43;

#[derive(Clone)]
pub struct ShortLinkService<R> {
    repository: Arc<R>,
}

pub trait ShortLinkRepository: Send + Sync {
    fn insert_if_absent<'a>(
        &'a self,
        code: &'a ShortCode,
        destination: &'a DestinationUrl,
        created_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<InsertOutcome, StorageError>> + Send + 'a;

    fn find_by_code<'a>(
        &'a self,
        code: &'a ShortCode,
    ) -> impl Future<Output = Result<Option<String>, StorageError>> + Send + 'a;

    fn is_ready(&self) -> impl Future<Output = Result<(), StorageError>> + Send;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InsertOutcome {
    Inserted,
    AlreadyExists(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatedShortLink {
    pub code: ShortCode,
    pub was_created: bool,
}

#[derive(Debug, thiserror::Error)]
#[error("storage operation failed")]
pub struct StorageError;

#[derive(Debug, thiserror::Error)]
pub enum ShortLinkError {
    #[error("storage operation failed")]
    Storage(#[from] StorageError),
    #[error("short-code digest prefix space was exhausted")]
    CodeSpaceExhausted,
    #[error("stored canonical destination is invalid")]
    StoredDataInvalid,
}

impl<R> ShortLinkService<R>
where
    R: ShortLinkRepository,
{
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub async fn create(
        &self,
        destination: &DestinationUrl,
    ) -> Result<CreatedShortLink, ShortLinkError> {
        let digest = destination.digest();
        let created_at = Utc::now();
        for length in FIRST_CODE_LENGTH..=MAX_CODE_LENGTH {
            let code = ShortCode::from_digest_prefix(&digest, length)
                .map_err(|_| ShortLinkError::CodeSpaceExhausted)?;

            match self
                .repository
                .insert_if_absent(&code, destination, created_at)
                .await?
            {
                InsertOutcome::Inserted => {
                    return Ok(CreatedShortLink {
                        code,
                        was_created: true,
                    });
                }
                InsertOutcome::AlreadyExists(stored_destination)
                    if stored_destination == destination.as_str() =>
                {
                    return Ok(CreatedShortLink {
                        code,
                        was_created: false,
                    });
                }
                InsertOutcome::AlreadyExists(_) => {}
            }
        }

        Err(ShortLinkError::CodeSpaceExhausted)
    }

    pub async fn resolve(
        &self,
        code: &ShortCode,
    ) -> Result<Option<DestinationUrl>, ShortLinkError> {
        self.repository
            .find_by_code(code)
            .await?
            .map(|value| {
                DestinationUrl::parse(&value).map_err(|_| ShortLinkError::StoredDataInvalid)
            })
            .transpose()
    }

    pub async fn is_ready(&self) -> Result<(), ShortLinkError> {
        self.repository.is_ready().await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::{DateTime, Utc};

    use super::{InsertOutcome, ShortLinkRepository, ShortLinkService, StorageError};
    use crate::short_link::domain::{DestinationUrl, ShortCode};

    #[derive(Clone, Default)]
    struct MemoryRepository {
        rows: Arc<Mutex<HashMapRows>>,
        fail: bool,
    }

    type HashMapRows = std::collections::HashMap<String, String>;

    impl ShortLinkRepository for MemoryRepository {
        async fn insert_if_absent(
            &self,
            code: &ShortCode,
            destination: &DestinationUrl,
            _created_at: DateTime<Utc>,
        ) -> Result<InsertOutcome, StorageError> {
            if self.fail {
                return Err(StorageError);
            }
            let mut rows = self.rows.lock().expect("memory repository lock");
            match rows.get(code.as_str()) {
                Some(stored) => Ok(InsertOutcome::AlreadyExists(stored.clone())),
                None => {
                    rows.insert(code.as_str().to_owned(), destination.as_str().to_owned());
                    Ok(InsertOutcome::Inserted)
                }
            }
        }

        async fn find_by_code(&self, code: &ShortCode) -> Result<Option<String>, StorageError> {
            if self.fail {
                return Err(StorageError);
            }
            Ok(self
                .rows
                .lock()
                .expect("memory repository lock")
                .get(code.as_str())
                .cloned())
        }

        async fn is_ready(&self) -> Result<(), StorageError> {
            if self.fail { Err(StorageError) } else { Ok(()) }
        }
    }

    #[tokio::test]
    async fn duplicate_destination_returns_the_existing_code() {
        let service = ShortLinkService::new(MemoryRepository::default());
        let destination = DestinationUrl::parse("https://example.com/one").unwrap();

        let first = service.create(&destination).await.unwrap();
        let duplicate = service.create(&destination).await.unwrap();

        assert!(first.was_created);
        assert!(!duplicate.was_created);
        assert_eq!(first.code, duplicate.code);
    }

    #[tokio::test]
    async fn hash_collision_extends_the_code_prefix() {
        let repository = MemoryRepository::default();
        let destination = DestinationUrl::parse("https://example.com/target").unwrap();
        let occupied = DestinationUrl::parse("https://example.com/occupied").unwrap();
        let prefix = ShortCode::from_digest_prefix(&destination.digest(), 12).unwrap();
        repository
            .rows
            .lock()
            .unwrap()
            .insert(prefix.to_string(), occupied.as_str().to_owned());
        let service = ShortLinkService::new(repository);

        let created = service.create(&destination).await.unwrap();

        assert!(created.was_created);
        assert_eq!(created.code.as_str().len(), 13);
        assert!(created.code.as_str().starts_with(prefix.as_str()));
    }

    #[tokio::test]
    async fn repository_failure_is_reported_without_exposing_storage_details() {
        let service = ShortLinkService::new(MemoryRepository {
            rows: Arc::default(),
            fail: true,
        });
        let destination = DestinationUrl::parse("https://example.com/target").unwrap();

        assert!(service.create(&destination).await.is_err());
    }
}
