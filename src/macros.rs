//! Macros for generating boilerplate Rust smart contract code.

/// Generate deploy method
#[macro_export]
macro_rules! deploy {
	($endpoint:ident, $contract:ident) => {
		#[no_mangle]
		pub fn deploy() {
			let mut endpoint = $endpoint::new($contract{});
			endpoint.dispatch_ctor(&owasm_ethereum::input());
		}
	}
}

/// Generate call method
#[macro_export]
macro_rules! call {
	($endpoint:ident, $contract:ident) => {
		#[no_mangle]
		pub fn call() {
			let mut endpoint = $endpoint::new($contract{});
			owasm_ethereum::ret(&endpoint.dispatch(&owasm_ethereum::input()));
		}
	}
}

/// Generate all boilerplate code
#[macro_export]
macro_rules! contract_boilerplate {
	($endpoint:ident, $contract:ident) => {
		use owasm_abi::eth::EndpointInterface;

		deploy!{$endpoint, $contract}

		call!{$endpoint, $contract}
	}
}
