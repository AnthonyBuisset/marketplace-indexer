//! The streaming request will stream messages to be sent to the indexing server
use super::apibara::{
	connect_indexer_request::Message, AckBlock, ConnectIndexer, ConnectIndexerRequest,
};
use crate::domain::*;
use async_trait::async_trait;
use futures::{lock::Mutex, FutureExt, Stream};
use std::{collections::VecDeque, task::Poll};

#[derive(Default)]
pub struct StreamingRequest {
	indexer_id: String,
	blocks: Mutex<VecDeque<BlockHash>>,
}

impl StreamingRequest {
	pub fn new(indexer_id: String) -> Self {
		Self {
			indexer_id,
			..Default::default()
		}
	}

	fn connect(id: String) -> ConnectIndexerRequest {
		ConnectIndexerRequest {
			message: Some(Message::Connect(ConnectIndexer { id })),
		}
	}

	fn ack_block(block_hash: BlockHash) -> ConnectIndexerRequest {
		ConnectIndexerRequest {
			message: Some(Message::Ack(AckBlock {
				hash: block_hash.bytes(),
			})),
		}
	}
}

#[async_trait]
impl BlockchainObserver for StreamingRequest {
	async fn on_new_block(&self, block_hash: BlockHash) {
		// On new block received, we will save the block hash to acknoledge it during the next
		// stream call
		self.blocks.lock().await.push_front(block_hash);
	}

	async fn on_connect(&self, _indexer_id: IndexerId) {}

	async fn on_new_event(&self, _event: Event) {}

	async fn on_reorg(&self) {}

	async fn on_error(&self, _error: std::sync::Arc<dyn std::error::Error + Send + Sync>) {}
}

impl Stream for StreamingRequest {
	type Item = ConnectIndexerRequest;

	/**
	 * Stream a request
	 * 1. Acquire a lock on the inner block hashes to be achnoledged
	 * 2. If a block needs to be acknoledged, return an Ack request
	 * 3. If no block needs to be acknoledged, return a Connect request
	 */
	fn poll_next(
		self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> Poll<Option<Self::Item>> {
		match self.blocks.lock().boxed().poll_unpin(cx) {
			Poll::Ready(mut blocks) =>
				if let Some(block_hash) = blocks.pop_back() {
					Poll::Ready(Some(Self::ack_block(block_hash)))
				} else {
					Poll::Ready(Some(Self::connect(self.indexer_id.clone())))
				},
			Poll::Pending => Poll::Pending,
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(1, None)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use futures::StreamExt;
	use rstest::*;

	#[fixture]
	fn indexer_id() -> String {
		String::from("ID")
	}

	#[fixture]
	fn block_hash() -> BlockHash {
		vec![12].into()
	}

	#[fixture]
	fn connect_request(indexer_id: String) -> ConnectIndexerRequest {
		ConnectIndexerRequest {
			message: Some(Message::Connect(ConnectIndexer { id: indexer_id })),
		}
	}

	#[fixture]
	fn ack_request(block_hash: BlockHash) -> ConnectIndexerRequest {
		ConnectIndexerRequest {
			message: Some(Message::Ack(AckBlock {
				hash: block_hash.bytes(),
			})),
		}
	}

	#[fixture]
	fn streaming_request(indexer_id: String) -> StreamingRequest {
		StreamingRequest::new(indexer_id)
	}

	#[rstest]
	fn streaming_request_can_connect_an_indexer(connect_request: ConnectIndexerRequest) {
		assert_eq!(
			connect_request,
			StreamingRequest::connect(String::from("ID"))
		);
	}

	#[rstest]
	fn streaming_request_can_acknoledge_a_block(
		ack_request: ConnectIndexerRequest,
		block_hash: BlockHash,
	) {
		assert_eq!(ack_request, StreamingRequest::ack_block(block_hash));
	}

	#[rstest]
	#[tokio::test]
	async fn streaming_request_can_stream(
		connect_request: ConnectIndexerRequest,
		mut streaming_request: StreamingRequest,
	) {
		assert_eq!(Some(connect_request), streaming_request.next().await);
	}

	#[rstest]
	#[tokio::test]
	async fn streaming_request_ack_block_when_needed(
		connect_request: ConnectIndexerRequest,
		ack_request: ConnectIndexerRequest,
		block_hash: BlockHash,
		mut streaming_request: StreamingRequest,
	) {
		assert_eq!(
			Some(connect_request.clone()),
			streaming_request.next().await
		);
		streaming_request.on_new_block(block_hash).await;

		assert_eq!(Some(ack_request), streaming_request.next().await);
		assert_eq!(Some(connect_request), streaming_request.next().await);
	}
}
