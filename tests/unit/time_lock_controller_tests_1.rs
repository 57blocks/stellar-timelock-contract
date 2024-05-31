#![cfg(test)]
extern crate std;

use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _, Events,
};
use soroban_sdk::{symbol_short, vec, Address, BytesN, Env, IntoVal, Symbol};
use time_lock::{
    CallExecutedEvent, CallScheduledEvent, OperationState, RoleKey, RoleLabel, TimeLockController,
    TimeLockControllerClient,
};
use time_lock_example_contract::{IncrementContract, IncrementContractClient};
use time_lock_tests_common::{current_timestamp, hash_call_data, set_env_timestamp};

const MIN_DELAY: u64 = 259200; // 60 * 60 * 24 * 3 => 3 days

const DONE_TIMESTAMP: u64 = 1;

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );
    assert_eq!(client.get_min_delay(), MIN_DELAY);
    assert_eq!(client.has_role(&proposer, &RoleLabel::Proposer), true);
    assert_eq!(client.has_role(&proposer, &RoleLabel::Canceller), true);
    assert_eq!(client.has_role(&executor, &RoleLabel::Executor), true);

    let all_actual_events = env.events().all();
    assert_eq!(all_actual_events.len(), 4);

    assert_eq!(
        all_actual_events,
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "AdminSet"),).into_val(&env),
                admin.clone().into_val(&env)
            ),
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "RoleGranted"),
                    RoleKey::Proposers(proposer.clone())
                )
                    .into_val(&env),
                proposer.clone().into_val(&env)
            ),
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "RoleGranted"),
                    RoleKey::Cancellers(proposer.clone())
                )
                    .into_val(&env),
                proposer.clone().into_val(&env)
            ),
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "RoleGranted"),
                    RoleKey::Executors(executor.clone())
                )
                    .into_val(&env),
                executor.clone().into_val(&env)
            ),
        ]
    );
}

#[test]
#[should_panic = "Error(Contract, #2)"]
fn test_initialize_twice_should_panic() {
    let env = Env::default();
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
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &Address::generate(&env),
    );
}

#[test]
#[should_panic = "Error(Contract, #0)"]
fn test_initialize_with_0_min_delay_should_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    client.initialize(
        &0,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &Address::generate(&env),
    );
}

#[test]
#[should_panic = "Error(Contract, #0)"]
fn test_initialize_without_proposer_should_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let executor = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env],
        &vec![&env, executor.clone()],
        &Address::generate(&env),
    );
}

#[test]
#[should_panic = "Error(Contract, #0)"]
fn test_initialize_without_executor_should_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env],
        &Address::generate(&env),
    );
}

#[test]
#[should_panic = "Error(Contract, #0)"]
fn test_initialize_without_proposer_and_executor_should_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    client.initialize(
        &MIN_DELAY,
        &vec![&env],
        &vec![&env],
        &Address::generate(&env),
    );
}

#[test]
#[should_panic = "Error(Contract, #7)"]
fn test_initialize_proposer_exceed_should_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let proposers = vec![
        &env,
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
    ];
    client.initialize(
        &MIN_DELAY,
        &proposers,
        &vec![&env, executor.clone()],
        &Address::generate(&env),
    );
}

#[test]
#[should_panic = "Error(Contract, #7)"]
fn test_initialize_executor_exceed_should_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let executors = vec![
        &env,
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
    ];
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &executors,
        &Address::generate(&env),
    );
}

#[test]
#[should_panic = "Error(Contract, #7)"]
fn test_initialize_proposer_and_executor_exceed_should_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let proposers = vec![
        &env,
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
        proposer.clone(),
    ];
    let executors = vec![
        &env,
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
        executor.clone(),
    ];
    client.initialize(&MIN_DELAY, &proposers, &executors, &Address::generate(&env));
}

