use soroban_sdk::Env;

use crate::config::RoleKey;

pub(crate) fn has_role(e: &Env, key: &RoleKey) -> bool {
    if let Some(_) = e.storage().instance().get::<RoleKey, bool>(key) {
        return true;
    } else {
        return false;
    }
}

pub(crate) fn grant_role(e: &Env, key: &RoleKey) -> bool {
    if !has_role(e, key) {
        e.storage()
            .instance()
            .set(key, &true);
        true
    } else {
        false
    }
}

pub(crate) fn revoke_role(e: &Env, key: &RoleKey) -> bool {
    if has_role(e, key) {
        e.storage()
            .instance()
            .remove(key);
        true
    } else {
        false
    }
}