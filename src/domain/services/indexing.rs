use crate::domain::*;
use async_trait::async_trait;
use mockall::automock;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
	#[error("unable to connect the indexer `{id}`: {details}")]
	Connection { id: IndexerId, details: String },
}

type Result<T> = std::result::Result<T, Error>;

#[automock]
#[async_trait]
pub trait Service {
	async fn fetch_new_events(&self, indexer: &Indexer) -> Result<()>;
}
