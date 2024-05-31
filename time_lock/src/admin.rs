use soroban_sdk::{contracterror, panic_with_error, Address, Env, Symbol};

use crate::config::RoleKey;

#[contracterror]
#[derive(Copy, Clone)]
#[repr(u32)]
enum AdminError {
    NotSet = 1,
}

pub fn has_admin(e: &Env) -> bool {
    let key = RoleKey::Admin;
    e.storage().instance().has(&key)
}

pub fn read_admin(e: &Env) -> Address {
    let key = RoleKey::Admin;
    if !has_admin(e) {
        panic_with_error!(&e, AdminError::NotSet);
    }
    e.storage().instance().get(&key).unwrap()
}

pub fn set_admin(e: &Env, id: &Address) {
    let key = RoleKey::Admin;
    e.storage().instance().set(&key, id);
    e.events().publish((Symbol::new(e,"AdminSet"),), *&id);
}
