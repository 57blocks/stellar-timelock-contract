use soroban_sdk::InvokeError;
use soroban_sdk::{
    contracterror, contracttype, panic_with_error, xdr::ToXdr, Address, Bytes, BytesN, Env, Symbol,
    TryFromVal, Val, Vec,
};

use core::primitive::u64;

use crate::role_base;
use crate::role_base::RoleLabel;
use owner::owner;

const DONE_TIMESTAMP: u64 = 1;
const MAX_ACCOUNTS_NUM: u32 = 10;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Scheduler(BytesN<32>),
    MinDelay,
    SelfManaged,
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
    ExecuteFailed = 10,
    InvalidFuncName = 11,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CallExecutedEvent {
    pub opt_id: BytesN<32>,
    pub target: Address,
    pub fn_name: Symbol,
    pub data: Vec<Val>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CallScheduledEvent {
    pub opt_id: BytesN<32>,
    pub target: Address,
    pub fn_name: Symbol,
    pub data: Vec<Val>,
    pub predecessor: BytesN<32>,
    pub delay: u64,
}

pub(crate) fn initialize(
    e: &Env,
    min_delay: u64,
    proposers: &Vec<Address>,
    executors: &Vec<Address>,
    owner: &Address,
    self_managed: bool,
) {
    if owner::has_owner(e) {
        panic_with_error!(e, TimeLockError::AlreadyInitialized);
    }

    if min_delay == 0 {
        panic_with_error!(e, TimeLockError::InvalidParams);
    }

    if proposers.len() == 0 || executors.len() == 0 {
        panic_with_error!(e, TimeLockError::InvalidParams);
    }

    if proposers.len() > MAX_ACCOUNTS_NUM || executors.len() > MAX_ACCOUNTS_NUM {
        panic_with_error!(e, TimeLockError::ExceedMaxCount);
    }

    update_min_delay(e, min_delay);

    _set_self_managed(e, self_managed);

    owner::set_owner(e, owner);

    for proposer in proposers.iter() {
        role_base::grant_role(e, &proposer, &RoleLabel::Proposer);
        role_base::grant_role(e, &proposer, &RoleLabel::Canceller);
    }

    for executor in executors.iter() {
        role_base::grant_role(e, &executor, &RoleLabel::Executor);
    }
}

pub(crate) fn schedule(
    e: &Env,
    target: &Address,
    fn_name: &Symbol,
    data: &Vec<Val>,
    salt: &BytesN<32>,
    predecessor: &Option<BytesN<32>>,
    delay: u64,
) -> BytesN<32> {
    if !_is_contract(e, target) {
        panic_with_error!(e, TimeLockError::InvalidParams);
    }

    let min_delay = e.storage().instance().get(&DataKey::MinDelay).unwrap();
    if delay < min_delay {
        panic_with_error!(e, TimeLockError::InsufficientDelay);
    }

    let operation_id = _hash_call(e, target, fn_name, data, salt, predecessor);
    _add_operation(e, &operation_id, delay);

    let actual_predecessor = match predecessor {
        Some(predecessor) => predecessor.clone(),
        None => BytesN::from_array(e, &[0_u8; 32]),
    };

    e.events().publish(
        (Symbol::new(e, "CallScheduled"),),
        CallScheduledEvent {
            opt_id: operation_id.clone(),
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
    e: &Env,
    target: &Address,
    fn_name: &Symbol,
    data: &Vec<Val>,
    salt: &BytesN<32>,
    predecessor: &Option<BytesN<32>>,
    is_native: bool,
) {
    let operation_id = _hash_call(e, target, fn_name, data, salt, predecessor);
    _check_execute(e, &operation_id, predecessor);

    if is_native {
        _exec_native(e, fn_name, data);
    } else {
        _exec_external(e, target, fn_name, data);
    }

    e.storage()
        .persistent()
        .set(&DataKey::Scheduler(operation_id.clone()), &DONE_TIMESTAMP);

    e.events().publish(
        (Symbol::new(e, "CallExecuted"),),
        CallExecutedEvent {
            opt_id: operation_id,
            target: target.clone(),
            fn_name: fn_name.clone(),
            data: data.clone(),
        },
    );
}

pub(crate) fn cancel(e: &Env, operation_id: &BytesN<32>) {
    let state = _get_operation_state(e, operation_id);
    if state == OperationState::Ready || state == OperationState::Waiting {
        e.storage()
            .persistent()
            .remove(&DataKey::Scheduler(operation_id.clone()));
    } else {
        panic_with_error!(e, TimeLockError::InvalidStatus);
    }

    e.events().publish(
        (Symbol::new(e, "OperationCancelled"),),
        operation_id.clone(),
    );
}

pub(crate) fn update_min_delay(e: &Env, delay: u64) {
    e.storage().instance().set(&DataKey::MinDelay, &delay);
    e.events()
        .publish((Symbol::new(e, "MinDelayUpdated"),), delay);
}

pub(crate) fn get_schedule_lock_time(e: &Env, operation_id: &BytesN<32>) -> u64 {
    let key = DataKey::Scheduler(operation_id.clone());
    if let Some(schedule) = e.storage().persistent().get::<DataKey, u64>(&key) {
        schedule
    } else {
        0_u64
    }
}

pub(crate) fn is_self_managed(e: &Env) -> bool {
    e.storage()
        .instance()
        .get(&DataKey::SelfManaged)
        .unwrap_or(false)
}

fn _set_self_managed(e: &Env, self_managed: bool) {
    e.storage()
        .instance()
        .set(&DataKey::SelfManaged, &self_managed);
    e.events()
        .publish((Symbol::new(e, "SelfManaged"),), self_managed);
}

fn _get_operation_state(e: &Env, operation_id: &BytesN<32>) -> OperationState {
    let ledger_time = e.ledger().timestamp();
    let lock_time = get_schedule_lock_time(e, operation_id);
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

fn _add_operation(e: &Env, operation_id: &BytesN<32>, delay: u64) {
    let ledger_time = e.ledger().timestamp();
    if _get_operation_state(e, operation_id) != OperationState::Unset {
        panic_with_error!(e, TimeLockError::AlreadyExists);
    }

    let time = ledger_time + delay;
    e.storage()
        .persistent()
        .set(&DataKey::Scheduler(operation_id.clone()), &time);
}

fn _check_execute(e: &Env, operation_id: &BytesN<32>, predecessor: &Option<BytesN<32>>) {
    if _get_operation_state(e, operation_id) != OperationState::Ready {
        panic_with_error!(e, TimeLockError::TimeNotReady);
    }

    if let Some(predecessor) = predecessor {
        if _get_operation_state(e, predecessor) != OperationState::Executed {
            panic_with_error!(e, TimeLockError::PredecessorNotDone);
        }
    }
}

fn _exec_external(e: &Env, target: &Address, fn_name: &Symbol, data: &Vec<Val>) {
    let result = e.try_invoke_contract::<(), InvokeError>(&target, &fn_name, data.clone());

    match result {
        Ok(_) => {}
        Err(_) => {
            panic_with_error!(e, TimeLockError::ExecuteFailed);
        }
    }
}

fn _exec_native(e: &Env, fn_name: &Symbol, data: &Vec<Val>) {
    let fn_name = fn_name.clone();
    if fn_name == Symbol::new(e, "update_min_delay") {
        _update_min_delay(e, data);
    } else if fn_name == Symbol::new(e, "grant_role") {
        _update_role(e, data, true);
    } else if fn_name == Symbol::new(e, "revoke_role") {
        _update_role(e, data, false);
    } else if fn_name == Symbol::new(e, "update_owner") {
        _update_owner(e, data);
    } else {
        panic_with_error!(e, TimeLockError::InvalidFuncName);
    }
}

fn _hash_call(
    e: &Env,
    target: &Address,
    fn_name: &Symbol,
    data: &Vec<Val>,
    salt: &BytesN<32>,
    predecessor: &Option<BytesN<32>>,
) -> BytesN<32> {
    let mut calldata = Bytes::new(e);
    calldata.append(&target.clone().to_xdr(e));
    calldata.append(&fn_name.clone().to_xdr(e));
    calldata.append(&data.clone().to_xdr(e));
    if let Some(predecessor) = predecessor {
        calldata.append(&predecessor.clone().to_xdr(e));
    }
    calldata.append(&salt.clone().to_xdr(e));
    e.crypto().sha256(&calldata)
}

fn _is_contract(env: &Env, address: &Address) -> bool {
    let address_ = address.to_string().to_xdr(env);
    let first_char_index = address_.get(8).unwrap();
    if first_char_index == 67_u8 {
        return true;
    }
    false
}

fn _update_min_delay(e: &Env, data: &Vec<Val>) {
    let delay = data.get(0);
    if let Some(delay) = delay {
        let p = u64::try_from_val(e, &delay);
        if let Ok(delay) = p {
            update_min_delay(e, delay);
        } else {
            panic_with_error!(e, TimeLockError::InvalidParams);
        }
    } else {
        panic_with_error!(e, TimeLockError::InvalidParams);
    }
}

fn _update_role(e: &Env, data: &Vec<Val>, is_grand: bool) {
    let account = data.get(0);
    let role = data.get(1);
    if let Some(account) = account {
        let p = Address::try_from_val(e, &account);
        if let Ok(account) = p {
            if let Some(role) = role {
                let p = RoleLabel::try_from_val(e, &role);
                if let Ok(role) = p {
                    if is_grand {
                        role_base::grant_role(e, &account, &role);
                    } else {
                        role_base::revoke_role(e, &account, &role);
                    }
                } else {
                    panic_with_error!(e, TimeLockError::InvalidParams);
                }
            } else {
                panic_with_error!(e, TimeLockError::InvalidParams);
            }
        } else {
            panic_with_error!(e, TimeLockError::InvalidParams);
        }
    } else {
        panic_with_error!(e, TimeLockError::InvalidParams);
    }
}

fn _update_owner(e: &Env, data: &Vec<Val>) {
    let owner = data.get(0);
    if let Some(owner) = owner {
        let p = Address::try_from_val(e, &owner);
        if let Ok(owner) = p {
            owner::set_owner(e, &owner);
        } else {
            panic_with_error!(e, TimeLockError::InvalidParams);
        }
    } else {
        panic_with_error!(e, TimeLockError::InvalidParams);
    }
}
