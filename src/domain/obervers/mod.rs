mod composite;
pub use composite::ObserverComposite;

mod logging;
pub use logging::Logger;

use crate::domain::*;
use mockall::automock;

#[automock]
pub trait Observer: Send + Sync {
	fn on_connect(&self, indexer_id: &IndexerId);
	fn on_new_event(&self, event: &Event);
	fn on_new_block(&self, block_hash: &BlockHash);
	fn on_reorg(&self);
}
