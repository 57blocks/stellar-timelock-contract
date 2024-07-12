#![cfg(test)]
extern crate std;

use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _, Events,
};
use soroban_sdk::{vec, Address, Env, Error};
use time_lock::test::{
    CallExecutedEvent, CallScheduledEvent, RoleLabel, TimeLockController, TimeLockControllerClient,
    TimeLockError, OwnerError
};
use time_lock_example_contract::test::{IncrementContract, IncrementContractClient};
use time_lock_tests_common::{current_timestamp, hash_call_data, set_env_timestamp, Context};

const MIN_DELAY: u64 = 259200; // 60 * 60 * 24 * 3 => 3 days

const DONE_TIMESTAMP: u64 = 1;

fn setup(is_self_managed: bool) -> Context {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TimeLockController);
    let client = TimeLockControllerClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let executor = Address::generate(&env);
    let owner = Address::generate(&env);
    client.initialize(
        &MIN_DELAY,
        &vec![&env, proposer.clone()],
        &vec![&env, executor.clone()],
        &owner,
        &is_self_managed,
    );
    Context {
        env,
        contract: contract_id,
        time_lock: client,
        proposer,
        executor,
        owner,
    }
}
mod initialize {
    use super::*;
    use soroban_sdk::{vec, Address, Env, IntoVal, Symbol};

    #[test]
    fn is_ok() {
        let Context {
            env,
            contract: contract_id,
            time_lock: client,
            proposer,
            executor,
            owner,
        } = setup(true);

        assert_eq!(client.get_min_delay(), MIN_DELAY);
        assert_eq!(client.has_role(&proposer, &RoleLabel::Proposer), true);
        assert_eq!(client.has_role(&proposer, &RoleLabel::Canceller), true);
        assert_eq!(client.has_role(&executor, &RoleLabel::Executor), true);

        let all_actual_events = env.events().all();
        assert_eq!(all_actual_events.len(), 6);

        assert_eq!(
            all_actual_events,
            vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "MinDelayUpdated"),).into_val(&env),
                    MIN_DELAY.into_val(&env)
                ),
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "SelfManaged"),).into_val(&env),
                    true.into_val(&env)
                ),
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "OwnerSet"),).into_val(&env),
                    owner.clone().into_val(&env)
                ),
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "RoleGranted"), 1_u32).into_val(&env),
                    proposer.clone().into_val(&env)
                ),
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "RoleGranted"), 3_u32).into_val(&env),
                    proposer.clone().into_val(&env)
                ),
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "RoleGranted"), 2_u32).into_val(&env),
                    executor.clone().into_val(&env)
                ),
            ]
        );
    }

    #[test]
    #[should_panic = "Error(Contract, #2)"]
    fn twice_should_panic() {
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
            &true,
        );
        client.initialize(
            &MIN_DELAY,
            &vec![&env, proposer.clone()],
            &vec![&env, executor.clone()],
            &Address::generate(&env),
            &true,
        );
    }

    #[test]
    fn params_invalid_should_panic() {
        let env = Env::default();
        let contract_id = env.register_contract(None, TimeLockController);
        let client = TimeLockControllerClient::new(&env, &contract_id);

        let proposer = Address::generate(&env);
        let executor = Address::generate(&env);

        assert_eq!(
            client.try_initialize(
                &0,
                &vec![&env, proposer.clone()],
                &vec![&env, executor.clone()],
                &Address::generate(&env),
                &true
            ),
            Err(Ok(Error::from_contract_error(
                TimeLockError::InvalidParams as u32
            )))
        );

        assert_eq!(
            client.try_initialize(
                &MIN_DELAY,
                &vec![&env],
                &vec![&env, executor.clone()],
                &Address::generate(&env),
                &true
            ),
            Err(Ok(Error::from_contract_error(
                TimeLockError::InvalidParams as u32
            )))
        );

        assert_eq!(
            client.try_initialize(
                &MIN_DELAY,
                &vec![&env, proposer.clone()],
                &vec![&env],
                &Address::generate(&env),
                &true
            ),
            Err(Ok(Error::from_contract_error(
                TimeLockError::InvalidParams as u32
            )))
        );

        assert_eq!(
            client.try_initialize(
                &MIN_DELAY,
                &vec![&env],
                &vec![&env],
                &Address::generate(&env),
                &true
            ),
            Err(Ok(Error::from_contract_error(
                TimeLockError::InvalidParams as u32
            )))
        );

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

        assert_eq!(
            client.try_initialize(
                &MIN_DELAY,
                &proposers,
                &vec![&env, executor.clone()],
                &Address::generate(&env),
                &true
            ),
            Err(Ok(Error::from_contract_error(
                TimeLockError::ExceedMaxCount as u32
            )))
        );

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

        assert_eq!(
            client.try_initialize(
                &MIN_DELAY,
                &vec![&env, proposer.clone()],
                &executors,
                &Address::generate(&env),
                &true
            ),
            Err(Ok(Error::from_contract_error(
                TimeLockError::ExceedMaxCount as u32
            )))
        );

        assert_eq!(
            client.try_initialize(
                &MIN_DELAY,
                &proposers,
                &executors,
                &Address::generate(&env),
                &true
            ),
            Err(Ok(Error::from_contract_error(
                TimeLockError::ExceedMaxCount as u32
            )))
        );
    }
}

