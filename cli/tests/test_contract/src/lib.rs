#![no_std]

use owasm_std::logger::debug;

static KEY: H256 = H256([
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

#[owasm_abi_derive::contract]
trait TestContract {
    fn constructor(&mut self) {
        owasm_ethereum::write(&KEY, &U256::zero().into());
    }

    #[constant]
    fn testMethod(&mut self, quantity: u64, address: Address) -> U256 {
        debug("Getting count");
        U256::from_big_endian(&owasm_ethereum::read(&KEY))
    }

    fn secondTestMethod(&mut self) {
        owasm_ethereum::write(&KEY, &(self.testMethod(100, Address::zero()) + 1).into());
    }
}
