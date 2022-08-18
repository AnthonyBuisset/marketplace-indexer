use std::fmt::Display;

use crypto_bigint::U256;

use super::ContractAddress;

pub type ContributorId = U256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
	GithubIdentifierRegistered(GithubIdentifierRegisteredEvent),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GithubIdentifierRegisteredEvent {
	pub profile_contract: ContractAddress,
	pub contributor_id: ContributorId,
	pub identifier: u128,
}

impl Display for Event {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::GithubIdentifierRegistered(event) => event.fmt(f),
		}
	}
}
