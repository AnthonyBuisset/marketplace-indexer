#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractAddress(String);

impl<T: Into<String>> From<T> for ContractAddress {
	fn from(address: T) -> Self {
		Self(address.into())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn contract_address_can_be_created_from_a_string() {
		let _ = ContractAddress::from("0x1234");
		let _ = ContractAddress::from(String::from("0x4321"));
	}
}