#[test]
fn test_schedule() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    set_env_timestamp(&env, current_timestamp());

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let target = Address::generate(&env);
    let fn_name = symbol_short!("hello");
    let data = vec![&env, symbol_short!("lily").to_val()];
    let salt = BytesN::random(&env);
    let delay: u64 = 259201;

    let ledger_time = env.ledger().timestamp();
    let expect_lock_time = ledger_time + delay;

    let predecessor: Option<BytesN<32>> = None;
    let (operation_id, lock_time) = client.schedule(
        &proposer,
        &target,
        &fn_name,
        &data,
        &salt,
        &predecessor,
        &delay,
    );

    assert_eq!(
        env.auths(),
        std::vec![(
            proposer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "schedule"),
                    (
                        &proposer,
                        &target,
                        &fn_name,
                        data.clone(),
                        salt.clone(),
                        predecessor.clone(),
                        delay
                    )
                        .into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(lock_time, expect_lock_time);

    let expected_operation_id = hash_call_data(&env, &target, &fn_name, &data, &predecessor, &salt);
    assert_eq!(operation_id, expected_operation_id);

    let actual_lock_time = client.get_schedule_lock_time(&operation_id);
    assert_eq!(actual_lock_time, expect_lock_time);

    let actual_events = env.events().all();
    let event_len = actual_events.len();
    assert_eq!(event_len, 5);

    assert_eq! {
        actual_events.slice(event_len - 1..),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "CallScheduled"),).into_val(&env),
                CallScheduledEvent {
                    opt_id: operation_id.clone(),
                    index: 0_u32,
                    target: target.clone(),
                    fn_name:fn_name.clone(),
                    data: data.clone(),
                    predecessor: BytesN::from_array(&env, &[0_u8; 32]),
                    delay: delay
                }.into_val(&env)
            )
        ]
    }
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_schedule_without_proposer_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    set_env_timestamp(&env, current_timestamp());

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let target = Address::generate(&env);
    let fn_name = symbol_short!("hello");
    let data = vec![&env, symbol_short!("lily").to_val()];
    let salt = BytesN::random(&env);
    let delay: u64 = 259201;

    let caller = Address::generate(&env);
    client.schedule(&caller, &target, &fn_name, &data, &salt, &None, &delay);
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_schedule_without_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    set_env_timestamp(&env, current_timestamp());

    let target = Address::generate(&env);
    let fn_name = symbol_short!("hello");
    let data = vec![&env, symbol_short!("lily").to_val()];
    let salt = BytesN::random(&env);
    let delay: u64 = 259201;

    let caller = Address::generate(&env);
    client.schedule(&caller, &target, &fn_name, &data, &salt, &None, &delay);
}

#[test]
#[should_panic = "Error(Contract, #3)"]
fn test_schedule_twice_should_panic() {
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

    let target = Address::generate(&env);
    let fn_name = symbol_short!("hello");
    let data = vec![&env, symbol_short!("lily").to_val()];
    let salt = BytesN::random(&env);
    let delay: u64 = 259201;
    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);
    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);
}

#[test]
#[should_panic = "Error(Contract, #4)"]
fn test_schedule_insufficient_delay_should_panic() {
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

    let target = Address::generate(&env);
    let fn_name = symbol_short!("hello");
    let data = vec![&env, symbol_short!("lily").to_val()];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY - 10;
    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);
}

#[test]
fn test_schedule_batch() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let target1 = Address::generate(&env);
    let target2 = Address::generate(&env);
    let targets = vec![&env, target1.clone(), target2.clone()];
    let fn_name = symbol_short!("hello");
    let fn_names = vec![&env, fn_name.clone(), fn_name.clone()];
    let data1 = vec![&env, symbol_short!("lily").to_val()];
    let data2 = vec![&env, symbol_short!("lucy").to_val()];
    let datas = vec![&env, data1.clone(), data2.clone()];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY + 10;
    let predecessor: Option<BytesN<32>> = None;

    let ledger_time = env.ledger().timestamp();
    let expect_lock_time = ledger_time + delay;

    let (operation_id, lock_time) = client.schedule_batch(
        &proposer,
        &targets,
        &fn_names,
        &datas,
        &salt,
        &predecessor,
        &delay,
    );

    assert_eq!(
        env.auths(),
        std::vec![(
            proposer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "schedule_batch"),
                    (
                        &proposer,
                        targets.clone(),
                        fn_names.clone(),
                        datas.clone(),
                        salt.clone(),
                        predecessor.clone(),
                        delay
                    )
                        .into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(lock_time, expect_lock_time);

    let actual_lock_time = client.get_schedule_lock_time(&operation_id);
    assert_eq!(actual_lock_time, expect_lock_time);

    let actual_events = env.events().all();
    let event_len = actual_events.len();
    assert_eq!(event_len, 6);

    assert_eq! {
            actual_events.slice(event_len - 2..),
            vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "CallScheduled"),).into_val(&env),
                    CallScheduledEvent {
                        opt_id: operation_id.clone(),
                        index: 0_u32,
                        target: target1.clone(),
                        fn_name:fn_name.clone(),
                        data: data1.clone(),
                        predecessor: BytesN::from_array(&env, &[0_u8; 32]),
                        delay: delay
                    }.into_val(&env)
                ),
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "CallScheduled"),).into_val(&env),
                    CallScheduledEvent {
                        opt_id: operation_id.clone(),
                        index: 1_u32,
                        target: target2.clone(),
                        fn_name:fn_name.clone(),
                        data: data2.clone(),
                        predecessor: BytesN::from_array(&env, &[0_u8; 32]),
                        delay: delay
                    }.into_val(&env)
                )
            ]
    }
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_schedule_batch_without_proposer_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let targets = vec![&env, Address::generate(&env), Address::generate(&env)];
    let fn_names = vec![&env, symbol_short!("hello"), symbol_short!("hello")];
    let datas = vec![
        &env,
        vec![&env, symbol_short!("lily").to_val()],
        vec![&env, symbol_short!("lucy").to_val()],
    ];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY + 10;

    let caller = Address::generate(&env);
    client.schedule_batch(&caller, &targets, &fn_names, &datas, &salt, &None, &delay);
}

