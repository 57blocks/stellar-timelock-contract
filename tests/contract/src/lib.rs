#![no_std]

mod contract;

#[cfg(any(test, feature = "testutils"))]
pub mod test {
    pub use crate::contract::{IncrementContract, IncrementContractClient, ContractConfig};
}

