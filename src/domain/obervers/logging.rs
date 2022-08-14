use log::info;

use super::*;

type LoggingCallback = dyn Fn(String);
pub struct Logger<'a>(&'a LoggingCallback);

impl<'a> Logger<'a> {
	pub fn new(logger: &'a LoggingCallback) -> Self {
		Self(logger)
	}
}

impl Observer for Logger<'_> {
	fn on_new_event(&self, event: Event) {
		self.0(format!("Received new event: {:?}", event));
	}
}

impl Default for Logger<'_> {
	fn default() -> Self {
		Self(&|message| info!("{}", message))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use mockall::predicate::*;

	#[automock]
	trait LoggerCallback {
		fn log(&self, message: String);
	}

	#[test]
	fn handle_event() {
		let mut logger = MockLoggerCallback::new();
		logger
			.expect_log()
			.with(eq(String::from("Received event: Event")))
			.return_const(());
		let logging_callback = move |message| logger.log(message);

		let event = Event;
		let handler = Logger::new(&logging_callback);
		handler.on_new_event(event);
	}

	#[test]
	fn handler_can_be_created_using_default() {
		let handler = Logger::default();
		handler.on_new_event(Event);
	}
}
