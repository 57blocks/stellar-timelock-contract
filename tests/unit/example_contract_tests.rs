#![cfg(test)]

use soroban_sdk::testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Logs};
use soroban_sdk::{Address, Env, Symbol, IntoVal};
use time_lock_example_contract::{IncrementContract, IncrementContractClient};

extern crate std;

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IncrementContract);
    let client = IncrementContractClient::new(&env, &contract_id);

    assert_eq!(client.increment(&1), 1);
    assert_eq!(client.increment(&1), 2);
    assert_eq!(client.increment(&1), 3);

    std::println!("{}", env.logs().all().join("\n"));
}

#[test]
fn test_increment_only_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, IncrementContract);
    let client = IncrementContractClient::new(&env, &contract_id);

    let owner: Address = Address::generate(&env);
    client.initialize(&owner);

    client.increment_owner(&5);
    assert_eq!(
        env.auths(),
        std::vec![(
            owner.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "increment_owner"),
                    (5_u32,).into_val(&env)
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(client.get_count(), 5);
}
