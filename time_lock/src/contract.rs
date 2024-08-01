/*
 * Contract module which acts as a timelocked controller. When set as the
 * owner of an `Ownable` smart contract, it enforces a timelock on all
 * `onlyOwner` maintenance operations. This gives time for users of the
 * controlled contract to exit before a potentially dangerous maintenance
 * operation is applied.
 *
 * this contract can also self administered, meaning administration tasks have to
 * go through the timelock process.
 */
use crate::role_base;
use crate::role_base::RoleLabel;
use crate::time_lock;
use crate::time_lock::{DataKey, TimeLockError};
use owner::owner;

use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, BytesN, Env, Symbol, Val, Vec,
};

#[contract]
pub struct TimeLockController;

#[contractimpl]
impl TimeLockController {
    /*
     *  Initializes the contract with the following parameters:
     *
     * - `min_delay`: initial minimum delay in seconds for operations
     * - `proposers`: accounts to be granted proposer and canceller roles
     * - `executors`: accounts to be granted executor role
     * - `owner`: account to be granted owner role, the owner can run management tasks directly,
     *  if None the contract will only be self managed.
     */
    pub fn initialize(
        e: Env,
        min_delay: u64,
        proposers: Vec<Address>,
        executors: Vec<Address>,
        owner: Option<Address>,
    ) {
        time_lock::initialize(&e, min_delay, &proposers, &executors, &owner)
    }

    /*
     * Schedule an operation containing a single transaction.
     *
     * Emits a {CallScheduled} event.
     *
     * Requirements:
     *
     * - the caller must have the 'proposer' role.
     * - if the target is the timelock itself, the caller must have the 'owner' role.
     */
    pub fn schedule(
        e: Env,
        proposer: Address,
        target: Address,
        fn_name: Symbol,
        data: Vec<Val>,
        salt: BytesN<32>,
        predecessor: Option<BytesN<32>>,
        delay: u64,
    ) -> BytesN<32> {
        if target == e.current_contract_address() {
            Self::_check_fn_name(&e, &fn_name);
        }

        Self::_check_role(&e, &proposer, &RoleLabel::Proposer);

        time_lock::schedule(&e, &target, &fn_name, &data, &salt, &predecessor, delay)
    }

    /*
     * Execute an (ready) operation containing a single transaction.
     *
     * Emits a {CallExecuted} event.
     *
     * Requirements:
     *
     * - the caller must have the 'executor' role.
     * - if the target is the timelock itself, the caller must have the 'owner' role.
     */
    pub fn execute(
        e: Env,
        executor: Address,
        target: Address,
        fn_name: Symbol,
        data: Vec<Val>,
        salt: BytesN<32>,
        predecessor: Option<BytesN<32>>,
    ) {
        let mut is_native = false;
        if target == e.current_contract_address() {
            is_native = true;
        } 

        Self::_check_role(&e, &executor, &RoleLabel::Executor);
        
        time_lock::execute(&e, &target, &fn_name, &data, &salt, &predecessor, is_native);
    }

    /*
     * Cancel an operation.
     *
     * Requirements:
     *
     * - the caller must have the 'canceller' role.
     */
    pub fn cancel(e: Env, canceller: Address, operation_id: BytesN<32>) {
        Self::_check_role(&e, &canceller, &RoleLabel::Canceller);

        time_lock::cancel(&e, &operation_id)
    }

    /*
     * Changes the minimum timelock duration for future operations.
     *
     * Emits a {MinDelayUpdated} event.
     *
     * Requirements:
     *
     * - if the timelock is self-managed, caller can direct the timelock to update the min delay. In this case,
     * the timelock will check that the caller is the owner. If the timelock is not self-managed, the caller must
     * first schedule an operation where the timelock is the target. then execute the operation.
     */
    pub fn update_min_delay(e: Env, delay: u64) {
        if delay > time_lock::MAX_MIN_DELAY {
            panic_with_error!(e, TimeLockError::DelayTooLong);
        }

        owner::only_owner(&e);
        time_lock::update_min_delay(&e, delay);
    }

    /*
     * Grants a role to an account.
     *
     * Requirements:
     *
     * - if the timelock is self-managed, caller can direct the timelock to grant a role. In this case,
     * the timelock will check that the caller is the owner. If the timelock is not self-managed, the caller must
     * first schedule an operation where the timelock is the target. then execute the operation.
     */
    pub fn grant_role(e: Env, account: Address, role: RoleLabel) -> bool {
        owner::only_owner(&e);
        role_base::grant_role(&e, &account, &role)
    }

    /*
     * Revokes a role from an account.
     *
     * Requirements:
     *
     * - if the timelock is self-managed, caller can direct the timelock to revoke a role. In this case,
     * the timelock will check that the caller is the owner. If the timelock is not self-managed, the caller must
     * first schedule an operation where the timelock is the target. then execute the operation.
     */
    pub fn revoke_role(e: Env, account: Address, role: RoleLabel) -> bool {
        owner::only_owner(&e);
        role_base::revoke_role(&e, &account, &role)
    }

    /*
     * Reset the owner account.
     *
     * Requirements:
     *
     * - if the timelock is self-managed, caller can direct the timelock to reset the owner. In this case,
     * the timelock will check that the caller is the owner. If the timelock is not self-managed, the caller must
     * first schedule an operation where the timelock is the target. then execute the operation.
     */
    pub fn update_owner(e: Env, owner: Address) {
        owner::only_owner(&e);
        owner::set_owner(&e, &owner)
    }

    /*
     * Returns the timestamp at which an operation becomes ready (0 for
     * unset operations, 1 for done operations).
     */
    pub fn get_schedule_lock_time(e: Env, operation_id: BytesN<32>) -> u64 {
        time_lock::get_schedule_lock_time(&e, &operation_id)
    }

    /*
     * Returns `true` if `account` has been granted `role`.
     */
    pub fn has_role(e: Env, account: Address, role: RoleLabel) -> bool {
        role_base::has_role(&e, &account, &role)
    }

    fn _check_role(e: &Env, account: &Address, role: &RoleLabel) {
        if !role_base::has_role(e, account, role) {
            panic_with_error!(e, TimeLockError::NotPermitted);
        }

        account.require_auth();
    }

    fn _check_fn_name(e: &Env, fn_name: &Symbol) {
        let fn_name = fn_name.clone();
        if fn_name == Symbol::new(e, "update_min_delay") {
            return;
        } else if fn_name == Symbol::new(e, "grant_role") {
            return;
        } else if fn_name == Symbol::new(e, "revoke_role") {
            return;
        } else if fn_name == Symbol::new(e, "update_owner") {
            return;
        } else {
            panic_with_error!(e, TimeLockError::InvalidFuncName);
        }
    }
}

#[cfg(any(test, feature = "testutils"))]
#[contractimpl]
impl TimeLockController {
    pub fn get_min_delay(e: &Env) -> u64 {
        e.storage().instance().get(&DataKey::MinDelay).unwrap_or(0)
    }

    pub fn is_owner(e: &Env, account: Address) -> bool {
        owner::get_owner(e).map_or(false, |owner| owner == account)
    }
}
