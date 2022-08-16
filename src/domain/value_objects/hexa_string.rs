use mapinto::ResultMapErrInto;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

pub type ContractAddress = HexaString;
pub type BlockHash = HexaString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexaString(Vec<u8>);

impl HexaString {
	pub fn bytes(&self) -> Vec<u8> {
		self.0.clone()
	}
}

impl Display for HexaString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "0x{}", hex::encode(&self.0))
	}
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseHexaStringError {
	#[error("provided string shoud be at least 4 characters long")]
	TooShort,
	#[error("provided string shoud be '0x' prefixed")]
	InvalidPrefix,
	#[error("provided string is not a valid hexadecimal string")]
	InvalidHexa(#[from] hex::FromHexError),
}

impl FromStr for HexaString {
	type Err = ParseHexaStringError;

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

impl From<Vec<u8>> for HexaString {
	fn from(bytes: Vec<u8>) -> Self {
		Self(bytes)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use rstest::*;

	#[test]
	fn hexa_string_can_be_created_from_and_transformed_into_a_string() {
		assert_eq!(
			"0x1234",
			HexaString::from_str("0x1234").unwrap().to_string()
		);
	}

	#[test]
	fn hexa_string_can_be_created_from_and_transformed_into_bytes() {
		assert_eq!(vec![12_u8, 34_u8], HexaString::from(vec![12, 34]).bytes());
	}

	#[rstest]
	#[case("12", ParseHexaStringError::TooShort)]
	#[case("1234", ParseHexaStringError::InvalidPrefix)]
	#[case(
		"0x123",
		ParseHexaStringError::InvalidHexa(hex::FromHexError::OddLength)
	)]
	fn parsing_errors(#[case] value: &str, #[case] expected_error: ParseHexaStringError) {
		assert_eq!(expected_error, HexaString::from_str(value).unwrap_err());
	}
}
