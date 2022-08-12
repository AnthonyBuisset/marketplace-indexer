use async_trait::async_trait;
use futures::{stream, StreamExt, TryStreamExt};

use super::{
	apibara::{
		connect_indexer_request, connect_indexer_response::Message, ConnectIndexer,
		ConnectIndexerRequest,
	},
	*,
};
use crate::domain::*;

#[async_trait]
impl IndexingService for Client {
	async fn fetch_new_events(&self, indexer: &Indexer) -> Result<(), IndexingServiceError> {
		let request = ConnectIndexerRequest {
			message: Some(connect_indexer_request::Message::Connect(ConnectIndexer {
				id: indexer.id.to_string(),
			})),
		};

		self.0
			.write()
			.await
			.connect_indexer(stream::once(async { request }))
			.await
			.map_err(|e| IndexingServiceError::Connection {
				id: indexer.id.clone(),
				details: e.to_string(),
			})?
			.into_inner()
			.try_filter_map(|msg| async {
				match msg.message {
					Some(Message::Connected(msg)) => {
						println!("Indexer connected: {:?}", msg)
					},
					Some(Message::NewBlock(msg)) => {
						println!("New block: {:?}", msg)
					},
					Some(Message::Reorg(msg)) => {
						println!("Reorg: {:?}", msg)
					},
					Some(Message::NewEvents(msg)) => {
						println!("New events: {:?}", msg)
					},
					None => {
						println!("Empty message received")
					},
				};

				Ok(Some(()))
			})
			.collect::<Vec<_>>()
			.await;

		Ok(())
	}
}
