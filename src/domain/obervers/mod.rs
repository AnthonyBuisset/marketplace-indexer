mod composite;
pub use composite::ObserverComposite;

mod logging;
pub use logging::Logger;

use crate::domain::*;
use mockall::automock;

#[automock]
pub trait Observer {
	fn on_connect(&self);
	fn on_new_event(&self, event: Event);
	fn on_new_block(&self);
	fn on_reorg(&self);
}
