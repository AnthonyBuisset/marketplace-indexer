use log::info;

use super::*;

type LoggingCallback = dyn Fn(String) + Sync;
pub struct Logger<'a>(&'a LoggingCallback);

impl<'a> Logger<'a> {
	pub fn new(logger: &'a LoggingCallback) -> Self {
		Self(logger)
	}
}

#[async_trait]
impl Observer for Logger<'_> {
	async fn on_connect(&self, indexer_id: IndexerId) {
		self.0(format!("Indexer `{indexer_id}` connected"));
	}

	async fn on_new_event(&self, event: Event) {
		self.0(format!("⚡ New event: {:?}", event));
	}

	async fn on_new_block(&self, block_hash: BlockHash) {
		self.0(format!("⛏️ New block: {block_hash}"));
	}

	async fn on_reorg(&self) {
		self.0("🤕 Chain reorg".to_string());
	}

	async fn on_error(&self, error: Arc<dyn std::error::Error + Send + Sync>) {
		self.0(format!(
			"❌ Error while fetching messages from indexing server: {error}"
		));
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
	use rstest::*;
	use std::str::FromStr;

	#[automock]
	trait LoggerCallback {
		fn log(&self, message: String);
	}

	#[fixture]
	fn logger() -> MockLoggerCallback {
		MockLoggerCallback::new()
	}

	#[rstest]
	#[tokio::test]
	async fn on_new_event(mut logger: MockLoggerCallback) {
		logger
			.expect_log()
			.with(eq(String::from("⚡ New event: Event")))
			.return_const(());
		let logging_callback = move |message| logger.log(message);

		let event = Event;
		let handler = Logger::new(&logging_callback);
		handler.on_new_event(event).await;
	}

	#[rstest]
	#[tokio::test]
	async fn on_connect(mut logger: MockLoggerCallback) {
		logger
			.expect_log()
			.with(eq(String::from("Indexer `ID` connected")))
			.return_const(());
		let logging_callback = move |message| logger.log(message);

		let handler = Logger::new(&logging_callback);
		handler.on_connect(IndexerId::from("ID")).await;
	}

	#[rstest]
	#[tokio::test]
	async fn on_new_block(mut logger: MockLoggerCallback) {
		logger
			.expect_log()
			.with(eq(String::from("⛏️ New block: 0x1234")))
			.return_const(());
		let logging_callback = move |message| logger.log(message);

		let handler = Logger::new(&logging_callback);
		handler.on_new_block(BlockHash::from_str("0x1234").unwrap()).await;
	}

	#[rstest]
	#[tokio::test]
	async fn on_reorg(mut logger: MockLoggerCallback) {
		logger.expect_log().with(eq(String::from("🤕 Chain reorg"))).return_const(());
		let logging_callback = move |message| logger.log(message);

		let handler = Logger::new(&logging_callback);
		handler.on_reorg().await;
	}

	#[rstest]
	#[tokio::test]
	async fn on_error(mut logger: MockLoggerCallback) {
		use thiserror::Error;
		#[derive(Debug, Error)]
		#[error("oops")]
		struct Error;

		logger
			.expect_log()
			.with(eq(String::from(
				"❌ Error while fetching messages from indexing server: oops",
			)))
			.return_const(());
		let logging_callback = move |message| logger.log(message);

		let handler = Logger::new(&logging_callback);
		handler.on_error(Arc::new(Error)).await;
	}

	#[tokio::test]
	async fn handler_can_be_created_using_default() {
		let handler = Logger::default();
		handler.on_new_event(Event).await;
	}
}
