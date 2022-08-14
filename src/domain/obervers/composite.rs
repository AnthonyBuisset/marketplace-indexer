use super::*;

pub struct ObserverComposite(Vec<Box<dyn Observer>>);

impl ObserverComposite {
	pub fn new(observers: Vec<Box<dyn Observer>>) -> Self {
		Self(observers)
	}
}

impl Observer for ObserverComposite {
	fn on_connect(&self) {
		self.0.iter().for_each(|observer| observer.on_connect())
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

		let composite = ObserverComposite::new(vec![Box::new(observer1), Box::new(observer2)]);
		composite.on_new_event(event);
	}

	#[test]
	fn on_connect() {
		let mut observer1 = MockObserver::new();
		observer1.expect_on_connect().return_const(());

		let mut observer2 = MockObserver::new();
		observer2.expect_on_connect().return_const(());

		let composite = ObserverComposite::new(vec![Box::new(observer1), Box::new(observer2)]);
		composite.on_connect();
	}

	#[test]
	fn on_new_block() {
		let mut observer1 = MockObserver::new();
		observer1.expect_on_new_block().return_const(());

		let mut observer2 = MockObserver::new();
		observer2.expect_on_new_block().return_const(());

		let composite = ObserverComposite::new(vec![Box::new(observer1), Box::new(observer2)]);
		composite.on_new_block();
	}

	#[test]
	fn on_reorg() {
		let mut observer1 = MockObserver::new();
		observer1.expect_on_reorg().return_const(());

		let mut observer2 = MockObserver::new();
		observer2.expect_on_reorg().return_const(());

		let composite = ObserverComposite::new(vec![Box::new(observer1), Box::new(observer2)]);
		composite.on_reorg();
	}
}
