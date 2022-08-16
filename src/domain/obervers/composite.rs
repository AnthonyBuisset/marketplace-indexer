use super::*;
use futures::future::join_all;
use std::sync::Arc;

pub struct ObserverComposite(Vec<Arc<dyn Observer>>);

impl ObserverComposite {
	pub fn new(observers: Vec<Arc<dyn Observer>>) -> Self {
		Self(observers)
	}
}

#[async_trait]
impl Observer for ObserverComposite {
	async fn on_connect(&self, indexer_id: IndexerId) {
		let handles = self.0.iter().map(|observer| observer.on_connect(indexer_id.clone()));
		join_all(handles).await;
	}

	async fn on_new_event(&self, event: Event) {
		let handles = self.0.iter().map(|observer| observer.on_new_event(event.clone()));
		join_all(handles).await;
	}

	async fn on_new_block(&self, block_hash: BlockHash) {
		let handles = self.0.iter().map(|observer| observer.on_new_block(block_hash.clone()));
		join_all(handles).await;
	}

	async fn on_reorg(&self) {
		let handles = self.0.iter().map(|observer| observer.on_reorg());
		join_all(handles).await;
	}

	async fn on_error(&self, error: Arc<dyn std::error::Error + Send + Sync>) {
		let handles = self.0.iter().map(|observer| observer.on_error(error.clone()));
		join_all(handles).await;
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use super::*;
	use mockall::predicate::*;

	#[tokio::test]
	async fn on_new_event() {
		let event = Event;

		let mut observer1 = MockObserver::new();
		observer1
			.expect_on_new_event()
			.with(eq(event.clone()))
			.returning(|_| Box::pin(async {}));

		let mut observer2 = MockObserver::new();
		observer2
			.expect_on_new_event()
			.with(eq(event.clone()))
			.returning(|_| Box::pin(async {}));

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_new_event(event).await;
	}

	#[tokio::test]
	async fn on_connect() {
		let mut observer1 = MockObserver::new();
		observer1
			.expect_on_connect()
			.with(eq(IndexerId::from("ID")))
			.returning(|_| Box::pin(async {}));

		let mut observer2 = MockObserver::new();
		observer2
			.expect_on_connect()
			.with(eq(IndexerId::from("ID")))
			.returning(|_| Box::pin(async {}));

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_connect(IndexerId::from("ID")).await;
	}

	#[tokio::test]
	async fn on_new_block() {
		let block_hash = BlockHash::from_str("0x1234").unwrap();

		let mut observer1 = MockObserver::new();
		observer1
			.expect_on_new_block()
			.with(eq(block_hash.clone()))
			.returning(|_| Box::pin(async {}));

		let mut observer2 = MockObserver::new();
		observer2
			.expect_on_new_block()
			.with(eq(block_hash.clone()))
			.returning(|_| Box::pin(async {}));

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_new_block(block_hash).await;
	}

	#[tokio::test]
	async fn on_reorg() {
		let mut observer1 = MockObserver::new();
		observer1.expect_on_reorg().returning(|| Box::pin(async {}));

		let mut observer2 = MockObserver::new();
		observer2.expect_on_reorg().returning(|| Box::pin(async {}));

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_reorg().await;
	}

	#[tokio::test]
	async fn on_error() {
		use thiserror::Error;
		#[derive(Debug, Error, PartialEq, Eq)]
		#[error("oops")]
		struct Error;

		let mut observer1 = MockObserver::new();
		observer1.expect_on_error().returning(|_| Box::pin(async {}));

		let mut observer2 = MockObserver::new();
		observer2.expect_on_error().returning(|_| Box::pin(async {}));

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_error(Arc::new(Error)).await;
	}
}
