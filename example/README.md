# TimeLock Usage Example

Example cases to how to use time lock with other contract. We provider two types of code: soroban cli and soroban js SDK.

## Example Contract

The example contract [token](https://github.com/stellar/soroban-examples/tree/v20.0.0/token) is from soroban-examples contracts. You can find the source code in the repo. In our case, we use the token's builded wasm file.

In the token contract, there are two functions that only called by admin. We initialize the token contract by passing TimeLockController's instance as admin. So only the TimeLockController can invoke these two functions.

`
    pub fn mint(e: Env, to: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().mint(admin, to, amount);
    }

    pub fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_administrator(&e, &new_admin);
        TokenUtils::new(&e).events().set_admin(admin, new_admin);
    }   

`

## Main File & Directory

- /wasm
  store the token's optimized builded wasm file

- /cli
  organize these files related soroban cli

- /sdk
  organize these files related soroban js SDK

- docker.sh
  docker command to run the steller local node

- prepare.js
  script to deploy and initialize contract to local node

## Run Example Case

1. Make sure the TimeLockController contract in `../time_lock` are builded and optimized

2. Make sure the docker is running

3. Run `./docker.sh` to start the local node

4. Run `npm run prepare` to deploy and initialize the contracts

5. Run `npm run run-cli` to run these cases coding by soroban cli

6. Run `npm run run-sdk` to run these cases coding by JS SDK