#[test]
#[should_panic = "Error(Contract, #3)"]
fn test_schedule_batch_twice_should_panic() {
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

    let targets = vec![&env, Address::generate(&env), Address::generate(&env)];
    let fn_names = vec![&env, symbol_short!("hello"), symbol_short!("hello")];
    let datas = vec![
        &env,
        vec![&env, symbol_short!("lily").to_val()],
        vec![&env, symbol_short!("lucy").to_val()],
    ];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY + 10;
    client.schedule_batch(&proposer, &targets, &fn_names, &datas, &salt, &None, &delay);
    client.schedule_batch(&proposer, &targets, &fn_names, &datas, &salt, &None, &delay);
}

#[test]
#[should_panic = "Error(Contract, #4)"]
fn test_schedule_batch_insufficient_delay_should_panic() {
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

    let targets = vec![&env, Address::generate(&env), Address::generate(&env)];
    let fn_names = vec![&env, symbol_short!("hello"), symbol_short!("hello")];
    let datas = vec![
        &env,
        vec![&env, symbol_short!("lily").to_val()],
        vec![&env, symbol_short!("lucy").to_val()],
    ];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY - 10;
    client.schedule_batch(&proposer, &targets, &fn_names, &datas, &salt, &None, &delay);
}

#[test]
#[should_panic = "Error(Contract, #7)"]
fn test_schedule_batch_exceed_should_panic() {
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

    let target = Address::generate(&env);
    let targets = vec![
        &env,
        target.clone(),
        target.clone(),
        target.clone(),
        target.clone(),
        target.clone(),
        target.clone(),
    ];
    let fn_name = symbol_short!("hello");
    let fn_names = vec![
        &env,
        fn_name.clone(),
        fn_name.clone(),
        fn_name.clone(),
        fn_name.clone(),
        fn_name.clone(),
        fn_name.clone(),
    ];
    let datas = vec![
        &env,
        vec![&env, symbol_short!("lily").to_val()],
        vec![&env, symbol_short!("lucy").to_val()],
        vec![&env, symbol_short!("lucy").to_val()],
        vec![&env, symbol_short!("lucy").to_val()],
        vec![&env, symbol_short!("lucy").to_val()],
        vec![&env, symbol_short!("lucy").to_val()],
    ];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY + 10;

    client.schedule_batch(&proposer, &targets, &fn_names, &datas, &salt, &None, &delay);
}

#[test]
#[should_panic = "Error(Contract, #0)"]
fn test_schedule_batch_params_length_mismatch_should_panic() {
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

    let target = Address::generate(&env);
    let targets = vec![&env, target.clone(), target.clone()];
    let fn_name = symbol_short!("hello");
    let fn_names = vec![&env, fn_name.clone(), fn_name.clone(), fn_name.clone()];
    let datas = vec![
        &env,
        vec![&env, symbol_short!("lily").to_val()],
        vec![&env, symbol_short!("lucy").to_val()],
    ];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY + 10;

    client.schedule_batch(&proposer, &targets, &fn_names, &datas, &salt, &None, &delay);
}

#[test]
fn test_execute_work() {
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
    let fn_name = Symbol::new(&env, "increment");
    let sum: u32 = 1000;
    let data = (sum,).into_val(&env);
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = None;

    let (operation_id, _) = client.schedule(
        &proposer,
        &target,
        &fn_name,
        &data,
        &salt,
        &predecessor,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    client.execute(&executor, &target, &fn_name, &data, &salt, &predecessor);

    assert_eq!(
        env.auths(),
        std::vec![(
            executor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "execute"),
                    (
                        &executor,
                        &target,
                        &fn_name,
                        data.clone(),
                        salt.clone(),
                        predecessor.clone(),
                    )
                        .into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(example_client.get_count(), 1000);
    assert_eq!(
        client.get_schedule_state(&operation_id),
        OperationState::Executed
    );
    assert_eq!(client.get_schedule_lock_time(&operation_id), DONE_TIMESTAMP);

    let actual_events = env.events().all();
    let event_len = actual_events.len();
    assert_eq!(event_len, 6);

    assert_eq! {
        actual_events.slice(event_len - 1..),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "CallExecuted"),).into_val(&env),
                CallExecutedEvent {
                    opt_id: operation_id.clone(),
                    index: 0_u32,
                    target: target.clone(),
                    fn_name:fn_name.clone(),
                    data: data.clone(),
                }.into_val(&env)
            )
        ]
    }
}

