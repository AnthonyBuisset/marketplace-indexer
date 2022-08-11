mod proto;

use proto as apibara;

mod error;
use error::Error;

mod indexer_repository;

use apibara::indexer_manager_client::IndexerManagerClient;
use tokio::sync::RwLock;

pub struct Client(RwLock<IndexerManagerClient<tonic::transport::Channel>>);

impl Client {
	pub fn new(inner: IndexerManagerClient<tonic::transport::Channel>) -> Self {
		Self(RwLock::new(inner))
	}

	pub async fn default() -> Result<Self, Error> {
		let inner = IndexerManagerClient::connect(apibara_url())
			.await
			.map_err(|error| Error::from(error))?;
		Ok(Self::new(inner))
	}
}

fn apibara_url() -> String {
	std::env::var("APIBARA_URL").expect("APIBARA_URL must be set")
}