mod schedule {
    use super::*;
    use soroban_sdk::{symbol_short, vec, Address, BytesN, Env, IntoVal, String, Symbol};
    #[test]
    fn is_ok() {
        let Context {
            env,
            contract: contract_id,
            time_lock: client,
            proposer,
            executor: _,
            owner: _,
        } = setup(true);

        let target = Address::generate(&env);
        let fn_name = symbol_short!("hello");
        let data = vec![&env, symbol_short!("lily").to_val()];
        let salt = BytesN::random(&env);
        let delay: u64 = MIN_DELAY + 10;

        let ledger_time = env.ledger().timestamp();
        let expect_lock_time = ledger_time + delay;

        let predecessor: Option<BytesN<32>> = None;
        let operation_id = client.schedule(
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

        assert_eq!(
            client.get_schedule_lock_time(&operation_id),
            expect_lock_time
        );

        let expected_operation_id =
            hash_call_data(&env, &target, &fn_name, &data, &predecessor, &salt);
        assert_eq!(operation_id, expected_operation_id);

        let actual_lock_time = client.get_schedule_lock_time(&operation_id);
        assert_eq!(actual_lock_time, expect_lock_time);

        let actual_events = env.events().all();
        let event_len = actual_events.len();
        assert_eq!(event_len, 7);

        assert_eq! {
            actual_events.slice(event_len - 1..),
            vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "CallScheduled"),).into_val(&env),
                    CallScheduledEvent {
                        opt_id: operation_id.clone(),
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
    fn not_proposer_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer: _,
            executor: _,
            owner: _,
        } = setup(true);

        let target = Address::generate(&env);
        let fn_name = symbol_short!("hello");
        let data = vec![&env, symbol_short!("lily").to_val()];
        let salt = BytesN::random(&env);
        let delay: u64 = MIN_DELAY + 10;

        let caller = Address::generate(&env);
        client.schedule(&caller, &target, &fn_name, &data, &salt, &None, &delay);
    }

    #[test]
    #[should_panic = "Error(Contract, #9)"]
    fn without_initialize() {
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
    fn without_initialize_self_managed() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, TimeLockController);
        let client = TimeLockControllerClient::new(&env, &contract_id);

        set_env_timestamp(&env, current_timestamp());

        let target = contract_id.clone();
        let fn_name = Symbol::new(&env, "update_min_delay");
        let new_delay = MIN_DELAY + 100;
        let data = (new_delay,).into_val(&env);
        let salt = BytesN::random(&env);
        let delay = MIN_DELAY + 10;

        let caller = Address::generate(&env);
        assert_eq!(
            client.try_schedule(&caller, &target, &fn_name, &data, &salt, &None, &delay),
            Err(Ok(Error::from_contract_error(
                OwnerError::OnlyOwner as u32
            )))
        );
    }

    #[test]
    #[should_panic = "Error(Contract, #3)"]
    fn twice_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor: _,
            owner: _,
        } = setup(true);

        let target = Address::generate(&env);
        let fn_name = symbol_short!("hello");
        let data = vec![&env, symbol_short!("lily").to_val()];
        let salt = BytesN::random(&env);
        let delay: u64 = 259201;
        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);
        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);
    }

    #[test]
    fn params_invalid_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor: _,
            owner: _,
        } = setup(true);

        let target = Address::generate(&env);
        let fn_name = symbol_short!("hello");
        let data = vec![&env, symbol_short!("lily").to_val()];
        let salt = BytesN::random(&env);
        let delay: u64 = MIN_DELAY - 10;

        assert_eq!(
            client.try_schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay),
            Err(Ok(Error::from_contract_error(
                TimeLockError::InsufficientDelay as u32
            )))
        );

        let key = String::from_str(
            &env,
            "GDDPY2EQ4S5QAB43PD6SCS45QOFYCRJG3BCYOGFRFKW7LUS65FXMG4RQ",
        );
        let target2 = Address::from_string(&key);
        let delay2 = MIN_DELAY + 10;

        assert_eq!(
            client.try_schedule(&proposer, &target2, &fn_name, &data, &salt, &None, &delay2),
            Err(Ok(Error::from_contract_error(
                TimeLockError::InvalidParams as u32
            )))
        );
    }
}

