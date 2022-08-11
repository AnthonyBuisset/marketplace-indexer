use log::info;

use super::*;

type Logger = dyn Fn(String);
pub struct LoggingHandler<'a>(&'a Logger);

impl<'a> LoggingHandler<'a> {
	pub fn new(logger: &'a Logger) -> Self {
		Self(logger)
	}
}

impl Handler for LoggingHandler<'_> {
	fn handle_event(&self, event: Event) {
		self.0(format!("Received event: {:?}", event));
	}
}

impl Default for LoggingHandler<'_> {
	fn default() -> Self {
		Self(&|message| info!("{}", message))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use mockall::predicate::*;

	#[automock]
	trait Logger {
		fn log(&self, message: String);
	}

	#[test]
	fn handle_event() {
		let mut logger = MockLogger::new();
		logger
			.expect_log()
			.with(eq(String::from("Received event: Event")))
			.return_const(());
		let logging_callback = move |message| logger.log(message);

		let event = Event;
		let handler = LoggingHandler::new(&logging_callback);
		handler.handle_event(event);
	}

	#[test]
	fn handler_can_be_created_using_default() {
		let handler = LoggingHandler::default();
		handler.handle_event(Event);
	}
}
