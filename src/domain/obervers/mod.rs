mod composite;
use std::sync::Arc;

pub use composite::ObserverComposite;

mod logging;
pub use logging::Logger;

use crate::domain::*;
use mockall::automock;

#[automock]
pub trait Observer: Send + Sync {
	fn on_connect(&self, indexer_id: IndexerId);
	fn on_new_event(&self, event: Event);
	fn on_new_block(&self);
	fn on_reorg(&self);
	fn on_error(&self, error: Arc<dyn std::error::Error>);
}
