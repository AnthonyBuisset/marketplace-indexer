use crate::domain::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventFilter {
	contract_address: ContractAddress,
	event_name: String,
}

impl EventFilter {
	pub fn new<ADDRESS: Into<ContractAddress>, STRING: Into<String>>(
		contract_address: ADDRESS,
		event_name: STRING,
	) -> Self {
		Self {
			contract_address: contract_address.into(),
			event_name: event_name.into(),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn event_filter_can_be_created() {
		EventFilter::new("0x1234", "my_event");
	}
}
