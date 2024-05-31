#![cfg(test)]
extern crate std;

use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _};
use soroban_sdk::{symbol_short, vec, Address, BytesN, Env, IntoVal, Symbol};
use time_lock::{OperationState, TimeLockController, TimeLockControllerClient};
use time_lock_example_contract::{IncrementContract, IncrementContractClient};
use time_lock_tests_common::{current_timestamp, hash_call_data, set_env_timestamp};

const MIN_DELAY: u64 = 259200; // 60 * 60 * 24 * 3 => 3 days

#[test]
fn test_execute_work(){
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &Address::generate(&env),
    );

    let example_contract_id = env.register_contract(None, IncrementContract);
    let example_client = IncrementContractClient::new(&env, &example_contract_id);

    let owner = contract_id.clone();
    example_client.initialize(&owner);

    let target = example_contract_id.clone();
    let fn_name = Symbol::new(&env, "increment_owner");
    let sum: u32 = 1000;
    let data = (sum,).into_val(&env);
    let delay: u64 = 259201;
    let salt = BytesN::random(&env);

    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None,&delay);

    set_env_timestamp(&env, current_timestamp());

    client.execute(&executor, &target, &fn_name, &data, &salt, &None);

    let count = example_client.get_count();
    assert_eq!(count, 1000);
}

#[test]
#[should_panic]
fn test_execute_without_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &Address::generate(&env),
    );

    let example_contract_id = env.register_contract(None, IncrementContract);
    let example_client = IncrementContractClient::new(&env, &example_contract_id);

    let owner = Address::generate(&env);
    example_client.initialize(&owner);

    let target = example_contract_id.clone();
    let fn_name = Symbol::new(&env, "increment_owner");
    let sum: u32 = 1000;
    let data = (sum,).into_val(&env);
    let delay: u64 = 259201;
    let salt = BytesN::random(&env);

    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    set_env_timestamp(&env, current_timestamp());

    client.execute(&executor, &target, &fn_name, &data, &salt, &None);
}

#[test]
fn test_execute_work_with_no_args() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &Address::generate(&env),
    );

    let example_contract_id = env.register_contract(None, IncrementContract);
    let example_client = IncrementContractClient::new(&env, &example_contract_id);

    let target = example_contract_id.clone();
    let fn_name = Symbol::new(&env, "increment_five");
    let data = vec![&env];
    let delay: u64 = 259201;
    let salt = BytesN::random(&env);

    client.schedule(&proposer, &target, &fn_name, &data, &salt,&None, &delay);

    set_env_timestamp(&env, current_timestamp());

    client.execute(&executor, &target, &fn_name, &data, &salt, &None);

    let count = example_client.get_count();
    assert_eq!(count, 5);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_execute_waiting_revert() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &Address::generate(&env),
    );

    let example_contract_id = env.register_contract(None, IncrementContract);

    let target = example_contract_id.clone();
    let fn_name = Symbol::new(&env, "increment");
    let sum: u32 = 1000;
    let data = (sum,).into_val(&env);
    let delay: u64 = 259201;
    let salt = BytesN::random(&env);

    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None,&delay);

    client.execute(&executor, &target, &fn_name, &data, &salt, &None);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_execute_without_operation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    set_env_timestamp(&env, current_timestamp());

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &Address::generate(&env),
    );

    let target = Address::generate(&env);
    let fn_name = symbol_short!("hello");
    let data = vec![&env, symbol_short!("lily").to_val()];
    let salt = BytesN::random(&env);

    client.execute(&executor, &target, &fn_name, &data, &salt,&None);
}
