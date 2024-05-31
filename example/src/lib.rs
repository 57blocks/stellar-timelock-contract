#![no_std]

#[cfg(any(test, feature = "testutils"))]
pub use crate::contract::{IncrementContract, IncrementContractClient};

mod contract;