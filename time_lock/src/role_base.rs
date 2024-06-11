use soroban_sdk::{contracttype, Address, Env, Symbol};

#[derive(Clone)]
#[contracttype]
pub enum RoleKey {
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

pub(crate) fn grant_role(e: &Env, account: &Address, role: &RoleLabel) -> bool {
    let key: RoleKey;
    match role {
        RoleLabel::Proposer => key = RoleKey::Proposers(account.clone()),
        RoleLabel::Executor => key = RoleKey::Executors(account.clone()),
        RoleLabel::Canceller => key = RoleKey::Cancellers(account.clone()),
    }

    let res = _set_role(e, &key);
    e.events().publish(
        (Symbol::new(e, "RoleGranted"), role.clone()),
        account.clone(),
    );

    res
}

pub(crate) fn revoke_role(e: &Env, account: &Address, role: &RoleLabel) -> bool {
    let key: RoleKey;
    match role {
        RoleLabel::Proposer => key = RoleKey::Proposers(account.clone()),
        RoleLabel::Executor => key = RoleKey::Executors(account.clone()),
        RoleLabel::Canceller => key = RoleKey::Cancellers(account.clone()),
    }
    let res = _unset_role(e, &key);
    e.events()
        .publish((Symbol::new(e, "RoleRevoked"), role.clone()), account);

    res
}

pub(crate) fn has_role(e: &Env, account: &Address, role: &RoleLabel) -> bool {
    let key: RoleKey;
    match role {
        RoleLabel::Proposer => key = RoleKey::Proposers(account.clone()),
        RoleLabel::Executor => key = RoleKey::Executors(account.clone()),
        RoleLabel::Canceller => key = RoleKey::Cancellers(account.clone()),
    }
    is_role(e, &key)
}

fn is_role(e: &Env, key: &RoleKey) -> bool {
    if let Some(_) = e.storage().persistent().get::<RoleKey, bool>(key) {
        return true;
    } else {
        return false;
    }
}

fn _set_role(e: &Env, key: &RoleKey) -> bool {
    if !is_role(e, key) {
        e.storage().persistent().set(key, &true);
        true
    } else {
        false
    }
}

fn _unset_role(e: &Env, key: &RoleKey) -> bool {
    if is_role(e, key) {
        e.storage().persistent().remove(key);
        true
    } else {
        false
    }
}
