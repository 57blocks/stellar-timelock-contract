use core::panic;

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, log, panic_with_error, Address, BytesN, Env, String};

use owner::owner;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    LimitReached = 1,
    LimitReached2 = 2,
}

#[contracttype]
#[derive(Clone)]
pub struct ContractConfig {
    pub owner: Address,
    pub name: String,
}

#[contracttype]
pub enum DataKey {
    Counter,
    AccountTotal(Address),
    ContractInfo(BytesN<32>),
}

#[contract]
pub struct IncrementContract;

#[contractimpl]
impl IncrementContract {

    pub fn initialize(env: Env, owner: Address) {
        if owner::has_owner(&env) {
            panic!("Contract already initialized");
        }
       
        owner::set_owner(&env, &owner);
    }

    /// Increment increments an internal counter, and returns the value.
    pub fn increment(env: Env, num: u32) -> u32 {
        owner::only_owner(&env);
        // Get the current count.
        let mut count: u32 = env.storage().instance().get(&DataKey::Counter).unwrap_or(0); // If no value set, assume 0.
        log!(&env, "count: {}", count);

        // Increment the count.
        count += num;

        // Save the count.
        env.storage().instance().set(&DataKey::Counter, &count);

        // The contract instance will be bumped to have a lifetime of at least 100 ledgers if the current expiration lifetime at most 50.
        // If the lifetime is already more than 100 ledgers, this is a no-op. Otherwise,
        // the lifetime is extended to 100 ledgers. This lifetime bump includes the contract
        // instance itself and all entries in storage().instance(), i.e, COUNTER.
        env.storage().instance().extend_ttl(50, 100);

        // Return the count to the caller.
        count
    }

    pub fn increment_five(env: Env) -> u32 {
        owner::only_owner(&env);
        // Get the current count.
        let mut count: u32 = env.storage().instance().get(&DataKey::Counter).unwrap_or(0); // If no value set, assume 0.
        log!(&env, "count: {}", count);

        // Increment the count.
        count += 5;

        // Save the count.
        env.storage().instance().set(&DataKey::Counter, &count);

        // The contract instance will be bumped to have a lifetime of at least 100 ledgers if the current expiration lifetime at most 50.
        // If the lifetime is already more than 100 ledgers, this is a no-op. Otherwise,
        // the lifetime is extended to 100 ledgers. This lifetime bump includes the contract
        // instance itself and all entries in storage().instance(), i.e, COUNTER.
        env.storage().instance().extend_ttl(50, 100);

        // Return the count to the caller.
        count
    }

    pub fn increment_owner(env: Env, num: u32) -> u32 {
        owner::only_owner(&env);
        IncrementContract::increment(env, num)
    }

    pub fn increment_account_total(env: Env, account: Address, num: u32) -> u32 {
        owner::only_owner(&env);

        let mut count: u32 = env.storage().instance().get(&DataKey::AccountTotal(account.clone())).unwrap_or(0);
        count += num;

        env.storage().instance().set(&DataKey::AccountTotal(account), &count);
        count
    }

    pub fn increment_return_error(env: Env, num: u32) -> Result<u32, Error> {
        owner::only_owner(&env);
        if num > 100 {
           return Err(Error::LimitReached)
        }

        let count = IncrementContract::increment(env, num);
        Ok(count)
    }

    pub fn increment_with_panic_error(env: Env, num: u32) -> u32 {
        if num > 100 {
            panic_with_error!(env, Error::LimitReached2)
        }

        IncrementContract::increment(env, num)
    }

    pub fn increment_with_panic(env: Env, num: u32) -> u32 {
        owner::only_owner(&env);
        if num > 100 {
            panic!("Limit reached")
        }

        IncrementContract::increment(env, num)
    }

    pub fn set_contract_info(env: Env, info: BytesN<32>, config: ContractConfig) {
        owner::only_owner(&env);

        env.storage().instance().set(&DataKey::ContractInfo(info), &config);
    }

    pub fn get_count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Counter).unwrap_or(0)
    }

    pub fn get_account_total(env: Env, account: Address) -> u32 {
        env.storage().instance().get(&DataKey::AccountTotal(account)).unwrap_or(0)
    }

    pub fn get_contract_info(env: Env, info: BytesN<32>) -> ContractConfig {
        env.storage().instance().get(&DataKey::ContractInfo(info)).unwrap()
    }
}