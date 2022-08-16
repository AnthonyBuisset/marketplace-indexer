use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;

use super::{
	apibara::{
		connect_indexer_request::Message as RequestMessage,
		connect_indexer_response::Message as ResponseMessage, AckBlock, ConnectIndexer,
		ConnectIndexerRequest, ConnectIndexerResponse, IndexerConnected, NewBlock,
	},
	*,
};
use crate::domain::*;

#[async_trait]
impl IndexingService for Client {
	async fn fetch_new_events(
		&self,
		indexer: &Indexer,
		observer: Arc<dyn BlockchainObserver>,
	) -> Result<(), IndexingServiceError> {
		let channel = Channel::new();
		send_connect_request(&channel.tx, &indexer.id).await?;

		let mut response_stream = self
			.0
			.write()
			.await
			.connect_indexer(ReceiverStream::new(channel.rx))
			.await
			.map_err(|e| IndexingServiceError::Connection {
				id: indexer.id.clone(),
				details: e.to_string(),
			})?
			.into_inner();

		loop {
			let message = response_stream
				.message()
				.await
				.map_err(|error| IndexingServiceError::Receive(error.to_string()))?;
			if let Some(message) = message {
				handle_response(message, &channel.tx, &*observer).await?;
			}
		}
	}
}

struct Channel {
	tx: Sender<ConnectIndexerRequest>,
	rx: Receiver<ConnectIndexerRequest>,
}

impl Channel {
	pub fn new() -> Self {
		let (tx, rx) = mpsc::channel(64);
		Self { tx, rx }
	}
}

async fn send_connect_request(
	sender: &Sender<ConnectIndexerRequest>,
	indexer_id: &IndexerId,
) -> Result<(), IndexingServiceError> {
	send(sender, connect_request(indexer_id)).await
}

async fn send_ack_request(
	sender: &Sender<ConnectIndexerRequest>,
	block_hash: &BlockHash,
) -> Result<(), IndexingServiceError> {
	send(sender, ack_block(block_hash)).await
}

async fn send<T>(sender: &Sender<T>, request: T) -> Result<(), IndexingServiceError> {
	sender
		.send(request)
		.await
		.map_err(|error| IndexingServiceError::Send(error.to_string()))
}

fn connect_request(indexer_id: &IndexerId) -> ConnectIndexerRequest {
	ConnectIndexerRequest {
		message: Some(RequestMessage::Connect(ConnectIndexer {
			id: indexer_id.to_string(),
		})),
	}
}

fn ack_block(block_hash: &BlockHash) -> ConnectIndexerRequest {
	ConnectIndexerRequest {
		message: Some(RequestMessage::Ack(AckBlock {
			hash: block_hash.bytes(),
		})),
	}
}

async fn handle_response(
	response: ConnectIndexerResponse,
	sender: &Sender<ConnectIndexerRequest>,
	observer: &dyn BlockchainObserver,
) -> Result<(), IndexingServiceError> {
	match response.message {
		Some(ResponseMessage::Connected(IndexerConnected {
			indexer: Some(indexer),
			version: _,
		})) => Ok(observer.on_connect(indexer.id.into())),

		Some(ResponseMessage::NewBlock(NewBlock {
			new_head: Some(new_head),
		})) => {
			let block_hash = BlockHash::from(new_head.hash);
			send_ack_request(sender, &block_hash).await?;
			Ok(observer.on_new_block(block_hash))
		},

		Some(ResponseMessage::Reorg(_)) => Ok(observer.on_reorg()),

		Some(ResponseMessage::NewEvents(_)) => Ok(observer.on_new_event(Event)),

		_ => Ok(()),
	}
}

#[cfg(test)]
mod test {
	use super::{apibara::BlockHeader, *};
	use mockall::predicate::*;
	use rstest::*;
	use tokio::sync::mpsc::error::TryRecvError;

	#[fixture]
	fn indexer_id() -> IndexerId {
		IndexerId::from("ID")
	}

	#[fixture]
	fn block_hash() -> BlockHash {
		vec![12].into()
	}

	#[fixture]
	fn channel() -> Channel {
		Channel::new()
	}

	#[fixture]
	fn observer() -> MockBlockchainObserver {
		MockBlockchainObserver::new()
	}

	#[rstest]
	#[tokio::test]
	async fn can_send_a_connect_request(mut channel: Channel, indexer_id: IndexerId) {
		send_connect_request(&channel.tx, &indexer_id).await.unwrap();
		let request = channel.rx.recv().await.unwrap();
		assert_eq!(
			RequestMessage::Connect(ConnectIndexer {
				id: indexer_id.to_string(),
			}),
			request.message.unwrap()
		);
	}

	#[rstest]
	#[tokio::test]
	async fn can_handle_a_connect_response(
		indexer_id: IndexerId,
		mut channel: Channel,
		mut observer: MockBlockchainObserver,
	) {
		let response = ConnectIndexerResponse {
			message: Some(ResponseMessage::Connected(IndexerConnected {
				indexer: Some(apibara::Indexer {
					id: indexer_id.to_string(),
					..Default::default()
				}),
				..Default::default()
			})),
		};

		observer.expect_on_connect().return_const(());

		let result = handle_response(response, &channel.tx, &observer).await;
		assert!(result.is_ok(), "{}", result.err().unwrap());
		assert_eq!(TryRecvError::Empty, channel.rx.try_recv().unwrap_err());
	}

	#[rstest]
	#[tokio::test]
	async fn can_handle_a_new_block_response(
		block_hash: BlockHash,
		mut channel: Channel,
		mut observer: MockBlockchainObserver,
	) {
		let response = ConnectIndexerResponse {
			message: Some(ResponseMessage::NewBlock(NewBlock {
				new_head: Some(BlockHeader {
					hash: block_hash.bytes(),
					..Default::default()
				}),
			})),
		};

		observer.expect_on_new_block().with(eq(block_hash.clone())).return_const(());

		let result = handle_response(response, &channel.tx, &observer).await;
		assert!(result.is_ok(), "{}", result.err().unwrap());

		let request = channel.rx.try_recv().unwrap();
		assert_eq!(
			RequestMessage::Ack(AckBlock {
				hash: block_hash.bytes(),
			}),
			request.message.unwrap()
		);
	}

	#[rstest]
	#[tokio::test]
	async fn can_handle_a_new_event_response(
		mut channel: Channel,
		mut observer: MockBlockchainObserver,
	) {
		let response = ConnectIndexerResponse {
			message: Some(ResponseMessage::NewEvents(apibara::NewEvents::default())),
		};

		observer.expect_on_new_event().return_const(());

		let result = handle_response(response, &channel.tx, &observer).await;
		assert!(result.is_ok(), "{}", result.err().unwrap());
		assert_eq!(TryRecvError::Empty, channel.rx.try_recv().unwrap_err());
	}

	#[rstest]
	#[tokio::test]
	async fn can_handle_a_new_reorg_response(
		mut channel: Channel,
		mut observer: MockBlockchainObserver,
	) {
		let response = ConnectIndexerResponse {
			message: Some(ResponseMessage::Reorg(apibara::Reorg::default())),
		};

		observer.expect_on_reorg().return_const(());

		let result = handle_response(response, &channel.tx, &observer).await;
		assert!(result.is_ok(), "{}", result.err().unwrap());
		assert_eq!(TryRecvError::Empty, channel.rx.try_recv().unwrap_err());
	}
}