#[test]
fn test_execute_work_with_predecessor() {
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
    let fn_name = Symbol::new(&env, "increment");
    let sum: u32 = 1000;
    let data = (sum,).into_val(&env);
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = None;

    let (pre_operation_id, _) = client.schedule(
        &proposer,
        &target,
        &fn_name,
        &data,
        &salt,
        &predecessor,
        &delay,
    );

    let salt2 = BytesN::random(&env);
    let predecessor2 = Some(pre_operation_id.clone());
    let (operation_id, _) = client.schedule(
        &proposer,
        &target,
        &fn_name,
        &data,
        &salt2,
        &predecessor2,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    client.execute(&executor, &target, &fn_name, &data, &salt, &predecessor);
    assert_eq!(example_client.get_count(), 1000);
    assert_eq!(
        client.get_schedule_state(&pre_operation_id),
        OperationState::Executed
    );

    client.execute(&executor, &target, &fn_name, &data, &salt2, &predecessor2);
    assert_eq!(example_client.get_count(), 2000);
    assert_eq!(
        client.get_schedule_state(&operation_id),
        OperationState::Executed
    );
}

#[test]
#[should_panic = "Error(Contract, #6)"]
fn test_execute_revert_with_predecessor_not_ready() {
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

    let (pre_operation_id, _) =
        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    let salt2 = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = Some(pre_operation_id);
    client.schedule(
        &proposer,
        &target,
        &fn_name,
        &data,
        &salt2,
        &predecessor,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    client.execute(&executor, &target, &fn_name, &data, &salt2, &predecessor);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_execute_revert_with_predecessor_not_exist() {
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

    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    set_env_timestamp(&env, current_timestamp());

    let predecessor: Option<BytesN<32>> = Some(BytesN::random(&env));
    client.execute(&executor, &target, &fn_name, &data, &salt, &predecessor);
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_execute_without_executor_should_panic() {
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
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);

    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    set_env_timestamp(&env, current_timestamp());

    let caller = Address::generate(&env);
    client.execute(&caller, &target, &fn_name, &data, &salt, &None);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_execute_with_not_scheduled_operation_should_panic() {
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
    let salt = BytesN::random(&env);

    set_env_timestamp(&env, current_timestamp());

    client.execute(&executor, &target, &fn_name, &data, &salt, &None);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_execute_with_operation_not_ready_should_panic() {
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
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);

    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    client.execute(&executor, &target, &fn_name, &data, &salt, &None);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_execute_twice_operation_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let example_contract_id = env.register_contract(None, IncrementContract);

    let target = example_contract_id.clone();
    let fn_name = Symbol::new(&env, "increment");
    let sum: u32 = 1000;
    let data = (sum,).into_val(&env);
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);

    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    set_env_timestamp(&env, current_timestamp());

    client.execute(&executor, &target, &fn_name, &data, &salt, &None);
    client.execute(&executor, &target, &fn_name, &data, &salt, &None);
}

#[test]
fn test_execute_batch_work() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let example_contract_id = env.register_contract(None, IncrementContract);
    let example_client = IncrementContractClient::new(&env, &example_contract_id);

    let targets = vec![
        &env,
        example_contract_id.clone(),
        example_contract_id.clone(),
    ];
    let fn_name = Symbol::new(&env, "increment");
    let fn_names = vec![&env, fn_name.clone(), fn_name.clone()];
    let data1 = 1000_u32;
    let data2 = 2000_u32;
    let datas = vec![&env, (data1,).into_val(&env), (data2,).into_val(&env)];
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = None;

    let (operation_id, _) = client.schedule_batch(
        &proposer,
        &targets,
        &fn_names,
        &datas,
        &salt,
        &predecessor,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    client.execute_batch(&executor, &targets, &fn_names, &datas, &salt, &predecessor);

    assert_eq!(
        env.auths(),
        std::vec![(
            executor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "execute_batch"),
                    (
                        &executor,
                        targets.clone(),
                        fn_names.clone(),
                        datas.clone(),
                        salt.clone(),
                        predecessor.clone(),
                    )
                        .into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(example_client.get_count(), 3000);
    assert_eq!(
        client.get_schedule_state(&operation_id),
        OperationState::Executed
    );

    let actual_events = env.events().all();
    let event_len = actual_events.len();
    assert_eq!(event_len, 8);

    assert_eq! {
        actual_events.slice(event_len - 2..),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "CallExecuted"),).into_val(&env),
                CallExecutedEvent {
                    opt_id: operation_id.clone(),
                    index: 0_u32,
                    target: example_contract_id.clone(),
                    fn_name:fn_name.clone(),
                    data: (data1,).into_val(&env),
                }.into_val(&env)
            ),
            (
                contract_id.clone(),
                (Symbol::new(&env, "CallExecuted"),).into_val(&env),
                CallExecutedEvent {
                    opt_id: operation_id.clone(),
                    index: 1_u32,
                    target: example_contract_id.clone(),
                    fn_name:fn_name.clone(),
                    data: (data2,).into_val(&env),
                }.into_val(&env)
            )
        ]
    }
}

#[test]
fn test_execute_batch_work_with_predecessor() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let example_contract_id = env.register_contract(None, IncrementContract);
    let example_client = IncrementContractClient::new(&env, &example_contract_id);

    let target = example_contract_id.clone();
    let fn_name = Symbol::new(&env, "increment");
    let data = (1000_u32,).into_val(&env);
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY + 10;

    let (pre_operation_id, _) =
        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    let targets = vec![
        &env,
        example_contract_id.clone(),
        example_contract_id.clone(),
    ];
    let fn_names = vec![
        &env,
        Symbol::new(&env, "increment"),
        Symbol::new(&env, "increment"),
    ];
    let datas = vec![&env, (1000_u32,).into_val(&env), (2000_u32,).into_val(&env)];
    let salt2 = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = Some(pre_operation_id.clone());

    let (operation_id, _) = client.schedule_batch(
        &proposer,
        &targets,
        &fn_names,
        &datas,
        &salt2,
        &predecessor,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    client.execute(&executor, &target, &fn_name, &data, &salt, &None);
    client.execute_batch(&executor, &targets, &fn_names, &datas, &salt2, &predecessor);

    assert_eq!(example_client.get_count(), 4000);
    assert_eq!(
        client.get_schedule_state(&pre_operation_id),
        OperationState::Executed
    );
    assert_eq!(
        client.get_schedule_state(&operation_id),
        OperationState::Executed
    );
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_execute_batch_without_executor_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let example_contract_id = env.register_contract(None, IncrementContract);

    let targets = vec![
        &env,
        example_contract_id.clone(),
        example_contract_id.clone(),
    ];
    let fn_names = vec![
        &env,
        Symbol::new(&env, "increment"),
        Symbol::new(&env, "increment"),
    ];
    let datas = vec![&env, (1000_u32,).into_val(&env), (2000_u32,).into_val(&env)];
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = None;

    client.schedule_batch(
        &proposer,
        &targets,
        &fn_names,
        &datas,
        &salt,
        &predecessor,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    let caller = Address::generate(&env);
    client.execute_batch(&caller, &targets, &fn_names, &datas, &salt, &predecessor);
}

#[test]
#[should_panic = "Error(Contract, #0)"]
fn test_execute_batch_params_length_mismatch_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let example_contract_id = env.register_contract(None, IncrementContract);

    let targets = vec![
        &env,
        example_contract_id.clone(),
        example_contract_id.clone(),
    ];
    let fn_names = vec![
        &env,
        Symbol::new(&env, "increment"),
        Symbol::new(&env, "increment"),
    ];
    let datas = vec![&env, (1000_u32,).into_val(&env)];
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = None;

    client.schedule_batch(
        &proposer,
        &targets,
        &fn_names,
        &datas,
        &salt,
        &predecessor,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    client.execute_batch(&executor, &targets, &fn_names, &datas, &salt, &predecessor);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_execute_batch_with_not_scheduled_operation_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let example_contract_id = env.register_contract(None, IncrementContract);

    let targets = vec![
        &env,
        example_contract_id.clone(),
        example_contract_id.clone(),
    ];
    let fn_names = vec![
        &env,
        Symbol::new(&env, "increment"),
        Symbol::new(&env, "increment"),
    ];
    let datas = vec![&env, (1000_u32,).into_val(&env), (2000_u32,).into_val(&env)];
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = None;

    set_env_timestamp(&env, current_timestamp());

    client.execute_batch(&executor, &targets, &fn_names, &datas, &salt, &predecessor);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_execute_batch_with_operation_not_ready_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let example_contract_id = env.register_contract(None, IncrementContract);

    let targets = vec![
        &env,
        example_contract_id.clone(),
        example_contract_id.clone(),
    ];
    let fn_names = vec![
        &env,
        Symbol::new(&env, "increment"),
        Symbol::new(&env, "increment"),
    ];
    let datas = vec![&env, (1000_u32,).into_val(&env), (2000_u32,).into_val(&env)];
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = None;

    client.schedule_batch(
        &proposer,
        &targets,
        &fn_names,
        &datas,
        &salt,
        &predecessor,
        &delay,
    );

    client.execute_batch(&executor, &targets, &fn_names, &datas, &salt, &predecessor);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_execute_batch_twice_operation_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let example_contract_id = env.register_contract(None, IncrementContract);

    let targets = vec![
        &env,
        example_contract_id.clone(),
        example_contract_id.clone(),
    ];
    let fn_names = vec![
        &env,
        Symbol::new(&env, "increment"),
        Symbol::new(&env, "increment"),
    ];
    let datas = vec![&env, (1000_u32,).into_val(&env), (2000_u32,).into_val(&env)];
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = None;

    client.schedule_batch(
        &proposer,
        &targets,
        &fn_names,
        &datas,
        &salt,
        &predecessor,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    client.execute_batch(&executor, &targets, &fn_names, &datas, &salt, &predecessor);
    client.execute_batch(&executor, &targets, &fn_names, &datas, &salt, &predecessor);
}

#[test]
#[should_panic = "Error(Contract, #6)"]
fn test_execute_batch_when_predecessor_not_executed_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let example_contract_id = env.register_contract(None, IncrementContract);

    let target = example_contract_id.clone();
    let fn_name = Symbol::new(&env, "increment");
    let sum: u32 = 1000;
    let data = (sum,).into_val(&env);
    let delay: u64 = 259201;
    let salt = BytesN::random(&env);

    let (pre_operation_id, _) =
        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    let targets = vec![
        &env,
        example_contract_id.clone(),
        example_contract_id.clone(),
    ];
    let fn_names = vec![
        &env,
        Symbol::new(&env, "increment"),
        Symbol::new(&env, "increment"),
    ];
    let datas = vec![&env, (1000_u32,).into_val(&env), (2000_u32,).into_val(&env)];
    let salt2 = BytesN::random(&env);
    let predecessor: Option<BytesN<32>> = Some(pre_operation_id.clone());

    client.schedule_batch(
        &proposer,
        &targets,
        &fn_names,
        &datas,
        &salt2,
        &predecessor,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    client.execute_batch(&executor, &targets, &fn_names, &datas, &salt2, &predecessor);
}

#[test]
fn test_cancel_operation_with_ready() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let target = Address::generate(&env);
    let fn_name = symbol_short!("hello");
    let data = vec![&env, symbol_short!("lily").to_val()];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY + 10;
    let predecessor: Option<BytesN<32>> = None;
    let (operation_id, _) = client.schedule(
        &proposer,
        &target,
        &fn_name,
        &data,
        &salt,
        &predecessor,
        &delay,
    );

    let ledger_time = env.ledger().timestamp();
    let expect_lock_time = ledger_time + delay;

    let actual_lock_time = client.get_schedule_lock_time(&operation_id);
    assert_eq!(actual_lock_time, expect_lock_time);

    set_env_timestamp(&env, current_timestamp());

    client.cancel(&proposer, &operation_id);

    assert_eq!(
        env.auths(),
        std::vec![(
            proposer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "cancel"),
                    (&proposer, operation_id.clone(),).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(
        client.get_schedule_state(&operation_id),
        OperationState::Unset
    );
    assert_eq!(client.get_schedule_lock_time(&operation_id), 0);

    let actual_events = env.events().all();
    let event_len = actual_events.len();
    assert_eq!(event_len, 6);

    assert_eq! {
        actual_events.slice(event_len - 1..),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "OperationCancelled"),).into_val(&env),
                operation_id.clone().into_val(&env)
            )
        ]
    }
}

