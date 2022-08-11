use super::{
	apibara::{self, CreateIndexerRequest, DeleteIndexerRequest, GetIndexerRequest},
	Client,
};
use crate::domain::*;
use async_trait::async_trait;
use itertools::Itertools;

/**
 * Implementation of the Client trait for apibara
 */
#[async_trait]
impl IndexerRepository for Client {
	async fn create(&self, indexer: &Indexer) -> Result<(), IndexerRepositoryError> {
		let response = self
			.0
			.write()
			.await
			.create_indexer(CreateIndexerRequest {
				id: indexer.id.to_string(),
				network_name: indexer.network.to_string(),
				index_from_block: indexer.index_from_block,
				filters: indexer.clone().filters.into_iter().map_into().collect(),
			})
			.await
			.map_err(|status| IndexerRepositoryError::CreateIndexer {
				id: indexer.id.clone(),
				details: status.to_string(),
			})?;

		response
			.into_inner()
			.indexer
			.ok_or_else(|| IndexerRepositoryError::CreateIndexer {
				id: indexer.id.clone(),
				details: String::from("Indexer not created"),
			})?;

		Ok(())
	}

	async fn by_id(
		&self,
		indexer_id: &IndexerId,
	) -> Result<Option<Indexer>, IndexerRepositoryError> {
		let response = self
			.0
			.write()
			.await
			.get_indexer(GetIndexerRequest {
				id: indexer_id.to_string(),
			})
			.await
			.map_err(|status| IndexerRepositoryError::GetIndexer {
				id: indexer_id.clone(),
				details: status.to_string(),
			})?;

		Ok(response.into_inner().indexer.map(Indexer::from))
	}

	async fn delete(&self, indexer_id: &IndexerId) -> Result<(), IndexerRepositoryError> {
		self.0
			.write()
			.await
			.delete_indexer(DeleteIndexerRequest {
				id: indexer_id.to_string(),
			})
			.await
			.map_err(|status| IndexerRepositoryError::DeleteIndexer {
				id: indexer_id.clone(),
				details: status.to_string(),
			})?;

		Ok(())
	}
}

impl ToString for Network {
	fn to_string(&self) -> String {
		match self {
			Network::Starknet(chain) => chain.to_string(),
		}
	}
}

// Hardcoded strings are referenced in the server configuration.toml file
impl ToString for StarknetChain {
	fn to_string(&self) -> String {
		match self {
			StarknetChain::Devnet => "starknet-devnet",
			StarknetChain::Goerli => "starknet-goerli",
			StarknetChain::Mainnet => "starknet-mainnet",
		}
		.to_owned()
	}
}

impl From<EventFilter> for apibara::EventFilter {
	fn from(filter: EventFilter) -> Self {
		Self {
			address: filter.contract_address.to_string().into_bytes(),
			signature: filter.event_name,
		}
	}
}

impl From<apibara::Network> for Network {
	fn from(network: apibara::Network) -> Self {
		match network.network {
			Some(network) => match network {
				apibara::network::Network::Starknet(chain) => match chain.name {
					chain if chain == "starknet-devnet" => Network::Starknet(StarknetChain::Devnet),
					chain if chain == "starknet-goerli" => Network::Starknet(StarknetChain::Goerli),
					chain if chain == "starknet-mainnet" =>
						Network::Starknet(StarknetChain::Mainnet),
					_ => Network::Starknet(StarknetChain::Devnet),
				},
				apibara::network::Network::Ethereum(_) => unimplemented!(),
			},
			None => Network::Starknet(StarknetChain::Devnet),
		}
	}
}

impl From<apibara::EventFilter> for EventFilter {
	fn from(filter: apibara::EventFilter) -> Self {
		Self {
			// TODO: Implement From<Vec<u8>> for ContractAddress
			contract_address: String::from_utf8(filter.address).unwrap_or_default().into(),
			event_name: filter.signature,
		}
	}
}

