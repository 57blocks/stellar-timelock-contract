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
     * - `admin`: account to be granted admin role
     * - `self_managed`: if true, the timelock will manage admin tasks directly; if false, these tasks
     *  will have to go through the timelock process.
     */
    pub fn initialize(
        env: Env,
        min_delay: u64,
        proposers: Vec<Address>,
        executors: Vec<Address>,
        admin: Address,
        self_managed: bool,
    ) {
        time_lock::initialize(
            &env,
            min_delay,
            &proposers,
            &executors,
            &admin,
            self_managed,
        )
    }

    /*
     * Schedule an operation containing a single transaction.
     *
     * Emits a {CallScheduled} event.
     *
     * Requirements:
     *
     * - the caller must have the 'proposer' role.
     * - if the target is the timelock itself, the caller must have the 'admin' role.
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
    ) -> BytesN<32> {
        if target == env.current_contract_address() {
            Self::_admin_check(&env);
        } else {
            Self::_role_check(&env, &proposer, RoleLabel::Proposer);
        }

        time_lock::schedule(&env, &target, &fn_name, &data, &salt, &predecessor, delay)
    }

    /*
     * Execute an (ready) operation containing a single transaction.
     *
     * Emits a {CallExecuted} event.
     *
     * Requirements:
     *
     * - the caller must have the 'executor' role.
     * - if the target is the timelock itself, the caller must have the 'admin' role.
     */
    pub fn execute(
        env: Env,
        executor: Address,
        target: Address,
        fn_name: Symbol,
        data: Vec<Val>,
        salt: BytesN<32>,
        predecessor: Option<BytesN<32>>,
    ) {
        let mut is_native = false;
        if target == env.current_contract_address() {
            Self::_admin_check(&env);
            is_native = true;
        } else {
            Self::_role_check(&env, &executor, RoleLabel::Executor);
        }
        time_lock::execute(
            &env,
            &target,
            &fn_name,
            &data,
            &salt,
            &predecessor,
            is_native,
        );
    }

    /*
     * Cancel an operation.
     *
     * Requirements:
     *
     * - the caller must have the 'canceller' role.
     */
    pub fn cancel(env: Env, canceller: Address, operation_id: BytesN<32>) {
        Self::_role_check(&env, &canceller, RoleLabel::Canceller);

        time_lock::cancel(&env, &operation_id)
    }

    /*
     * Changes the minimum timelock duration for future operations.
     *
     * Emits a {MinDelayUpdated} event.
     *
     * Requirements:
     *
     * - if the timelock is self-managed, caller can direct the timelock to update the min delay. In this case,
     * the timelock will check that the caller is the admin. If the timelock is not self-managed, the caller must
     * first schedule an operation where the timelock is the target. then execute the operation.
     */
    pub fn update_min_delay(env: Env, delay: u64) {
        if !time_lock::is_self_managed(&env) {
            panic_with_error!(env, TimeLockError::NotPermitted);
        }
        Self::_admin_check(&env);
        time_lock::update_min_delay(&env, delay);
    }

    /*
     * Grants a role to an account.
     *
     * Requirements:
     *
     * - if the timelock is self-managed, caller can direct the timelock to grant a role. In this case,
     * the timelock will check that the caller is the admin. If the timelock is not self-managed, the caller must
     * first schedule an operation where the timelock is the target. then execute the operation.
     */
    pub fn grant_role(env: Env, account: Address, role: RoleLabel) -> bool {
        if !time_lock::is_self_managed(&env) {
            panic_with_error!(env, TimeLockError::NotPermitted);
        }
        Self::_admin_check(&env);
        role_base::grant_role(&env, &account, &role)
    }

    /*
     * Revokes a role from an account.
     *
     * Requirements:
     *
     * - if the timelock is self-managed, caller can direct the timelock to revoke a role. In this case,
     * the timelock will check that the caller is the admin. If the timelock is not self-managed, the caller must
     * first schedule an operation where the timelock is the target. then execute the operation.
     */
    pub fn revoke_role(env: Env, account: Address, role: RoleLabel) -> bool {
        if !time_lock::is_self_managed(&env) {
            panic_with_error!(env, TimeLockError::NotPermitted);
        }
        Self::_admin_check(&env);
        role_base::revoke_role(&env, &account, &role)
    }

    /*
     * Reset the admin account.
     *
     * Requirements:
     *
     * - if the timelock is self-managed, caller can direct the timelock to reset the admin. In this case,
     * the timelock will check that the caller is the admin. If the timelock is not self-managed, the caller must
     * first schedule an operation where the timelock is the target. then execute the operation.
     */
    pub fn update_admin(env: Env, admin: Address) {
        if !time_lock::is_self_managed(&env) {
            panic_with_error!(env, TimeLockError::NotPermitted);
        }
        Self::_admin_check(&env);
        role_base::set_admin(&env, &admin)
    }

    /*
     * Returns the timestamp at which an operation becomes ready (0 for
     * unset operations, 1 for done operations).
     */
    pub fn get_schedule_lock_time(env: &Env, operation_id: BytesN<32>) -> u64 {
        time_lock::get_schedule_lock_time(&env, &operation_id)
    }

    /*
     * Returns `true` if `account` has been granted `role`.
     */
    pub fn has_role(env: &Env, account: Address, role: RoleLabel) -> bool {
        role_base::has_role(&env, &account, &role)
    }

    fn _admin_check(e: &Env) {
        let admin = role_base::read_admin(e);
        match admin {
            Some(admin) => {
                admin.require_auth();
            }
            None => panic_with_error!(e, TimeLockError::NotPermitted),
        }
    }

    fn _role_check(e: &Env, account: &Address, role: RoleLabel) {
        if !role_base::has_role(e, account, &role) {
            panic_with_error!(e, TimeLockError::NotPermitted);
        }

        account.require_auth();
    }
}

#[cfg(any(test, feature = "testutils"))]
#[contractimpl]
impl TimeLockController {
    pub fn get_min_delay(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::MinDelay)
            .unwrap_or(0)
    }

    pub fn is_admin(env: &Env, account: Address) -> bool {
        role_base::read_admin(env).map_or(false, |admin| admin == account)
    }
}
