use crate::domain::*;
use mockall::automock;

#[automock]
pub trait Service {
	fn fetch_new_events(&self, indexer: &Indexer);
}
