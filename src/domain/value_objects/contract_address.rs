#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractAddress(String);

impl<T: Into<String>> From<T> for ContractAddress {
	fn from(address: T) -> Self {
		Self(address.into())
	}
}

impl ToString for ContractAddress {
	fn to_string(&self) -> String {
		self.0.clone()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn contract_address_can_be_created_from_and_transformed_into_a_string() {
		assert_eq!("0x1234", ContractAddress::from("0x1234").to_string());
		assert_eq!(
			"0x4321",
			ContractAddress::from(String::from("0x4321")).to_string()
		);
	}
}