mod execute {
    use super::*;
    use soroban_sdk::{vec, Address, BytesN, IntoVal, Symbol};
    #[test]
    fn work_without_predecessor() {
        let Context {
            env,
            contract: contract_id,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

        let example_contract_id = env.register_contract(None, IncrementContract);
        let example_client = IncrementContractClient::new(&env, &example_contract_id);
        example_client.initialize(&contract_id.clone());

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);
        let predecessor: Option<BytesN<32>> = None;

        let operation_id = client.schedule(
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
        assert_eq!(client.get_schedule_lock_time(&operation_id), DONE_TIMESTAMP);
        assert_eq!(client.get_schedule_lock_time(&operation_id), DONE_TIMESTAMP);

        let actual_events = env.events().all();
        let event_len = actual_events.len();
        assert_eq!(event_len, 9);

        assert_eq! {
            actual_events.slice(event_len - 1..),
            vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "CallExecuted"),).into_val(&env),
                    CallExecutedEvent {
                        opt_id: operation_id.clone(),
                        target: target.clone(),
                        fn_name:fn_name.clone(),
                        data: data.clone(),
                    }.into_val(&env)
                )
            ]
        }
    }

    #[test]
    fn work_with_predecessor() {
        let Context {
            env,
            contract,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

        let example_contract_id = env.register_contract(None, IncrementContract);
        let example_client = IncrementContractClient::new(&env, &example_contract_id);
        example_client.initialize(&contract);
        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);
        let predecessor: Option<BytesN<32>> = None;

        let pre_operation_id = client.schedule(
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
        let operation_id = client.schedule(
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
            client.get_schedule_lock_time(&pre_operation_id),
            DONE_TIMESTAMP
        );

        client.execute(&executor, &target, &fn_name, &data, &salt2, &predecessor2);
        assert_eq!(example_client.get_count(), 2000);
        assert_eq!(client.get_schedule_lock_time(&operation_id), DONE_TIMESTAMP);
    }

    #[test]
    fn invoke_contract_return_error() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

        let example_contract_id = env.register_contract(None, IncrementContract);

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment_return_error");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);
        let predecessor: Option<BytesN<32>> = None;

        client.schedule(
            &proposer,
            &target,
            &fn_name,
            &data,
            &salt,
            &predecessor,
            &delay,
        );

        set_env_timestamp(&env, current_timestamp());

        assert_eq!(
            client.try_execute(&executor, &target, &fn_name, &data, &salt, &predecessor),
            Err(Ok(Error::from_contract_error(
                TimeLockError::ExecuteFailed as u32
            )))
        );
    }

    #[test]
    fn test_invoke_contract_panic_error() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

        let example_contract_id = env.register_contract(None, IncrementContract);

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment_with_panic_error");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);
        let predecessor: Option<BytesN<32>> = None;

        client.schedule(
            &proposer,
            &target,
            &fn_name,
            &data,
            &salt,
            &predecessor,
            &delay,
        );

        set_env_timestamp(&env, current_timestamp());

        assert_eq!(
            client.try_execute(&executor, &target, &fn_name, &data, &salt, &predecessor),
            Err(Ok(Error::from_contract_error(
                TimeLockError::ExecuteFailed as u32
            )))
        );
    }

    #[test]
    fn test_invoke_contract_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

        let example_contract_id = env.register_contract(None, IncrementContract);

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment_with_panic");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);
        let predecessor: Option<BytesN<32>> = None;

        client.schedule(
            &proposer,
            &target,
            &fn_name,
            &data,
            &salt,
            &predecessor,
            &delay,
        );

        set_env_timestamp(&env, current_timestamp());

        assert_eq!(
            client.try_execute(&executor, &target, &fn_name, &data, &salt, &predecessor),
            Err(Ok(Error::from_contract_error(
                TimeLockError::ExecuteFailed as u32
            )))
        );
    }

    #[test]
    #[should_panic = "Error(Contract, #6)"]
    fn predecessor_not_ready_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

        let example_contract_id = env.register_contract(None, IncrementContract);

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = 259201;
        let salt = BytesN::random(&env);

        let pre_operation_id =
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
    fn predecessor_not_exist_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

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
    fn not_executor_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor: _,
            owner: _,
        } = setup(true);

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
    fn no_scheduled_operation_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer: _,
            executor,
            owner: _,
        } = setup(true);

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
    fn operation_not_ready_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

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
    fn twice_operation_should_panic() {
        let Context {
            env,
            contract,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

        let example_contract_id = env.register_contract(None, IncrementContract);
        let example_client = IncrementContractClient::new(&env, &example_contract_id);
        example_client.initialize(&contract.clone());
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
}

mod cancel {
    use super::*;
    use soroban_sdk::{symbol_short, vec, Address, BytesN, IntoVal, Symbol};

    #[test]
    fn is_ok() {
        let Context {
            env,
            contract: contract_id,
            time_lock: client,
            proposer,
            executor: _,
            owner: _,
        } = setup(true);

        let target = Address::generate(&env);
        let fn_name = symbol_short!("hello");
        let data = vec![&env, symbol_short!("lily").to_val()];
        let salt = BytesN::random(&env);
        let delay: u64 = MIN_DELAY + 10;
        let predecessor: Option<BytesN<32>> = None;
        let operation_id = client.schedule(
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

        assert_eq!(client.get_schedule_lock_time(&operation_id), 0);
        assert_eq!(client.get_schedule_lock_time(&operation_id), 0);

        let actual_events = env.events().all();
        let event_len = actual_events.len();
        assert_eq!(event_len, 8);

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
    fn operation_waiting_is_ok() {
        let Context {
            env,
            contract: contract_id,
            time_lock: client,
            proposer,
            executor: _,
            owner: _,
        } = setup(true);

        set_env_timestamp(&env, current_timestamp());

        let target = Address::generate(&env);
        let fn_name = symbol_short!("hello");
        let data = vec![&env, symbol_short!("lily").to_val()];
        let salt = BytesN::random(&env);
        let delay: u64 = MIN_DELAY + 10;
        let predecessor: Option<BytesN<32>> = None;

        let operation_id = client.schedule(
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

        assert_eq!(client.get_schedule_lock_time(&operation_id), 0);
        assert_eq!(client.get_schedule_lock_time(&operation_id), 0);

        let actual_events = env.events().all();
        let event_len = actual_events.len();
        assert_eq!(event_len, 8);

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
    fn not_canceler_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor: _,
            owner: _,
        } = setup(true);

        let target = Address::generate(&env);
        let fn_name = symbol_short!("hello");
        let data = vec![&env, symbol_short!("lily").to_val()];
        let salt = BytesN::random(&env);
        let delay: u64 = MIN_DELAY + 10;

        let operation_id =
            client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

        set_env_timestamp(&env, current_timestamp());

        let caller = Address::generate(&env);
        client.cancel(&caller, &operation_id);
    }
    #[test]
    #[should_panic = "Error(Contract, #8)"]
    fn unset_operation_should_panic() {
        let Context {
            env,
            contract: _,
            time_lock: client,
            proposer,
            executor: _,
            owner: _,
        } = setup(true);

        let operation_id = BytesN::random(&env);

        client.cancel(&proposer, &operation_id);
    }

    #[test]
    #[should_panic = "Error(Contract, #8)"]
    fn executed_operation_should_panic() {
        let Context {
            env,
            contract,
            time_lock: client,
            proposer,
            executor,
            owner: _,
        } = setup(true);

        let example_contract_id = env.register_contract(None, IncrementContract);
        let example_client = IncrementContractClient::new(&env, &example_contract_id);
        example_client.initialize(&contract.clone());

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);

        let operation_id =
            client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

        set_env_timestamp(&env, current_timestamp());

        let predecessor: Option<BytesN<32>> = None;
        client.execute(&executor, &target, &fn_name, &data, &salt, &predecessor);

        client.cancel(&proposer, &operation_id);
    }
}

mod update_min_delay {
    use super::*;
    use soroban_sdk::{vec, IntoVal, Symbol};
    #[test]
    fn is_ok() {
        let Context {
            env,
            contract: contract_id,
            time_lock: client,
            proposer: _,
            executor: _,
            owner: _,
        } = setup(true);

        let delay = MIN_DELAY + 10;
        client.update_min_delay(&delay);
        assert_eq!(delay, client.get_min_delay());

        let actual_events = env.events().all();
        let event_len = actual_events.len();
        assert_eq!(event_len, 7);

        assert_eq! {
            actual_events.slice(event_len - 1..),
            vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "MinDelayUpdated"),).into_val(&env),
                    delay.into_val(&env)
                )
            ]
        }
    }

    #[test]
    fn not_owner_should_panic() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, TimeLockController);
        let client = TimeLockControllerClient::new(&env, &contract_id);
        let delay = MIN_DELAY + 10;

        assert_eq!(
            client.try_update_min_delay(&delay),
            Err(Ok(Error::from_contract_error(
                TimeLockError::NotPermitted as u32
            )))
        );
    }
}

