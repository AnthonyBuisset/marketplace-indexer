use mockall::automock;
use crate::domain::*;

#[automock]
pub trait Service {
	fn fetch_new_events(&self, indexer: &Indexer);
}
