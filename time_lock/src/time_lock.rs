use soroban_sdk::InvokeError;
use soroban_sdk::{
    contracterror, contracttype, panic_with_error, vec, xdr::ToXdr, Address, Bytes, BytesN, Env,
    IntoVal, Symbol, Val, Vec,
};

use crate::access_base;
use crate::access_base::RoleKey;

const DONE_TIMESTAMP: u64 = 1;
const MAX_ACCOUNTS_NUM: u32 = 10;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Scheduler(BytesN<32>),
    MinDelay,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[contracttype]
#[repr(u8)]
pub enum RoleLabel {
    Proposer = 1,
    Executor = 2,
    Canceller = 3,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[contracttype]
#[repr(u8)]
enum OperationState {
    Unset = 1,
    Waiting = 2,
    Ready = 3,
    Executed = 4,
}

#[derive(Copy, Clone)]
#[contracterror]
#[repr(u32)]
pub enum TimeLockError {
    InvalidParams = 0,
    NotInitialized = 1,
    AlreadyInitialized = 2,
    AlreadyExists = 3,
    InsufficientDelay = 4,
    TimeNotReady = 5,
    PredecessorNotDone = 6,
    ExceedMaxCount = 7,
    InvalidStatus = 8,
    NotPermitted = 9,
    AdminNotSet = 10,
    ExecuteFailed = 11,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CallExecutedEvent {
    pub opt_id: BytesN<32>,
    pub index: u32,
    pub target: Address,
    pub fn_name: Symbol,
    pub data: Vec<Val>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CallScheduledEvent {
    pub opt_id: BytesN<32>,
    pub index: u32,
    pub target: Address,
    pub fn_name: Symbol,
    pub data: Vec<Val>,
    pub predecessor: BytesN<32>,
    pub delay: u64,
}

pub(crate) fn initialize(
    env: &Env,
    min_delay: u64,
    proposers: &Vec<Address>,
    executors: &Vec<Address>,
    admin: &Address,
) {
    if access_base::has_admin(env) {
        panic_with_error!(env, TimeLockError::AlreadyInitialized);
    }

    if min_delay == 0 {
        panic_with_error!(env, TimeLockError::InvalidParams);
    }

    if proposers.len() == 0 || executors.len() == 0 {
        panic_with_error!(env, TimeLockError::InvalidParams);
    }

    if proposers.len() > MAX_ACCOUNTS_NUM || executors.len() > MAX_ACCOUNTS_NUM {
        panic_with_error!(env, TimeLockError::ExceedMaxCount);
    }

    env.storage().instance().set(&DataKey::MinDelay, &min_delay);
    access_base::set_admin(env, admin);

    for proposer in proposers.iter() {
        let p_role = RoleKey::Proposers(proposer.clone());
        let c_role = RoleKey::Cancellers(proposer.clone());
        access_base::grant_role(env, &p_role);
        access_base::grant_role(env, &c_role);
        env.events()
            .publish((Symbol::new(env, "RoleGranted"), p_role), proposer.clone());
        env.events()
            .publish((Symbol::new(env, "RoleGranted"), c_role), proposer.clone());
    }

    for executor in executors.iter() {
        let e_role = RoleKey::Executors(executor.clone());
        access_base::grant_role(env, &e_role);
        env.events()
            .publish((Symbol::new(env, "RoleGranted"), e_role), executor.clone());
    }
}

pub(crate) fn schedule(
    env: &Env,
    proposer: &Address,
    target: &Address,
    fn_name: &Symbol,
    data: &Vec<Val>,
    salt: &BytesN<32>,
    predecessor: &Option<BytesN<32>>,
    delay: u64,
) -> BytesN<32> {
    if !_is_contract(env, target) {
        panic_with_error!(&env, TimeLockError::InvalidParams);
    }

    proposer.require_auth();
    if !has_role(&env, proposer, &RoleLabel::Proposer) {
        panic_with_error!(&env, TimeLockError::NotPermitted);
    }

    let operation_id = _hash_call(env, &target, &fn_name, &data, &salt, &predecessor);
    _add_operation(env, &operation_id, delay);

    let actual_predecessor = match predecessor {
        Some(predecessor) => predecessor.clone(),
        None => BytesN::from_array(env, &[0_u8; 32]),
    };

    env.events().publish(
        (Symbol::new(&env, "CallScheduled"),),
        CallScheduledEvent {
            opt_id: operation_id.clone(),
            index: 0,
            target: target.clone(),
            fn_name: fn_name.clone(),
            data: data.clone(),
            predecessor: actual_predecessor,
            delay,
        },
    );

    operation_id
}

pub(crate) fn execute(
    env: &Env,
    executor: &Address,
    target: &Address,
    fn_name: &Symbol,
    data: &Vec<Val>,
    salt: &BytesN<32>,
    predecessor: &Option<BytesN<32>>,
) {
    executor.require_auth();
    if !has_role(&env, executor, &RoleLabel::Executor) {
        panic_with_error!(&env, TimeLockError::NotPermitted);
    }

    let operation_id = _hash_call(env, &target, &fn_name, &data, &salt, &predecessor);
    _execute_check(&env, &operation_id, predecessor);

    let result = env.try_invoke_contract::<(), InvokeError>(&target, &fn_name, data.clone());

    match result {
        Ok(_) => {}
        Err(_) => {
            panic_with_error!(&env, TimeLockError::ExecuteFailed);
        }
    }

    // Update the state of the operation to executed
    env.storage()
        .persistent()
        .set(&DataKey::Scheduler(operation_id.clone()), &DONE_TIMESTAMP);

    env.events().publish(
        (Symbol::new(&env, "CallExecuted"),),
        CallExecutedEvent {
            opt_id: operation_id,
            index: 0,
            target: target.clone(),
            fn_name: fn_name.clone(),
            data: data.clone(),
        },
    );
}

pub(crate) fn cancel(env: &Env, canceller: &Address, operation_id: &BytesN<32>) {
    canceller.require_auth();

    if !has_role(&env, canceller, &RoleLabel::Canceller) {
        panic_with_error!(&env, TimeLockError::NotPermitted);
    }

    let ledger_time = env.ledger().timestamp();
    let lock_time = get_schedule_lock_time(env, &operation_id);
    let state = _get_operation_state(ledger_time, lock_time);
    if state == OperationState::Ready || state == OperationState::Waiting {
        env.storage()
            .persistent()
            .remove(&DataKey::Scheduler(operation_id.clone()));
    } else {
        panic_with_error!(&env, TimeLockError::InvalidStatus);
    }

    env.events().publish(
        (Symbol::new(&env, "OperationCancelled"),),
        operation_id.clone(),
    );
}

pub(crate) fn update_min_delay(env: &Env, delay: u64, salt: &BytesN<32>) {
    let operation_id = _hash_call(
        env,
        &env.current_contract_address(),
        &Symbol::new(&env, "update_min_delay"),
        &vec![env, delay.into_val(env), salt.into_val(env)],
        &salt,
        &None,
    );
    let ledger_time = env.ledger().timestamp();
    let lock_time = get_schedule_lock_time(env, &operation_id);
    if _get_operation_state(ledger_time, lock_time) != OperationState::Ready {
        panic_with_error!(env, TimeLockError::TimeNotReady);
    }

    env.storage().instance().set(&DataKey::MinDelay, &delay);

    env.storage()
        .persistent()
        .set(&DataKey::Scheduler(operation_id), &DONE_TIMESTAMP);

    env.events()
        .publish((Symbol::new(env, "MinDelayUpdated"),), delay);
}

pub(crate) fn grant_role(env: &Env, account: &Address, role: &RoleLabel) -> bool {
    let admin = access_base::read_admin(env);
    match admin {
        Some(admin) => {
            admin.require_auth();
        }
        None => panic_with_error!(env, TimeLockError::AdminNotSet),
    }

    let key: RoleKey;
    match role {
        RoleLabel::Proposer => key = RoleKey::Proposers(account.clone()),
        RoleLabel::Executor => key = RoleKey::Executors(account.clone()),
        RoleLabel::Canceller => key = RoleKey::Cancellers(account.clone()),
    }

    let res = access_base::grant_role(env, &key);
    env.events()
        .publish((Symbol::new(&env, "RoleGranted"), role.clone()), account);

    res
}

pub(crate) fn revoke_role(env: &Env, account: &Address, role: &RoleLabel) -> bool {
    let admin = access_base::read_admin(env);
    match admin {
        Some(admin) => {
            admin.require_auth();
        }
        None => panic_with_error!(env, TimeLockError::AdminNotSet),
    }

    let key: RoleKey;
    match role {
        RoleLabel::Proposer => key = RoleKey::Proposers(account.clone()),
        RoleLabel::Executor => key = RoleKey::Executors(account.clone()),
        RoleLabel::Canceller => key = RoleKey::Cancellers(account.clone()),
    }
    let res = access_base::revoke_role(env, &key);
    env.events()
        .publish((Symbol::new(&env, "RoleRevoked"), role.clone()), account);

    res
}

pub(crate) fn get_schedule_lock_time(env: &Env, operation_id: &BytesN<32>) -> u64 {
    let key = DataKey::Scheduler(operation_id.clone());
    if let Some(schedule) = env.storage().persistent().get::<DataKey, u64>(&key) {
        schedule
    } else {
        0_u64
    }
}

pub(crate) fn has_role(env: &Env, account: &Address, role: &RoleLabel) -> bool {
    let key: RoleKey;
    match role {
        RoleLabel::Proposer => key = RoleKey::Proposers(account.clone()),
        RoleLabel::Executor => key = RoleKey::Executors(account.clone()),
        RoleLabel::Canceller => key = RoleKey::Cancellers(account.clone()),
    }
    access_base::has_role(env, &key)
}

fn _get_operation_state(ledger_time: u64, lock_time: u64) -> OperationState {
    if lock_time == 0 {
        OperationState::Unset
    } else if lock_time == DONE_TIMESTAMP {
        OperationState::Executed
    } else if ledger_time < lock_time {
        OperationState::Waiting
    } else {
        OperationState::Ready
    }
}

fn _add_operation(env: &Env, operation_id: &BytesN<32>, delay: u64) {
    let lock_time = get_schedule_lock_time(&env, operation_id);
    let ledger_time = env.ledger().timestamp();
    if _get_operation_state(ledger_time, lock_time) != OperationState::Unset {
        panic_with_error!(&env, TimeLockError::AlreadyExists);
    }
    let min_delay = env.storage().instance().get(&DataKey::MinDelay).unwrap();
    if delay < min_delay {
        panic_with_error!(&env, TimeLockError::InsufficientDelay);
    }

    let schedule = ledger_time + delay;
    env.storage()
        .persistent()
        .set(&DataKey::Scheduler(operation_id.clone()), &schedule);
}

fn _execute_check(env: &Env, operation_id: &BytesN<32>, predecessor: &Option<BytesN<32>>) {
    let ledger_time = env.ledger().timestamp();
    let lock_time = get_schedule_lock_time(env, operation_id);
    if _get_operation_state(ledger_time, lock_time) != OperationState::Ready {
        panic_with_error!(env, TimeLockError::TimeNotReady);
    }

    if let Some(predecessor) = predecessor {
        let pre_lock_time = get_schedule_lock_time(&env, predecessor);
        if _get_operation_state(ledger_time, pre_lock_time) != OperationState::Executed {
            panic_with_error!(&env, TimeLockError::PredecessorNotDone);
        }
    }
}

fn _hash_call(
    env: &Env,
    target: &Address,
    fn_name: &Symbol,
    data: &Vec<Val>,
    salt: &BytesN<32>,
    predecessor: &Option<BytesN<32>>,
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

fn _is_contract(env: &Env, address: &Address) -> bool {
    let address_ = address.to_string().to_xdr(env);
    let first_char_index = address_.get(8).unwrap();
    if first_char_index == 67_u8 {
        return true;
    }
    false
}
