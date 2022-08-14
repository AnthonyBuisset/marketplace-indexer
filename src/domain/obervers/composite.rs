use super::*;

pub struct ObserverComposite(Vec<Box<dyn Observer>>);

impl ObserverComposite {
	pub fn new(observers: Vec<Box<dyn Observer>>) -> Self {
		Self(observers)
	}
}

impl Observer for ObserverComposite {
	fn on_new_event(&self, event: Event) {
		self.0.iter().for_each(|observer| observer.on_new_event(event.clone()))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use mockall::predicate::*;

	#[test]
	fn handle_event() {
		let event = Event;

		let mut handler1 = MockObserver::new();
		handler1.expect_on_new_event().with(eq(event.clone())).return_const(());

		let mut handler2 = MockObserver::new();
		handler2.expect_on_new_event().with(eq(event.clone())).return_const(());

		let composite = ObserverComposite::new(vec![Box::new(handler1), Box::new(handler2)]);
		composite.on_new_event(event);
	}
}