#[test]
fn test_cancel_operation_with_waiting() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    set_env_timestamp(&env, current_timestamp());

    let target = Address::generate(&env);
    let fn_name = symbol_short!("hello");
    let data = vec![&env, symbol_short!("lily").to_val()];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY + 10;
    let predecessor: Option<BytesN<32>> = None;

    let (operation_id, _) = client.schedule(
        &proposer,
        &target,
        &fn_name,
        &data,
        &salt,
        &predecessor,
        &delay,
    );

    let ledger_time = env.ledger().timestamp();
    let expect_lock_time = ledger_time + delay;

    let actual_lock_time = client.get_schedule_lock_time(&operation_id);
    assert_eq!(actual_lock_time, expect_lock_time);

    client.cancel(&proposer, &operation_id);

    assert_eq!(
        env.auths(),
        std::vec![(
            proposer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "cancel"),
                    (&proposer, operation_id.clone(),).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(
        client.get_schedule_state(&operation_id),
        OperationState::Unset
    );
    assert_eq!(client.get_schedule_lock_time(&operation_id), 0);

    let actual_events = env.events().all();
    let event_len = actual_events.len();
    assert_eq!(event_len, 6);

    assert_eq! {
        actual_events.slice(event_len - 1..),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "OperationCancelled"),).into_val(&env),
                operation_id.clone().into_val(&env)
            )
        ]
    }
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_cancel_without_canceler_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let target = Address::generate(&env);
    let fn_name = symbol_short!("hello");
    let data = vec![&env, symbol_short!("lily").to_val()];
    let salt = BytesN::random(&env);
    let delay: u64 = MIN_DELAY + 10;

    let (operation_id, _) =
        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    set_env_timestamp(&env, current_timestamp());

    let caller = Address::generate(&env);
    client.cancel(&caller, &operation_id);
}

