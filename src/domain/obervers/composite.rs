use std::sync::Arc;

use super::*;

pub struct ObserverComposite(Vec<Arc<dyn Observer>>);

impl ObserverComposite {
	pub fn new(observers: Vec<Arc<dyn Observer>>) -> Self {
		Self(observers)
	}
}

impl Observer for ObserverComposite {
	fn on_connect(&self, indexer_id: IndexerId) {
		self.0.iter().for_each(|observer| observer.on_connect(indexer_id.clone()))
	}

	fn on_new_event(&self, event: Event) {
		self.0.iter().for_each(|observer| observer.on_new_event(event.clone()))
	}

	fn on_new_block(&self) {
		self.0.iter().for_each(|observer| observer.on_new_block())
	}

	fn on_reorg(&self) {
		self.0.iter().for_each(|observer| observer.on_reorg())
	}

	fn on_error(&self, error: Arc<dyn std::error::Error>) {
		self.0.iter().for_each(|observer| observer.on_error(error.clone()))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use mockall::predicate::*;

	#[test]
	fn on_new_event() {
		let event = Event;

		let mut observer1 = MockObserver::new();
		observer1.expect_on_new_event().with(eq(event.clone())).return_const(());

		let mut observer2 = MockObserver::new();
		observer2.expect_on_new_event().with(eq(event.clone())).return_const(());

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_new_event(event);
	}

	#[test]
	fn on_connect() {
		let mut observer1 = MockObserver::new();
		observer1.expect_on_connect().with(eq(IndexerId::from("ID"))).return_const(());

		let mut observer2 = MockObserver::new();
		observer2.expect_on_connect().with(eq(IndexerId::from("ID"))).return_const(());

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_connect(IndexerId::from("ID"));
	}

	#[test]
	fn on_new_block() {
		let mut observer1 = MockObserver::new();
		observer1.expect_on_new_block().return_const(());

		let mut observer2 = MockObserver::new();
		observer2.expect_on_new_block().return_const(());

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_new_block();
	}

	#[test]
	fn on_reorg() {
		let mut observer1 = MockObserver::new();
		observer1.expect_on_reorg().return_const(());

		let mut observer2 = MockObserver::new();
		observer2.expect_on_reorg().return_const(());

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_reorg();
	}

	#[test]
	fn on_error() {
		use thiserror::Error;
		#[derive(Debug, Error, PartialEq, Eq)]
		#[error("oops")]
		struct Error;

		let mut observer1 = MockObserver::new();
		observer1.expect_on_error().return_const(());

		let mut observer2 = MockObserver::new();
		observer2.expect_on_error().return_const(());

		let composite = ObserverComposite::new(vec![Arc::new(observer1), Arc::new(observer2)]);
		composite.on_error(Arc::new(Error));
	}
}
