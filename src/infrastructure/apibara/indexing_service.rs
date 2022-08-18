use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;

use super::{
	apibara::{
		connect_indexer_request::Message as RequestMessage,
		connect_indexer_response::Message as ResponseMessage, AckBlock, ConnectIndexer,
		ConnectIndexerRequest, ConnectIndexerResponse, IndexerConnected, NewBlock, NewEvents,
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
			match response_stream
				.message()
				.await
				.map_err(|error| IndexingServiceError::Receive(error.to_string()))?
			{
				Some(response) => handle_response(response, &channel.tx, &*observer).await?,
				None => continue,
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
		})) => {
			observer.on_connect(&indexer.id.into());
			Ok(())
		},

		Some(ResponseMessage::NewBlock(NewBlock {
			new_head: Some(new_head),
		})) => {
			let block_hash = BlockHash::from(new_head.hash);
			observer.on_new_block(&block_hash);
			Ok(())
		},

		Some(ResponseMessage::Reorg(_)) => {
			observer.on_reorg();
			Ok(())
		},

		Some(ResponseMessage::NewEvents(NewEvents { block, events })) => {
			events.into_iter().for_each(|event| {
				if let Some(event) = event.try_into().ok() {
					observer.on_new_event(&event);
				}
			});

			match block {
				Some(header) => send_ack_request(sender, &header.hash.into()).await,
				_ => Ok(()),
			}
		},

		_ => Ok(()),
	}
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FromEventError {
	#[error("Unsupported event")]
	Unsupported,
}

impl TryFrom<apibara::Event> for Event {
	type Error = FromEventError;

	fn try_from(event: apibara::Event) -> Result<Self, Self::Error> {
		match event.event {
			Some(apibara::event::Event::Starknet(apibara::StarkNetEvent {
				address,
				log_index,
				topics,
				data,
			})) => {
				info!(
					"New Event from {}: [{log_index}] {:?}. Data: {:?}",
					ContractAddress::from(address),
					topics,
					data
				);
				Ok(Event)
			},
			_ => Err(Self::Error::Unsupported),
		}
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

	#[fixture]
	fn apibara_event() -> apibara::Event {
		apibara::Event {
			event: Some(apibara::event::Event::Starknet(apibara::StarkNetEvent {
				..Default::default()
			})),
		}
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
		assert_eq!(TryRecvError::Empty, channel.rx.try_recv().unwrap_err());
	}

	#[rstest]
	#[tokio::test]
	async fn can_handle_a_new_events_response(
		mut channel: Channel,
		mut observer: MockBlockchainObserver,
		apibara_event: apibara::Event,
		block_hash: BlockHash,
	) {
		let response = ConnectIndexerResponse {
			message: Some(ResponseMessage::NewEvents(apibara::NewEvents {
				block: Some(BlockHeader {
					hash: block_hash.bytes(),
					..Default::default()
				}),
				events: vec![apibara_event.clone(), apibara_event, Default::default()],
			})),
		};

		observer.expect_on_new_event().times(2).return_const(());

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

	#[rstest]
	#[tokio::test]
	async fn can_handle_an_empty_response(mut channel: Channel, observer: MockBlockchainObserver) {
		let response = ConnectIndexerResponse { message: None };

		let result = handle_response(response, &channel.tx, &observer).await;
		assert!(result.is_ok(), "{}", result.err().unwrap());
		assert_eq!(TryRecvError::Empty, channel.rx.try_recv().unwrap_err());
	}
}
