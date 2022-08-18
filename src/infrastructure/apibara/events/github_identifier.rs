use std::collections::VecDeque;

use super::{topics::*, FromEventError};
use crate::domain::*;
use crate::infrastructure::apibara::proto::TopicValue;
use crypto_bigint::U256;
use starknet::core::{types::FieldElement, utils::get_selector_from_name};

pub fn selector() -> FieldElement {
	get_selector_from_name("GithubIdentifierRegistered").unwrap()
}

impl TryFrom<VecDeque<TopicValue>> for GithubIdentifierRegisteredEvent {
	type Error = FromEventError;

	fn try_from(mut data: VecDeque<TopicValue>) -> Result<Self, Self::Error> {
		let profile_contract: ContractAddress =
			data.pop_front_as().map_err(|_| Self::Error::Invalid)?;
		let contributor_id: U256 = data.pop_front_as().map_err(|_| Self::Error::Invalid)?;
		let identifier: u128 = data.pop_front_as().map_err(|_| Self::Error::Invalid)?;

		Ok(Self {
			profile_contract,
			contributor_id,
			identifier,
		})
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use super::*;
	use rstest::*;

	#[fixture]
	fn apibara_event_data() -> VecDeque<TopicValue> {
		vec![
			TopicValue {
				value: vec![
					0, 65, 118, 135, 43, 113, 88, 60, 185, 188, 54, 113, 219, 40, 242, 110, 127,
					66, 106, 124, 7, 100, 97, 58, 8, 56, 187, 153, 239, 55, 58, 166,
				],
			},
			TopicValue {
				value: vec![
					0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
					0, 0, 0, 0, 0, 203,
				],
			},
			TopicValue {
				value: vec![
					0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
					0, 0, 0, 0, 0, 0,
				],
			},
			TopicValue {
				value: vec![
					0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
					0, 0, 6, 101, 25, 175,
				],
			},
		]
		.into()
	}

	#[rstest]
	fn create_event_from_apibara(apibara_event_data: VecDeque<TopicValue>) {
		let result = apibara_event_data.try_into();
		assert!(result.is_ok(), "{}", result.err().unwrap());
		assert_eq!(
			GithubIdentifierRegisteredEvent {
				profile_contract: ContractAddress::from_str(
					"0x004176872b71583cb9bc3671db28f26e7f426a7c0764613a0838bb99ef373aa6"
				)
				.unwrap(),
				contributor_id: ContributorId::from_u128(203),
				identifier: 107289007
			},
			result.unwrap()
		);
	}
}
