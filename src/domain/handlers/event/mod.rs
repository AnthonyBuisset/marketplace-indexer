use mockall::automock;

mod composite;
pub use composite::HandlerComposite;

mod logging;
pub use logging::LoggingHandler;

use crate::domain::*;

#[automock]
pub trait Handler {
	fn handle_event(&self, event: Event);
}
