use soroban_sdk::{contracttype, Address, Env, Symbol};

#[derive(Clone)]
#[contracttype]
pub enum RoleKey {
    Admin,
    Proposers(Address),
    Cancellers(Address),
    Executors(Address),
}

pub(crate) fn has_admin(e: &Env) -> bool {
    let key = RoleKey::Admin;
    e.storage().instance().has(&key)
}

pub(crate) fn read_admin(e: &Env) -> Option<Address> {
    let key = RoleKey::Admin;
    e.storage().instance().get(&key)
}

pub(crate) fn set_admin(e: &Env, id: &Address) {
    let key = RoleKey::Admin;
    e.storage().instance().set(&key, id);
    e.events().publish((Symbol::new(e, "AdminSet"),), *&id);
}

pub(crate) fn has_role(e: &Env, key: &RoleKey) -> bool {
    if let Some(_) = e.storage().persistent().get::<RoleKey, bool>(key) {
        return true;
    } else {
        return false;
    }
}

pub(crate) fn grant_role(e: &Env, key: &RoleKey) -> bool {
    if !has_role(e, key) {
        e.storage().persistent().set(key, &true);
        true
    } else {
        false
    }
}

pub(crate) fn revoke_role(e: &Env, key: &RoleKey) -> bool {
    if has_role(e, key) {
        e.storage().persistent().remove(key);
        true
    } else {
        false
    }
}