mod grant_role {
    use super::*;

    mod grant_proposer_role {
        use super::*;
        use soroban_sdk::{Address, Env, IntoVal, Symbol};

        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(true);

            let new_proposer = Address::generate(&env);
            client.grant_role(&new_proposer, &RoleLabel::Proposer);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
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
        fn duplicate_add() {
            let Context {
                env,
                contract: _,
                time_lock: client,
                proposer: _,
                executor: _,
                owner: _,
            } = setup(true);

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
        fn not_initialized_should_panic() {
            let env = Env::default();
            env.mock_all_auths();

            let contract_id = env.register_contract(None, TimeLockController);
            let client = TimeLockControllerClient::new(&env, &contract_id);

            let new_proposer = Address::generate(&env);
            client.grant_role(&new_proposer, &RoleLabel::Proposer);
        }
    }

    mod grant_executor_role {
        use super::*;
        use soroban_sdk::{Address, Env, IntoVal, Symbol};
        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(true);

            let new_executor = Address::generate(&env);
            client.grant_role(&new_executor, &RoleLabel::Executor);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
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
        fn duplicate_add() {
            let Context {
                env,
                contract: _,
                time_lock: client,
                proposer: _,
                executor: _,
                owner: _,
            } = setup(true);

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
        fn not_initialized_should_panic() {
            let env = Env::default();
            env.mock_all_auths();

            let contract_id = env.register_contract(None, TimeLockController);
            let client = TimeLockControllerClient::new(&env, &contract_id);

            let new_executor = Address::generate(&env);
            client.grant_role(&new_executor, &RoleLabel::Executor);
        }
    }

    mod grant_canceller_role {
        use super::*;
        use soroban_sdk::{Address, Env, IntoVal, Symbol};

        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(true);

