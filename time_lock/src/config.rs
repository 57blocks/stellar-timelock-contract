use soroban_sdk::{
    contracterror, contracttype, xdr::ToXdr, Address, Bytes, BytesN, Env, Symbol, Val, Vec,
};

pub(crate) const DONE_TIMESTAMP: u64 = 1;
pub(crate) const MAX_ACCOUNTS_NUM: u32 = 10;
pub(crate) const BATCH_MAX: u32 = 5;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Scheduler(BytesN<32>),
    MinDelay,
    IsInit,
}

#[derive(Clone)]
#[contracttype]
pub enum RoleKey {
    Admin,
    Proposers(Address),
    Cancellers(Address),
    Executors(Address),
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
pub enum OperationState {
    Unset = 1,
    Waiting = 2,
    Ready = 3,
    Executed = 4,
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
}

pub(crate) fn get_operation_state(ledger_time: u64, lock_time: u64) -> OperationState {
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

pub(crate) fn hash_call(
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

pub(crate) fn hash_call_batch(
    env: &Env,
    target: &Vec<Address>,
    fn_names: &Vec<Symbol>,
    datas: &Vec<Vec<Val>>,
    salt: &BytesN<32>,
    predecessor: &Option<BytesN<32>>,
) -> BytesN<32> {
    let mut calldata = Bytes::new(&env);
    calldata.append(&target.clone().to_xdr(&env));
    calldata.append(&fn_names.clone().to_xdr(&env));
    calldata.append(&datas.clone().to_xdr(&env));
    if let Some(predecessor) = predecessor {
        calldata.append(&predecessor.clone().to_xdr(&env));
    }
    calldata.append(&salt.clone().to_xdr(&env));
    env.crypto().sha256(&calldata)
}