#[test]
#[should_panic = "Error(Contract, #8)"]
fn test_cancel_with_unset_operation_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let operation_id = BytesN::random(&env);

    client.cancel(&proposer, &operation_id);
}

#[test]
#[should_panic = "Error(Contract, #8)"]
fn test_cancel_with_executed_operation_should_panic() {
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
    let delay: u64 = MIN_DELAY + 10;
    let salt = BytesN::random(&env);

    let (operation_id, _) =
        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    set_env_timestamp(&env, current_timestamp());

    let predecessor: Option<BytesN<32>> = None;
    client.execute(&executor, &target, &fn_name, &data, &salt, &predecessor);

    client.cancel(&proposer, &operation_id);
}

#[test]
fn test_update_min_delay() {
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

    let target = contract_id.clone();
    let fn_name = Symbol::new(&env, "update_min_delay");
    let delay: u64 = 300000;
    let salt = BytesN::random(&env);
    let data = vec![&env, delay.into_val(&env), salt.to_val()];
    let predecessor: Option<BytesN<32>> = None;

    let (operation_id, _) = client.schedule(
        &proposer,
        &target,
        &fn_name,
        &data,
        &salt,
        &predecessor,
        &delay,
    );

    set_env_timestamp(&env, current_timestamp());

    let new_min_delay = client.update_min_delay(&delay, &salt);
    assert_eq!(new_min_delay, delay);

    let actual_delay = client.get_min_delay();
    assert_eq!(actual_delay, delay);
    assert_eq!(
        client.get_schedule_state(&operation_id),
        OperationState::Executed
    );

    let actual_events = env.events().all();
    let event_len = actual_events.len();
    assert_eq!(event_len, 6);

    assert_eq! {
        actual_events.slice(event_len - 1..),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "MinDelayUpdated"),).into_val(&env),
                new_min_delay.into_val(&env)
            )
        ]
    }
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_update_min_delay_when_waiting() {
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

    let target = contract_id.clone();
    let fn_name = Symbol::new(&env, "update_min_delay");
    let delay: u64 = 300000;
    let salt = BytesN::random(&env);
    let data = vec![&env, delay.into_val(&env), salt.to_val()];

    client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

    client.update_min_delay(&delay, &salt);
}