            let new_canceller = Address::generate(&env);
            client.grant_role(&new_canceller, &RoleLabel::Canceller);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
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
        fn duplicate_add() {
            let Context {
                env,
                contract: _,
                time_lock: client,
                proposer: _,
                executor: _,
                owner: _,
            } = setup(true);

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
        fn not_initialized_should_panic() {
            let env = Env::default();
            env.mock_all_auths();

            let contract_id = env.register_contract(None, TimeLockController);
            let client = TimeLockControllerClient::new(&env, &contract_id);

            let new_canceller = Address::generate(&env);
            client.grant_role(&new_canceller, &RoleLabel::Canceller);
        }
    }
}

mod revoke_role {
    use super::*;

    mod revoke_proposer_role {
        use super::*;
        use soroban_sdk::{Address, Env, IntoVal, Symbol};

        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(true);

            let new_proposer = Address::generate(&env);
            client.grant_role(&new_proposer, &RoleLabel::Proposer);
            assert_eq!(client.has_role(&new_proposer, &RoleLabel::Proposer), true);

            client.revoke_role(&new_proposer, &RoleLabel::Proposer);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
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
        fn duplicate_revoke() {
            let Context {
                env,
                contract: _,
                time_lock: client,
                proposer: _,
                executor: _,
                owner: _,
            } = setup(true);

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
        fn not_initialized_should_panic() {
            let env = Env::default();
            env.mock_all_auths();

            let contract_id = env.register_contract(None, TimeLockController);
            let client = TimeLockControllerClient::new(&env, &contract_id);

            let new_proposer = Address::generate(&env);

            client.revoke_role(&new_proposer, &RoleLabel::Proposer);
        }
    }

    mod revoke_executor_role {
        use super::*;
        use soroban_sdk::{Address, Env, IntoVal, Symbol};

        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(true);

            let new_executor = Address::generate(&env);
            client.grant_role(&new_executor, &RoleLabel::Executor);
            assert_eq!(client.has_role(&new_executor, &RoleLabel::Executor), true);

            client.revoke_role(&new_executor, &RoleLabel::Executor);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
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
        fn duplicate_revoke() {
            let Context {
                env,
                contract: _,
                time_lock: client,
                proposer: _,
                executor: _,
                owner: _,
            } = setup(true);

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
        fn not_initialized_should_panic() {
            let env = Env::default();
            env.mock_all_auths();

            let contract_id = env.register_contract(None, TimeLockController);
            let client = TimeLockControllerClient::new(&env, &contract_id);

            let new_executor = Address::generate(&env);

            client.revoke_role(&new_executor, &RoleLabel::Executor);
        }
    }

    mod revoke_canceller_role {
        use super::*;
        use soroban_sdk::{Address, Env, IntoVal, Symbol};

        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(true);

            let new_canceller = Address::generate(&env);
            client.grant_role(&new_canceller, &RoleLabel::Canceller);
            assert_eq!(client.has_role(&new_canceller, &RoleLabel::Canceller), true);

            client.revoke_role(&new_canceller, &RoleLabel::Canceller);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
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
        fn duplicate_revoke() {
            let Context {
                env,
                contract: _,
                time_lock: client,
                proposer: _,
                executor: _,
                owner: _,
            } = setup(true);

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
        fn not_initialized_should_panic() {
            let env = Env::default();
            env.mock_all_auths();

            let contract_id = env.register_contract(None, TimeLockController);
            let client = TimeLockControllerClient::new(&env, &contract_id);

            let new_canceller = Address::generate(&env);

            client.revoke_role(&new_canceller, &RoleLabel::Canceller);
        }
    }
}

mod update_owner {
    use super::*;
    use soroban_sdk::{Address, Env, IntoVal, Symbol};

    #[test]
    fn is_ok() {
        let Context {
            env,
            contract: contract_id,
            time_lock: client,
            proposer: _,
            executor: _,
            owner,
        } = setup(true);

        let new_owner = Address::generate(&env);
        client.update_owner(&new_owner);

        assert_eq!(
            env.auths(),
            std::vec![(
                owner.clone(),
                AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        contract_id.clone(),
                        Symbol::new(&env, "update_owner"),
                        (&new_owner,).into_val(&env)
                    )),
                    sub_invocations: std::vec![]
                }
            )]
        );
    }

    #[test]
    fn not_owner_should_panic() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, TimeLockController);
        let client = TimeLockControllerClient::new(&env, &contract_id);

        let new_owner = Address::generate(&env);
        assert_eq!(
            client.try_update_owner(&new_owner),
            Err(Ok(Error::from_contract_error(
                TimeLockError::NotPermitted as u32
            )))
        );
    }

    #[test]
    fn not_initialized_should_panic() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, TimeLockController);
        let client = TimeLockControllerClient::new(&env, &contract_id);

        let new_owner = Address::generate(&env);
        assert_eq!(
            client.try_update_owner(&new_owner),
            Err(Ok(Error::from_contract_error(
                TimeLockError::NotPermitted as u32
            )))
        );
    }
}

mod integrate_test_with_increment {
    use super::*;
    use soroban_sdk::{vec, Address, BytesN, Env, IntoVal, String, Symbol};
    use time_lock_example_contract::test::ContractConfig;

    #[test]
    fn test_increment_when_time_lock() {
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
            &true,
        );

