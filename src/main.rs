use std::sync::Arc;

use dotenv::dotenv;
use marketplace_indexer::{application::IndexerBuilder, domain::*, infrastructure::ApibaraClient};

#[tokio::main]
async fn main() {
	dotenv().ok();
	env_logger::init();

	let apibara_client =
		Arc::new(ApibaraClient::default().await.expect("Unable to connect to Apibara server"));

	let indexer = IndexerBuilder::new(apibara_client.clone())
		.network(Network::Starknet(StarknetChain::Goerli))
		.start_at_block(291345)
		.on_conflict_recreate()
		.filter(registry_contract_address(), "GithubIdentifierRegistered")
		.filter(registry_contract_address(), "GithubIdentifierUnregistered")
		.build("indexer-goerli".into())
		.await
		.expect("Unable to create the indexer");

	apibara_client
		.fetch_new_events(&indexer, Arc::new(BlockchainLogger::default()))
		.await
		.expect("Error while fetching events");
}

fn registry_contract_address() -> ContractAddress {
	let address = std::env::var("REGISTRY_ADDRESS").expect("REGISTRY_ADDRESS must be set");
	address.parse().expect("REGISTRY_ADDRESS is not a valid contract address")
}