#[test]
#[should_panic = "Error(Contract, #5)"]
fn test_update_min_delay_without_schedule() {
    let env = Env::default();
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

    let delay: u64 = 300000;
    let salt = BytesN::random(&env);

    client.update_min_delay(&delay, &salt);
}

#[test]
fn test_grant_proposer_role() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_proposer = Address::generate(&env);
    client.grant_role(&new_proposer, &RoleLabel::Proposer);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "grant_role"),
                    (&new_proposer, RoleLabel::Proposer,).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(client.has_role(&new_proposer, &RoleLabel::Proposer), true);
}

#[test]
fn test_grant_proposer_role_duplicate_add() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_proposer = Address::generate(&env);
    assert_eq!(client.grant_role(&new_proposer, &RoleLabel::Proposer), true);
    assert_eq!(
        client.grant_role(&new_proposer, &RoleLabel::Proposer),
        false
    );

    assert_eq!(client.has_role(&new_proposer, &RoleLabel::Proposer), true);
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_grant_proposer_role_not_initialized_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let new_proposer = Address::generate(&env);
    client.grant_role(&new_proposer, &RoleLabel::Proposer);
}

#[test]
fn test_revoke_proposer_role() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_proposer = Address::generate(&env);
    client.grant_role(&new_proposer, &RoleLabel::Proposer);
    assert_eq!(client.has_role(&new_proposer, &RoleLabel::Proposer), true);

    client.revoke_role(&new_proposer, &RoleLabel::Proposer);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "revoke_role"),
                    (&new_proposer, RoleLabel::Proposer,).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(client.has_role(&new_proposer, &RoleLabel::Proposer), false);
}

