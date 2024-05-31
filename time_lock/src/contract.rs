use crate::access_control_base::{grant_role, has_role, revoke_role};
use crate::admin::{has_admin, read_admin, set_admin};
use crate::config::{
    get_operation_state, hash_call, hash_call_batch, CallExecutedEvent, CallScheduledEvent,
    DataKey, OperationState, RoleKey, RoleLabel, TimeLockError, BATCH_MAX, DONE_TIMESTAMP,
    MAX_ACCOUNTS_NUM,
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, vec, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

#[contract]
pub struct TimeLockController;

#[contractimpl]
impl TimeLockController {
    /**
     * Initialize the contract with the minimum delay, proposers, executors, and admin.
     */
    pub fn initialize(
        env: Env,
        min_delay: u64,
        proposers: Vec<Address>,
        executors: Vec<Address>,
        admin: Address,
    ) {
        if env.storage().instance().has(&DataKey::IsInit) {
            panic_with_error!(&env, TimeLockError::AlreadyInitialized);
        }

        if min_delay == 0 {
            panic_with_error!(&env, TimeLockError::InvalidParams);
        }

        if proposers.len() == 0 || executors.len() == 0 {
            panic_with_error!(&env, TimeLockError::InvalidParams);
        }

        if proposers.len() > MAX_ACCOUNTS_NUM || executors.len() > MAX_ACCOUNTS_NUM {
            panic_with_error!(&env, TimeLockError::ExceedMaxCount);
        }

        env.storage().instance().set(&DataKey::IsInit, &());
        env.storage().instance().set(&DataKey::MinDelay, &min_delay);
        set_admin(&env, &admin);

        for proposer in proposers.iter() {
            let p_role = RoleKey::Proposers(proposer.clone());
            let c_role = RoleKey::Cancellers(proposer.clone());
            grant_role(&env, &p_role);
            grant_role(&env, &c_role);
            env.events()
                .publish((Symbol::new(&env, "RoleGranted"), p_role), &proposer);
            env.events()
                .publish((Symbol::new(&env, "RoleGranted"), c_role), &proposer);
        }

        for executor in executors.iter() {
            let e_role = RoleKey::Executors(executor.clone());
            grant_role(&env, &e_role);
            env.events()
                .publish((Symbol::new(&env, "RoleGranted"), e_role), &executor);
        }
    }

    /**
     * Schedule a call to the target contract with the given data.
     */
    pub fn schedule(
        env: Env,
        proposer: Address,
        target: Address,
        fn_name: Symbol,
        data: Vec<Val>,
        salt: BytesN<32>,
        predecessor: Option<BytesN<32>>,
        delay: u64,
    ) -> (BytesN<32>, u64) {
        proposer.require_auth();
        if !Self::has_role(&env, proposer.clone(), RoleLabel::Proposer) {
            panic_with_error!(&env, TimeLockError::NotPermitted);
        }

        let operation_id = hash_call(&env, &target, &fn_name, &data, &salt, &predecessor);
        let schedule = Self::_add_operation(&env, operation_id.clone(), delay);

        let actual_predecessor = match predecessor {
            Some(predecessor) => predecessor,
            None => BytesN::from_array(&env, &[0_u8; 32]),
        };

        env.events().publish(
            (Symbol::new(&env, "CallScheduled"),),
            CallScheduledEvent {
                opt_id: operation_id.clone(),
                index: 0,
                target,
                fn_name,
                data,
                predecessor: actual_predecessor,
                delay,
            },
        );

        (operation_id, schedule)
    }

    /**
     * Schedule a batch of calls to the target contracts with the given data.
     */
    pub fn schedule_batch(
        env: Env,
        proposer: Address,
        targets: Vec<Address>,
        fn_names: Vec<Symbol>,
        data: Vec<Vec<Val>>,
        salt: BytesN<32>,
        predecessor: Option<BytesN<32>>,
        delay: u64,
    ) -> (BytesN<32>, u64) {
        proposer.require_auth();

        if !Self::has_role(&env, proposer.clone(), RoleLabel::Proposer) {
            panic_with_error!(&env, TimeLockError::NotPermitted);
        }

        if targets.len() != fn_names.len() || targets.len() != data.len() {
            panic_with_error!(&env, TimeLockError::InvalidParams);
        }

        if targets.len() > BATCH_MAX {
            panic_with_error!(&env, TimeLockError::ExceedMaxCount);
        }

        let operation_id = hash_call_batch(&env, &targets, &fn_names, &data, &salt, &predecessor);
        let schedule = Self::_add_operation(&env, operation_id.clone(), delay);

        let actual_predecessor = match predecessor {
            Some(predecessor) => predecessor,
            None => BytesN::from_array(&env, &[0_u8; 32]),
        };

        for i in 0..targets.len() {
            let target = targets.get(i).unwrap();
            let fn_name = fn_names.get(i).unwrap();
            let data = data.get(i).unwrap();
            env.events().publish(
                (Symbol::new(&env, "CallScheduled"),),
                CallScheduledEvent {
                    opt_id: operation_id.clone(),
                    index: i,
                    target,
                    fn_name,
                    data,
                    predecessor: actual_predecessor.clone(),
                    delay,
                },
            );
        }

        (operation_id, schedule)
    }

    /**
     * Execute the scheduled call if the time is ready.
     */
    pub fn execute(
        env: Env,
        executor: Address,
        target: Address,
        fn_name: Symbol,
        data: Vec<Val>,
        salt: BytesN<32>,
        predecessor: Option<BytesN<32>>,
    ) -> Val {
        executor.require_auth();
        if !Self::has_role(&env, executor.clone(), RoleLabel::Executor) {
            panic_with_error!(&env, TimeLockError::NotPermitted);
        }

        let operation_id = hash_call(&env, &target, &fn_name, &data, &salt, &predecessor);
        Self::_execute_check(&env, operation_id.clone(), predecessor);

        // Execute the operation
        let res: Val = env.invoke_contract(&target, &fn_name, data.clone());

        // Update the state of the operation to executed
        env.storage()
            .instance()
            .set(&DataKey::Scheduler(operation_id.clone()), &DONE_TIMESTAMP);

        env.events().publish(
            (Symbol::new(&env, "CallExecuted"),),
            CallExecutedEvent {
                opt_id: operation_id,
                index: 0,
                target,
                fn_name,
                data,
            },
        );
        res
    }

    /**
     * Execute the scheduled batch of calls if the time is ready.
     */
    pub fn execute_batch(
        env: Env,
        executor: Address,
        targets: Vec<Address>,
        fn_names: Vec<Symbol>,
        data: Vec<Vec<Val>>,
        salt: BytesN<32>,
        predecessor: Option<BytesN<32>>,
    ) -> Vec<Val> {
        executor.require_auth();

        if !Self::has_role(&env, executor.clone(), RoleLabel::Executor) {
            panic_with_error!(&env, TimeLockError::NotPermitted);
        }

        if targets.len() != fn_names.len() || targets.len() != data.len() {
            panic_with_error!(&env, TimeLockError::InvalidParams);
        }

        let operation_id = hash_call_batch(&env, &targets, &fn_names, &data, &salt, &predecessor);
        Self::_execute_check(&env, operation_id.clone(), predecessor);

        let mut res = Vec::new(&env);
        for i in 0..targets.len() {
            let target = targets.get(i).unwrap();
            let fn_name = fn_names.get(i).unwrap();
            let data = data.get(i).unwrap();
            let val = env.invoke_contract(&target, &fn_name, data.clone());
            env.events().publish(
                (Symbol::new(&env, "CallExecuted"),),
                CallExecutedEvent {
                    opt_id: operation_id.clone(),
                    index: i,
                    target,
                    fn_name,
                    data,
                },
            );
            res.push_back(val);
        }

        env.storage()
            .instance()
            .set(&DataKey::Scheduler(operation_id.clone()), &DONE_TIMESTAMP);
        res
    }

    pub fn cancel(env: Env, canceller: Address, operation_id: BytesN<32>) {
        canceller.require_auth();

        if !Self::has_role(&env, canceller.clone(), RoleLabel::Canceller) {
            panic_with_error!(&env, TimeLockError::NotPermitted);
        }

        let ledger_time = env.ledger().timestamp();
        let lock_time = Self::get_schedule_lock_time(&env, operation_id.clone());
        let state = get_operation_state(ledger_time, lock_time);
        if state == OperationState::Ready || state == OperationState::Waiting {
            env.storage()
                .instance()
                .remove(&DataKey::Scheduler(operation_id.clone()));
        } else {
            panic_with_error!(&env, TimeLockError::InvalidStatus);
        }

        env.events()
            .publish((Symbol::new(&env, "OperationCancelled"),), operation_id);
    }

    pub fn update_min_delay(env: Env, delay: u64, salt: BytesN<32>) -> u64 {
        let operation_id = hash_call(
            &env,
            &env.current_contract_address(),
            &Symbol::new(&env, "update_min_delay"),
            &vec![&env, delay.into_val(&env), salt.into_val(&env)],
            &salt,
            &None,
        );
        let ledger_time = env.ledger().timestamp();
        let lock_time = Self::get_schedule_lock_time(&env, operation_id.clone());
        if get_operation_state(ledger_time, lock_time) != OperationState::Ready {
            panic_with_error!(&env, TimeLockError::TimeNotReady);
        }

        env.storage().instance().set(&DataKey::MinDelay, &delay);

        env.storage()
            .instance()
            .set(&DataKey::Scheduler(operation_id), &DONE_TIMESTAMP);

        env.events()
            .publish((Symbol::new(&env, "MinDelayUpdated"),), delay);

        delay
    }

    pub fn grant_role(env: Env, account: Address, role: RoleLabel) -> bool {
        if !has_admin(&env) {
            panic_with_error!(&env, TimeLockError::NotPermitted);
        }

        let admin = read_admin(&env);
        admin.require_auth();

        let key: RoleKey;
        match role {
            RoleLabel::Proposer => key = RoleKey::Proposers(account.clone()),
            RoleLabel::Executor => key = RoleKey::Executors(account.clone()),
            RoleLabel::Canceller => key = RoleKey::Cancellers(account.clone()),
        }

        let res = grant_role(&env, &key);
        env.events()
            .publish((Symbol::new(&env, "RoleGranted"), role), &account);

        res
    }

    pub fn revoke_role(env: Env, account: Address, role: RoleLabel) -> bool {
        if !has_admin(&env) {
            panic_with_error!(&env, TimeLockError::NotPermitted);
        }

        let admin = read_admin(&env);
        admin.require_auth();

        let key: RoleKey;
        match role {
            RoleLabel::Proposer => key = RoleKey::Proposers(account.clone()),
            RoleLabel::Executor => key = RoleKey::Executors(account.clone()),
            RoleLabel::Canceller => key = RoleKey::Cancellers(account.clone()),
        }
        let res = revoke_role(&env, &key);
        env.events()
            .publish((Symbol::new(&env, "RoleRevoked"), role), &account);

        res
    }

    pub fn get_min_delay(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::MinDelay)
            .unwrap_or(0)
    }

    pub fn get_schedule_lock_time(env: &Env, operation_id: BytesN<32>) -> u64 {
        let key = DataKey::Scheduler(operation_id);
        if let Some(schedule) = env.storage().instance().get::<DataKey, u64>(&key) {
            schedule
        } else {
            0_u64
        }
    }

    pub fn get_schedule_state(env: &Env, operation_id: BytesN<32>) -> OperationState {
        let ledger_time = env.ledger().timestamp();
        let lock_time = Self::get_schedule_lock_time(&env, operation_id);
        get_operation_state(ledger_time, lock_time)
    }

    pub fn has_role(env: &Env, account: Address, role: RoleLabel) -> bool {
        let key: RoleKey;
        match role {
            RoleLabel::Proposer => key = RoleKey::Proposers(account.clone()),
            RoleLabel::Executor => key = RoleKey::Executors(account.clone()),
            RoleLabel::Canceller => key = RoleKey::Cancellers(account.clone()),
        }
        has_role(env, &key)
    }

    fn _execute_check(env: &Env, operation_id: BytesN<32>, predecessor: Option<BytesN<32>>) {
        let ledger_time = env.ledger().timestamp();
        let lock_time = Self::get_schedule_lock_time(&env, operation_id);
        if get_operation_state(ledger_time, lock_time) != OperationState::Ready {
            panic_with_error!(&env, TimeLockError::TimeNotReady);
        }

        if let Some(predecessor) = predecessor {
            let pre_lock_time = Self::get_schedule_lock_time(&env, predecessor.clone());
            if get_operation_state(ledger_time, pre_lock_time) != OperationState::Executed {
                panic_with_error!(&env, TimeLockError::PredecessorNotDone);
            }
        }
    }

    fn _add_operation(env: &Env, operation_id: BytesN<32>, delay: u64) -> u64 {
        let lock_time = Self::get_schedule_lock_time(&env, operation_id.clone());
        let ledger_time = env.ledger().timestamp();
        if get_operation_state(ledger_time, lock_time) != OperationState::Unset {
            panic_with_error!(&env, TimeLockError::AlreadyExists);
        }
        let min_delay = env.storage().instance().get(&DataKey::MinDelay).unwrap();
        if delay < min_delay {
            panic_with_error!(&env, TimeLockError::InsufficientDelay);
        }

        let schedule = ledger_time + delay;
        env.storage()
            .instance()
            .set(&DataKey::Scheduler(operation_id.clone()), &schedule);
        schedule
    }
}
