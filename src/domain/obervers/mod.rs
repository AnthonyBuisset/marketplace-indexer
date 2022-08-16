mod composite;
use std::sync::Arc;

use async_trait::async_trait;
pub use composite::ObserverComposite;

mod logging;
pub use logging::Logger;

use crate::domain::*;
use mockall::automock;

#[async_trait]
#[automock]
pub trait Observer: Send + Sync {
	async fn on_connect(&self, indexer_id: IndexerId);
	async fn on_new_event(&self, event: Event);
	async fn on_new_block(&self, block_hash: BlockHash);
	async fn on_reorg(&self);
	async fn on_error(&self, error: Arc<dyn std::error::Error + Send + Sync>);
}