        let example_contract_id = env.register_contract(None, IncrementContract);
        let example_client = IncrementContractClient::new(&env, &example_contract_id);

        let owner = contract_id.clone();
        example_client.initialize(&owner);

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment_owner");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);

        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

        set_env_timestamp(&env, current_timestamp());

        client.execute(&executor, &target, &fn_name, &data, &salt, &None);

        let count = example_client.get_count();
        assert_eq!(count, 1000);
    }

    #[test]
    #[should_panic]
    fn test_increment_when_time_lock_but_caller_not_time_lock() {
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
            &true,
        );

        let example_contract_id = env.register_contract(None, IncrementContract);
        let example_client = IncrementContractClient::new(&env, &example_contract_id);

        let owner = Address::generate(&env);
        example_client.initialize(&owner);

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment_owner");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);

        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

        set_env_timestamp(&env, current_timestamp());

        client.execute(&executor, &target, &fn_name, &data, &salt, &None);
    }

    #[test]
    fn test_increment_when_time_lock_with_no_args() {
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
            &true,
        );

        let example_contract_id = env.register_contract(None, IncrementContract);
        let example_client = IncrementContractClient::new(&env, &example_contract_id);
        example_client.initialize(&contract_id.clone());
        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment_five");
        let data = vec![&env];
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);

        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

        set_env_timestamp(&env, current_timestamp());

        client.execute(&executor, &target, &fn_name, &data, &salt, &None);

        let count = example_client.get_count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_account_increment_when_time_lock_with_predecessor() {
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
            &true,
        );

        let example_contract_id = env.register_contract(None, IncrementContract);
        let example_client = IncrementContractClient::new(&env, &example_contract_id);

        let owner = contract_id.clone();
        example_client.initialize(&owner);

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "increment_owner");
        let sum: u32 = 1000;
        let data = (sum,).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);

        let operation_id =
            client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

        let fn_name_2 = Symbol::new(&env, "increment_account_total");
        let account = Address::generate(&env);
        let data_2 = (account.clone(), sum).into_val(&env);
        let salt_2 = BytesN::random(&env);
        let predecessor = Some(operation_id);

        client.schedule(
            &proposer,
            &target,
            &fn_name_2,
            &data_2,
            &salt_2,
            &predecessor,
            &delay,
        );

        set_env_timestamp(&env, current_timestamp());

        client.execute(&executor, &target, &fn_name, &data, &salt, &None);
        client.execute(
            &executor,
            &target,
            &fn_name_2,
            &data_2,
            &salt_2,
            &predecessor,
        );

        let count = example_client.get_count();
        assert_eq!(count, 1000);

        let account_total = example_client.get_account_total(&account);
        assert_eq!(account_total, 1000);
    }

    #[test]
    fn test_increment_contract_config_when_time_lock() {
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
            &true,
        );

        let example_contract_id = env.register_contract(None, IncrementContract);
        let example_client = IncrementContractClient::new(&env, &example_contract_id);

        let owner = contract_id.clone();
        example_client.initialize(&owner);

        let target = example_contract_id.clone();
        let fn_name = Symbol::new(&env, "set_contract_info");
        let key = BytesN::random(&env);
        let config = ContractConfig {
            owner: Address::generate(&env),
            name: String::from_str(&env, "IncrementContract"),
        };
        let data = (key.clone(), config.clone()).into_val(&env);
        let delay: u64 = MIN_DELAY + 10;
        let salt = BytesN::random(&env);

        client.schedule(&proposer, &target, &fn_name, &data, &salt, &None, &delay);

        set_env_timestamp(&env, current_timestamp());

        client.execute(&executor, &target, &fn_name, &data, &salt, &None);

        let contract_config = example_client.get_contract_info(&key);
        assert_eq!(contract_config.name, config.name);
        assert_eq!(contract_config.owner, config.owner);
    }
}

mod updata_self_with_time_lock {
    use super::*;
    use soroban_sdk::{BytesN, IntoVal, Symbol};

    #[test]
    fn not_exist_function() {
        let Context {
            env,
            contract: contract_id,
            time_lock: client,
            proposer: _,
            executor: _,
            owner,
        } = setup(true);

        let target = contract_id.clone();
        let fn_name = Symbol::new(&env, "not_exist_function");
        let data = ().into_val(&env);
        let salt = BytesN::random(&env);
        let delay: u64 = MIN_DELAY + 10;

        assert_eq!(
            client.try_schedule(&owner, &target, &fn_name, &data, &salt, &None, &delay),
            Err(Ok(Error::from_contract_error(
                TimeLockError::InvalidFuncName as u32
            )))
        );
    }

    mod update_min_delay {
        use super::*;
        use soroban_sdk::{BytesN, IntoVal, Symbol};

        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(false);

            let target = contract_id.clone();
            let fn_name = Symbol::new(&env, "update_min_delay");
            let new_delay = MIN_DELAY + 100;
            let data = (new_delay,).into_val(&env);
            let salt = BytesN::random(&env);
            let delay = MIN_DELAY + 10;
            let predecessor = None;

