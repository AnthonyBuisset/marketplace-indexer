use mapinto::ResultMapErrInto;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractAddress(Vec<u8>);

impl ContractAddress {
	pub fn bytes(&self) -> Vec<u8> {
		self.0.clone()
	}
}

impl ToString for ContractAddress {
	fn to_string(&self) -> String {
		format!("0x{}", hex::encode(&self.0))
	}
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseContractAddressError {
	#[error("contract address shoud be at least 4 characters long")]
	TooShort,
	#[error("contract address shoud be '0x' prefixed")]
	InvalidPrefix,
	#[error("contract address is not a valid hexadecimal string")]
	InvalidHexa(#[from] hex::FromHexError),
}

impl FromStr for ContractAddress {
	type Err = ParseContractAddressError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			s if s.len() < 3 => Err(Self::Err::TooShort),
			s if s[0..2].to_lowercase() != "0x" => Err(Self::Err::InvalidPrefix),
			s => {
				let decoded: Result<_, Self::Err> = hex::decode(&s[2..]).map_err_into();
				Ok(Self(decoded?))
			},
		}
	}
}

impl From<Vec<u8>> for ContractAddress {
	fn from(bytes: Vec<u8>) -> Self {
		Self(bytes)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use rstest::*;

	#[test]
	fn contract_address_can_be_created_from_and_transformed_into_a_string() {
		assert_eq!(
			"0x1234",
			ContractAddress::from_str("0x1234").unwrap().to_string()
		);
	}

	#[test]
	fn contract_address_can_be_created_from_and_transformed_into_bytes() {
		assert_eq!(
			vec![12_u8, 34_u8],
			ContractAddress::from(vec![12, 34]).bytes()
		);
	}

	#[rstest]
	#[case("12", ParseContractAddressError::TooShort)]
	#[case("1234", ParseContractAddressError::InvalidPrefix)]
	#[case(
		"0x123",
		ParseContractAddressError::InvalidHexa(hex::FromHexError::OddLength)
	)]
	fn parsing_errors(#[case] value: &str, #[case] expected_error: ParseContractAddressError) {
		assert_eq!(
			expected_error,
			ContractAddress::from_str(value).unwrap_err()
		);
	}
}