impl From<apibara::Indexer> for Indexer {
	fn from(indexer: apibara::Indexer) -> Self {
		Self {
			id: indexer.id.into(),
			network: indexer
				.network
				.map(|network| network.into())
				.unwrap_or_else(|| Network::Starknet(StarknetChain::Devnet)),
			index_from_block: indexer.index_from_block,
			filters: indexer.filters.into_iter().map_into().collect(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rstest::*;

	#[test]
	fn network_to_apibara_string() {
		assert_eq!(
			"starknet-devnet",
			Network::Starknet(StarknetChain::Devnet).to_string()
		);

		assert_eq!(
			"starknet-goerli",
			Network::Starknet(StarknetChain::Goerli).to_string()
		);

		assert_eq!(
			"starknet-mainnet",
			Network::Starknet(StarknetChain::Mainnet).to_string()
		);
	}

	#[rstest]
	#[case(StarknetChain::Devnet, "starknet-devnet")]
	#[case(StarknetChain::Goerli, "starknet-goerli")]
	#[case(StarknetChain::Mainnet, "starknet-mainnet")]
	#[case(StarknetChain::Devnet, "non-existent")]
	fn network_from_apibara(#[case] chain: StarknetChain, #[case] network_name: &str) {
		assert_eq!(
			Network::Starknet(chain),
			Network::from(apibara::Network {
				network: Some(apibara::network::Network::Starknet(
					apibara::StarkNetNetwork {
						name: String::from(network_name)
					}
				))
			})
		)
	}

	#[test]
	fn network_from_apibara_should_default_to_starknet_devnet() {
		assert_eq!(
			Network::Starknet(StarknetChain::Devnet),
			Network::from(apibara::Network { network: None })
		);
	}

	#[test]
	#[should_panic(expected = "not implemented")]
	fn network_from_apibara_should_panic_for_unimplemented() {
		let _ = Network::from(apibara::Network {
			network: Some(apibara::network::Network::Ethereum(
				apibara::EthereumNetwork {
					name: String::from(""),
				},
			)),
		});
	}

	#[test]
	fn event_filter_from_domain() {
		let filter = apibara::EventFilter::from(EventFilter {
			contract_address: "0x04e16efc9bc2d8d40ecb73d3d69e3e2d6f0fc3e2e6e9b7601310fdfa7dd6c7cf"
				.into(),
			event_name: "GithubUserRegistered".to_owned(),
		});

		// TODO: Check this is correct with end-to-end testing
		assert_eq!(
			vec![
				48, 120, 48, 52, 101, 49, 54, 101, 102, 99, 57, 98, 99, 50, 100, 56, 100, 52, 48,
				101, 99, 98, 55, 51, 100, 51, 100, 54, 57, 101, 51, 101, 50, 100, 54, 102, 48, 102,
				99, 51, 101, 50, 101, 54, 101, 57, 98, 55, 54, 48, 49, 51, 49, 48, 102, 100, 102,
				97, 55, 100, 100, 54, 99, 55, 99, 102
			],
			filter.address
		);
		assert_eq!("GithubUserRegistered", filter.signature);
	}

	#[test]
	fn event_filter_from_apibara() {
		let filter = EventFilter::from(apibara::EventFilter {
			address: vec![
				48, 120, 48, 52, 101, 49, 54, 101, 102, 99, 57, 98, 99, 50, 100, 56, 100, 52, 48,
				101, 99, 98, 55, 51, 100, 51, 100, 54, 57, 101, 51, 101, 50, 100, 54, 102, 48, 102,
				99, 51, 101, 50, 101, 54, 101, 57, 98, 55, 54, 48, 49, 51, 49, 48, 102, 100, 102,
				97, 55, 100, 100, 54, 99, 55, 99, 102,
			],
			signature: String::from("GithubUserRegistered"),
		});

		assert_eq!(
			ContractAddress::from(
				"0x04e16efc9bc2d8d40ecb73d3d69e3e2d6f0fc3e2e6e9b7601310fdfa7dd6c7cf"
			),
			filter.contract_address
		);
		assert_eq!("GithubUserRegistered", filter.event_name);
	}

	#[test]
	fn indexer_from_apibara() {
		let indexer = Indexer::from(apibara::Indexer {
			id: String::from("ID"),
			network: Some(apibara::Network {
				network: Some(apibara::network::Network::Starknet(
					apibara::StarkNetNetwork {
						name: String::from("starknet-goerli"),
					},
				)),
			}),
			index_from_block: 1234,
			indexed_to_block: None,
			filters: vec![
				apibara::EventFilter {
					address: String::from("0x1234").into_bytes(),
					signature: String::from("event1"),
				},
				apibara::EventFilter {
					address: String::from("0x1234").into_bytes(),
					signature: String::from("event2"),
				},
			],
		});

		let expected_indexer = Indexer::new(
			IndexerId::from("ID"),
			Network::Starknet(StarknetChain::Goerli),
			1234,
			vec![
				EventFilter::new("0x1234", "event1"),
				EventFilter::new("0x1234", "event2"),
			],
		);

		assert_eq!(expected_indexer, indexer);
	}

	#[test]
	fn indexer_from_apibara_with_no_network() {
		let indexer = Indexer::from(apibara::Indexer {
			id: String::from("ID"),
			network: None,
			index_from_block: 1234,
			indexed_to_block: None,
			filters: Vec::new(),
		});

		let expected_indexer = Indexer::new(
			IndexerId::from("ID"),
			Network::Starknet(StarknetChain::Devnet),
			1234,
			Vec::new(),
		);

		assert_eq!(expected_indexer, indexer);
	}
}