            let operation_id =
                client.schedule(&owner, &target, &fn_name, &data, &salt, &None, &delay);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            contract_id.clone(),
                            Symbol::new(&env, "schedule"),
                            (
                                &owner,
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

            set_env_timestamp(&env, current_timestamp());

            client.execute(&owner, &target, &fn_name, &data, &salt, &predecessor);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            contract_id.clone(),
                            Symbol::new(&env, "execute"),
                            (
                                &owner,
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

            assert_eq!(client.get_min_delay(), new_delay);
            assert_eq!(client.get_schedule_lock_time(&operation_id), DONE_TIMESTAMP);
        }

        #[test]
        fn invoke_params_invalid() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(false);

            let target = contract_id.clone();
            let fn_name = Symbol::new(&env, "update_min_delay");
            let new_delay = -100_i64;
            let data = ().into_val(&env);
            let data_1 = (new_delay,).into_val(&env);
            let salt = BytesN::random(&env);
            let delay = MIN_DELAY + 10;
            let predecessor = None;

            client.schedule(
                &owner,
                &target,
                &fn_name,
                &data,
                &salt,
                &predecessor,
                &delay,
            );
            client.schedule(
                &owner,
                &target,
                &fn_name,
                &data_1,
                &salt,
                &predecessor,
                &delay,
            );

            set_env_timestamp(&env, current_timestamp());

            assert_eq!(
                client.try_execute(&owner, &target, &fn_name, &data, &salt, &predecessor),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::InvalidParams as u32
                )))
            );

            assert_eq!(
                client.try_execute(&owner, &target, &fn_name, &data_1, &salt, &predecessor),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::InvalidParams as u32
                )))
            );
        }

        #[test]
        fn direct_call() {
            let Context {
                env: _,
                contract: _,
                time_lock: client,
                proposer: _,
                executor: _,
                owner: _,
            } = setup(false);

            let new_delay = MIN_DELAY + 100;
            assert_eq!(
                client.try_update_min_delay(&new_delay),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::NotPermitted as u32
                )))
            );
        }
    }

    mod update_owner {
        use super::*;
        use soroban_sdk::{Address, BytesN, IntoVal, Symbol};

        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(false);

            let target = contract_id.clone();
            let fn_name = Symbol::new(&env, "update_owner");
            let new_owner = Address::generate(&env);
            let data = (new_owner.clone(),).into_val(&env);
            let salt = BytesN::random(&env);
            let delay = MIN_DELAY + 10;
            let predecessor = None;

            let operation_id =
                client.schedule(&owner, &target, &fn_name, &data, &salt, &None, &delay);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            contract_id.clone(),
                            Symbol::new(&env, "schedule"),
                            (
                                &owner,
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

            set_env_timestamp(&env, current_timestamp());

            client.execute(&owner, &target, &fn_name, &data, &salt, &predecessor);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            contract_id.clone(),
                            Symbol::new(&env, "execute"),
                            (
                                &owner,
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

            assert_eq!(client.is_owner(&owner), false);
            assert_eq!(client.is_owner(&new_owner), true);
            assert_eq!(client.get_schedule_lock_time(&operation_id), DONE_TIMESTAMP);
        }

        #[test]
        fn invoke_params_invalid() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(false);

            let target = contract_id.clone();
            let fn_name = Symbol::new(&env, "update_owner");
            let new_owner = 100_u64;
            let data = ().into_val(&env);
            let data_1 = (new_owner,).into_val(&env);
            let salt = BytesN::random(&env);
            let delay = MIN_DELAY + 10;
            let predecessor = None;

            client.schedule(
                &owner,
                &target,
                &fn_name,
                &data,
                &salt,
                &predecessor,
                &delay,
            );
            client.schedule(
                &owner,
                &target,
                &fn_name,
                &data_1,
                &salt,
                &predecessor,
                &delay,
            );

            set_env_timestamp(&env, current_timestamp());

            assert_eq!(
                client.try_execute(&owner, &target, &fn_name, &data, &salt, &predecessor),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::InvalidParams as u32
                )))
            );

            assert_eq!(
                client.try_execute(&owner, &target, &fn_name, &data_1, &salt, &predecessor),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::InvalidParams as u32
                )))
            );
        }

        #[test]
        fn direct_call() {
            let Context {
                env,
                contract: _,
                time_lock: client,
                proposer: _,
                executor: _,
                owner: _,
            } = setup(false);

            let new_owner = Address::generate(&env);
            assert_eq!(
                client.try_update_owner(&new_owner),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::NotPermitted as u32
                )))
            );
        }
    }

    mod grant_role {
        use super::*;
        use soroban_sdk::{Address, BytesN, IntoVal, Symbol};

        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(false);

            let target = contract_id.clone();
            let fn_name = Symbol::new(&env, "grant_role");
            let new_role = RoleLabel::Proposer;
            let new_address = Address::generate(&env);
            let data = (new_address.clone(), new_role.clone()).into_val(&env);
            let salt = BytesN::random(&env);
            let delay = MIN_DELAY + 10;
            let predecessor = None;

            let operation_id =
                client.schedule(&owner, &target, &fn_name, &data, &salt, &None, &delay);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            contract_id.clone(),
                            Symbol::new(&env, "schedule"),
                            (
                                &owner,
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

            set_env_timestamp(&env, current_timestamp());

            client.execute(&owner, &target, &fn_name, &data, &salt, &predecessor);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            contract_id.clone(),
                            Symbol::new(&env, "execute"),
                            (
                                &owner,
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

            assert_eq!(client.has_role(&new_address, &new_role), true);
            assert_eq!(client.get_schedule_lock_time(&operation_id), DONE_TIMESTAMP);
        }

        #[test]
        fn invoke_params_invalid() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer: _,
                executor: _,
                owner,
            } = setup(false);

            let target = contract_id.clone();
            let fn_name = Symbol::new(&env, "grant_role");
            let valid_role = RoleLabel::Proposer;
            let invalid_role = -100_i64;
            let invalid_address = 100_u64;
            let valid_address = Address::generate(&env);
            let predecessor = None;
            let data = ().into_val(&env);
            let data_1 = (invalid_address.clone(), valid_role.clone()).into_val(&env);
            let data_2 = (valid_address.clone(), invalid_role.clone()).into_val(&env);
            let data_3 = (invalid_address.clone(), invalid_role.clone()).into_val(&env);
            let data_4 = (valid_address.clone(),).into_val(&env);
            let salt = BytesN::random(&env);
            let delay = MIN_DELAY + 10;

            client.schedule(
                &owner,
                &target,
                &fn_name,
                &data,
                &salt,
                &predecessor,
                &delay,
            );
            client.schedule(
                &owner,
                &target,
                &fn_name,
                &data_1,
                &salt,
                &predecessor,
                &delay,
            );
            client.schedule(
                &owner,
                &target,
                &fn_name,
                &data_2,
                &salt,
                &predecessor,
                &delay,
            );
            client.schedule(
                &owner,
                &target,
                &fn_name,
                &data_3,
                &salt,
                &predecessor,
                &delay,
            );
            client.schedule(
                &owner,
                &target,
                &fn_name,
                &data_4,
                &salt,
                &predecessor,
                &delay,
            );

            set_env_timestamp(&env, current_timestamp());

            assert_eq!(
                client.try_execute(&owner, &target, &fn_name, &data, &salt, &predecessor),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::InvalidParams as u32
                )))
            );

            assert_eq!(
                client.try_execute(&owner, &target, &fn_name, &data_1, &salt, &predecessor),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::InvalidParams as u32
                )))
            );

            assert_eq!(
                client.try_execute(&owner, &target, &fn_name, &data_2, &salt, &predecessor),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::InvalidParams as u32
                )))
            );

            assert_eq!(
                client.try_execute(&owner, &target, &fn_name, &data_3, &salt, &predecessor),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::InvalidParams as u32
                )))
            );

            assert_eq!(
                client.try_execute(&owner, &target, &fn_name, &data_4, &salt, &predecessor),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::InvalidParams as u32
                )))
            );
        }

        #[test]
        fn direct_call() {
            let Context {
                env,
                contract: _,
                time_lock: client,
                proposer: _,
                executor: _,
                owner: _,
            } = setup(false);

            let new_role = RoleLabel::Proposer;
            let new_address = Address::generate(&env);
            assert_eq!(
                client.try_grant_role(&new_address, &new_role),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::NotPermitted as u32
                )))
            );
        }
    }

    mod revoke_role {
        use super::*;
        use soroban_sdk::{BytesN, IntoVal, Symbol};

        #[test]
        fn is_ok() {
            let Context {
                env,
                contract: contract_id,
                time_lock: client,
                proposer,
                executor: _,
                owner,
            } = setup(false);

            let target = contract_id.clone();
            let fn_name = Symbol::new(&env, "revoke_role");
            let role = RoleLabel::Proposer;
            let data = (proposer.clone(), role.clone()).into_val(&env);
            let salt = BytesN::random(&env);
            let delay = MIN_DELAY + 10;
            let predecessor = None;

            assert_eq!(client.has_role(&proposer, &role), true);

            let operation_id =
                client.schedule(&owner, &target, &fn_name, &data, &salt, &None, &delay);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            contract_id.clone(),
                            Symbol::new(&env, "schedule"),
                            (
                                &owner,
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

            set_env_timestamp(&env, current_timestamp());

            client.execute(&owner, &target, &fn_name, &data, &salt, &predecessor);

            assert_eq!(
                env.auths(),
                std::vec![(
                    owner.clone(),
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            contract_id.clone(),
                            Symbol::new(&env, "execute"),
                            (
                                &owner,
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

            assert_eq!(client.has_role(&proposer, &role), false);
            assert_eq!(client.get_schedule_lock_time(&operation_id), DONE_TIMESTAMP);
        }

        #[test]
        fn direct_call() {
            let Context {
                env: _,
                contract: _,
                time_lock: client,
                proposer,
                executor: _,
                owner: _,
            } = setup(false);

            let role = RoleLabel::Proposer;
            assert_eq!(
                client.try_revoke_role(&proposer, &role),
                Err(Ok(Error::from_contract_error(
                    TimeLockError::NotPermitted as u32
                )))
            );
        }
    }
}
