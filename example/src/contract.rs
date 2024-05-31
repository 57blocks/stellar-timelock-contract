use soroban_sdk::{contract, contracttype, contractimpl, log, Address, Env};

#[contracttype]
pub enum DataKey {
    Owner,
    Counter,
}

#[contract]
pub struct IncrementContract;

#[contractimpl]
impl IncrementContract {

    pub fn initialize(env: Env, owner: Address) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&DataKey::Owner, &owner);
    }

    /// Increment increments an internal counter, and returns the value.
    pub fn increment(env: Env, num: u32) -> u32 {
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
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();
        IncrementContract::increment(env, num)
    }

    pub fn get_count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Counter).unwrap_or(0)
    }
}