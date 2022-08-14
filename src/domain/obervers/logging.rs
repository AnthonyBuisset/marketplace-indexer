use log::info;

use super::*;

type LoggingCallback = dyn Fn(String) + Sync;
pub struct Logger<'a>(&'a LoggingCallback);

impl<'a> Logger<'a> {
	pub fn new(logger: &'a LoggingCallback) -> Self {
		Self(logger)
	}
}

impl Observer for Logger<'_> {
	fn on_connect(&self, indexer_id: IndexerId) {
		self.0(format!("Indexer `{indexer_id}` connected"));
	}

	fn on_new_event(&self, event: Event) {
		self.0(format!("New event: {:?}", event));
	}

	fn on_new_block(&self) {
		self.0("New block".to_string());
	}

	fn on_reorg(&self) {
		self.0("Chain reorg".to_string());
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
	fn on_new_event() {
		let mut logger = MockLoggerCallback::new();
		logger.expect_log().with(eq(String::from("New event: Event"))).return_const(());
		let logging_callback = move |message| logger.log(message);

		let event = Event;
		let handler = Logger::new(&logging_callback);
		handler.on_new_event(event);
	}

	#[test]
	fn on_connect() {
		let mut logger = MockLoggerCallback::new();
		logger
			.expect_log()
			.with(eq(String::from("Indexer `ID` connected")))
			.return_const(());
		let logging_callback = move |message| logger.log(message);

		let handler = Logger::new(&logging_callback);
		handler.on_connect(IndexerId::from("ID"));
	}

	#[test]
	fn on_new_block() {
		let mut logger = MockLoggerCallback::new();
		logger.expect_log().with(eq(String::from("New block"))).return_const(());
		let logging_callback = move |message| logger.log(message);

		let handler = Logger::new(&logging_callback);
		handler.on_new_block();
	}

	#[test]
	fn on_reorg() {
		let mut logger = MockLoggerCallback::new();
		logger.expect_log().with(eq(String::from("Chain reorg"))).return_const(());
		let logging_callback = move |message| logger.log(message);

		let handler = Logger::new(&logging_callback);
		handler.on_reorg();
	}

	#[test]
	fn handler_can_be_created_using_default() {
		let handler = Logger::default();
		handler.on_new_event(Event);
	}
}
