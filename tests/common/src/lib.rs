use soroban_sdk::testutils::{Ledger, LedgerInfo};
use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env, Symbol, Val, Vec};
use std::time::{SystemTime, UNIX_EPOCH};
use time_lock::test::TimeLockControllerClient;

pub struct Context {
    pub env: Env,
    pub contract: Address,
    pub time_lock: TimeLockControllerClient<'static>,
    pub proposer: Address,
    pub executor: Address,
    pub admin: Address,
}

pub fn hash_call_data(
    env: &Env,
    target: &Address,
    fn_name: &Symbol,
    data: &Vec<Val>,
    predecessor: &Option<BytesN<32>>,
    salt: &BytesN<32>,
) -> BytesN<32> {
    let mut calldata = Bytes::new(&env);
    calldata.append(&target.clone().to_xdr(&env));
    calldata.append(&fn_name.clone().to_xdr(&env));
    calldata.append(&data.clone().to_xdr(&env));
    if let Some(predecessor) = predecessor {
        calldata.append(&predecessor.clone().to_xdr(&env));
    }
    calldata.append(&salt.clone().to_xdr(&env));
    env.crypto().sha256(&calldata)
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn set_env_timestamp(env: &Env, timestamp: u64) {
    env.ledger().set(LedgerInfo {
        timestamp,
        protocol_version: 20,
        sequence_number: 0,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 2_000_000,
    })
}
