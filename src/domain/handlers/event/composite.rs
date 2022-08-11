use super::*;

pub struct HandlerComposite(Vec<Box<dyn Handler>>);

impl HandlerComposite {
	pub fn new(handlers: Vec<Box<dyn Handler>>) -> Self {
		Self(handlers)
	}
}

impl Handler for HandlerComposite {
	fn handle_event(&self, event: Event) {
		self.0.iter().for_each(|handler| handler.handle_event(event.clone()))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use mockall::predicate::*;

	#[test]
	fn handle_event() {
		let event = Event;

		let mut handler1 = MockHandler::new();
		handler1.expect_handle_event().with(eq(event.clone())).return_const(());

		let mut handler2 = MockHandler::new();
		handler2.expect_handle_event().with(eq(event.clone())).return_const(());

		let composite = HandlerComposite::new(vec![Box::new(handler1), Box::new(handler2)]);
		composite.handle_event(event);
	}
}
