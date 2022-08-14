use async_trait::async_trait;
use futures::{stream, StreamExt};
use std::sync::Arc;

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
	async fn fetch_new_events(
		&self,
		indexer: &Indexer,
		observer: Arc<dyn BlockchainObserver>,
	) -> Result<(), IndexingServiceError> {
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
			.for_each(|msg| async {
				match msg {
					Ok(msg) => match msg.message {
						Some(Message::Connected(msg)) =>
							observer.on_connect(msg.indexer.unwrap_or_default().id.into()),
						Some(Message::NewBlock(_)) => observer.on_new_block(),
						Some(Message::Reorg(_)) => observer.on_reorg(),
						Some(Message::NewEvents(_)) => observer.on_new_event(Event),
						None => {
							println!("Empty message received")
						},
					},
					Err(error) => observer.on_error(Arc::new(error)),
				};
			})
			.await;

		Ok(())
	}
}