#[test]
fn test_revoke_proposer_role_duplicate_revoke() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_proposer = Address::generate(&env);
    client.grant_role(&new_proposer, &RoleLabel::Proposer);
    assert_eq!(client.has_role(&new_proposer, &RoleLabel::Proposer), true);

    assert_eq!(
        client.revoke_role(&new_proposer, &RoleLabel::Proposer),
        true
    );

    assert_eq!(
        client.revoke_role(&new_proposer, &RoleLabel::Proposer),
        false
    );

    assert_eq!(client.has_role(&new_proposer, &RoleLabel::Proposer), false);
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_revoke_proposer_role_not_initialized_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let new_proposer = Address::generate(&env);

    client.revoke_role(&new_proposer, &RoleLabel::Proposer);
}

#[test]
fn test_grant_executor_role() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_executor = Address::generate(&env);
    client.grant_role(&new_executor, &RoleLabel::Executor);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "grant_role"),
                    (&new_executor, RoleLabel::Executor,).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(client.has_role(&new_executor, &RoleLabel::Executor), true);
}

#[test]
fn test_grant_executor_role_duplicate_add() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_executor = Address::generate(&env);
    assert_eq!(client.grant_role(&new_executor, &RoleLabel::Executor), true);
    assert_eq!(
        client.grant_role(&new_executor, &RoleLabel::Executor),
        false
    );

    assert_eq!(client.has_role(&new_executor, &RoleLabel::Executor), true);
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_grant_executor_role_not_initialized_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let new_executor = Address::generate(&env);
    client.grant_role(&new_executor, &RoleLabel::Executor);
}

#[test]
fn test_revoke_executor_role() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_executor = Address::generate(&env);
    client.grant_role(&new_executor, &RoleLabel::Executor);
    assert_eq!(client.has_role(&new_executor, &RoleLabel::Executor), true);

    client.revoke_role(&new_executor, &RoleLabel::Executor);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "revoke_role"),
                    (&new_executor, RoleLabel::Executor,).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(client.has_role(&new_executor, &RoleLabel::Executor), false);
}

#[test]
fn test_revoke_executor_role_duplicate_revoke() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_executor = Address::generate(&env);
    client.grant_role(&new_executor, &RoleLabel::Executor);
    assert_eq!(client.has_role(&new_executor, &RoleLabel::Executor), true);

    assert_eq!(
        client.revoke_role(&new_executor, &RoleLabel::Executor),
        true
    );

    assert_eq!(
        client.revoke_role(&new_executor, &RoleLabel::Executor),
        false
    );

    assert_eq!(client.has_role(&new_executor, &RoleLabel::Executor), false);
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_revoke_executor_role_not_initialized_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let new_executor = Address::generate(&env);

    client.revoke_role(&new_executor, &RoleLabel::Executor);
}

#[test]
fn test_grant_canceller_role() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_canceller = Address::generate(&env);
    client.grant_role(&new_canceller, &RoleLabel::Canceller);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "grant_role"),
                    (&new_canceller, RoleLabel::Canceller,).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(client.has_role(&new_canceller, &RoleLabel::Canceller), true);
}

#[test]
fn test_grant_canceller_role_duplicate_add() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_canceller = Address::generate(&env);
    assert_eq!(
        client.grant_role(&new_canceller, &RoleLabel::Canceller),
        true
    );
    assert_eq!(
        client.grant_role(&new_canceller, &RoleLabel::Canceller),
        false
    );

    assert_eq!(client.has_role(&new_canceller, &RoleLabel::Canceller), true);
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_grant_canceller_role_not_initialized_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let new_canceller = Address::generate(&env);
    client.grant_role(&new_canceller, &RoleLabel::Canceller);
}

#[test]
fn test_revoke_canceller_role() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_canceller = Address::generate(&env);
    client.grant_role(&new_canceller, &RoleLabel::Canceller);
    assert_eq!(client.has_role(&new_canceller, &RoleLabel::Canceller), true);

    client.revoke_role(&new_canceller, &RoleLabel::Canceller);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "revoke_role"),
                    (&new_canceller, RoleLabel::Canceller,).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(
        client.has_role(&new_canceller, &RoleLabel::Canceller),
        false
    );
}

#[test]
fn test_revoke_canceller_role_duplicate_revoke() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let admin = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &admin,
    );

    let new_canceller = Address::generate(&env);
    client.grant_role(&new_canceller, &RoleLabel::Canceller);
    assert_eq!(client.has_role(&new_canceller, &RoleLabel::Canceller), true);

    assert_eq!(
        client.revoke_role(&new_canceller, &RoleLabel::Canceller),
        true
    );

    assert_eq!(
        client.revoke_role(&new_canceller, &RoleLabel::Canceller),
        false
    );

    assert_eq!(
        client.has_role(&new_canceller, &RoleLabel::Canceller),
        false
    );
}

#[test]
#[should_panic = "Error(Contract, #9)"]
fn test_revoke_canceller_role_not_initialized_should_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let new_canceller = Address::generate(&env);

    client.revoke_role(&new_canceller, &RoleLabel::Canceller);
}
