require("dotenv").config();
const { Keypair } = require("@stellar/stellar-sdk");

const {
  scheduleOperation,
  executeOperation,
  cancelOperation,
  getOperationTimeLock,
  hasRole,
  grantRole,
  revokeRole,
  updateOwner,
  updateMinDelay,
} = require("./timelockHelper");
const { getTokenBalance, tokenMint, updateTokenAdmin } = require("./tokenHelper");

const { sleep, users, generateNewKeypair } = require("../common");

async function schedule_and_execute_token_mint() {
  let target = process.env.mock_token_contract_id;
  let fnName = "mint";
  let account = generateNewKeypair();
  let amount = 1000000000;
  let data = `[{"address":"${account}"},{"i128":[0,${amount}]}]`;
  let currentTimeStamp = new Date().getTime() + "";
  let salt = Buffer.from(currentTimeStamp).toString("hex").padStart(64, "0");
  let predecessor = null;
  let operationId = await scheduleOperation(
    target,
    fnName,
    data,
    salt,
    predecessor
  );
  await getOperationTimeLock(operationId);
  
  await sleep(25000);
  await executeOperation(target, fnName, data, salt, predecessor);

  await getOperationTimeLock(operationId);

  await getTokenBalance(account);
}

async function schedule_and_execute_token_mint_with_predecessor() {
    let target = process.env.mock_token_contract_id;
    let fnName = "mint";
    let account = generateNewKeypair();
    let amount = 1000000000;
    let data = `[{"address":"${account}"},{"i128":[0,${amount}]}]`;
    let currentTimeStamp = new Date().getTime() + "";
    let salt = Buffer.from(currentTimeStamp).toString("hex").padStart(64, "0");
    let predecessor = await scheduleOperation(
        target,
        fnName,
        data,
        salt,
        null
    );

    currentTimeStamp = new Date().getTime() + "";
    let salt2 = Buffer.from(currentTimeStamp).toString("hex").padStart(64, "0");
    let operationId = await scheduleOperation(
        target,
        fnName,
        data,
        salt2,
        predecessor
    );
    await getOperationTimeLock(operationId);
    
    await sleep(25000);
    await executeOperation(target, fnName, data, salt, null);
    await executeOperation(target, fnName, data, salt2, predecessor);
    
    await getOperationTimeLock(operationId);
    
    await getTokenBalance(account);
}

async function update_token_admin() {
    let target = process.env.mock_token_contract_id;
    let fnName = "set_admin";
    let newAdminLabel = users.deployer;
    let newAdmin = process.env[`${newAdminLabel}_pubkey`];
    let data = `[{"address":"${newAdmin}"}]`;
    let currentTimeStamp = new Date().getTime() + "";
    let salt = Buffer.from(currentTimeStamp).toString("hex").padStart(64, "0");
    let predecessor = null;
    let operationId = await scheduleOperation(
        target,
        fnName,
        data,
        salt,
        predecessor
    );
    await getOperationTimeLock(operationId);
    
    await sleep(25000);
    await executeOperation(target, fnName, data, salt, predecessor);
    
    await getOperationTimeLock(operationId);

    let account = generateNewKeypair();
    let amount = 1000000000;
    await tokenMint(newAdminLabel, account, amount);
    await getTokenBalance(account);

    await updateTokenAdmin(newAdminLabel, process.env.timelock_contract_id);
}

async function add_new_role_to_timelock(account, roleType){
    let target = process.env.timelock_contract_id;
    let fnName = "grant_role";
    let data = `[{"address":"${account}"},{"u32":${roleType}}]`;
    let currentTimeStamp = new Date().getTime() + "";
    let salt = Buffer.from(currentTimeStamp).toString("hex").padStart(64, "0");
    let predecessor = null;
    let operationId = await scheduleOperation(
        target,
        fnName,
        data,
        salt,
        predecessor
    );
    await getOperationTimeLock(operationId);
    
    await hasRole(account, roleType);
    await sleep(25000);
    await executeOperation(target, fnName, data, salt, predecessor);
    
    await getOperationTimeLock(operationId);
    await hasRole(account, roleType);
}

async function add_new_role_to_timelock_self_managed(account, roleType){

    await hasRole(account, roleType, process.env.timelock_self_managed_contract_id);

    await grantRole(account, roleType);

    await hasRole(account, roleType, process.env.timelock_self_managed_contract_id);
}

async function revoke_role_from_timelock(account, roleType) {
    let target = process.env.timelock_contract_id;
    let fnName = "revoke_role";
    let data = `[{"address":"${account}"},{"u32":${roleType}}]`;
    let currentTimeStamp = new Date().getTime() + "";
    let salt = Buffer.from(currentTimeStamp).toString("hex").padStart(64, "0");
    let predecessor = null;
    let operationId = await scheduleOperation(
        target,
        fnName,
        data,
        salt,
        predecessor
    );
    await getOperationTimeLock(operationId);
    
    await hasRole(account, roleType);
    await sleep(25000);
    await executeOperation(target, fnName, data, salt, predecessor);
    
    await getOperationTimeLock(operationId);
    await hasRole(account, roleType);

}

async function revoke_role_from_timelock_self_managed(account, roleType) {
    await hasRole(account, roleType, process.env.timelock_self_managed_contract_id);

    await revokeRole(account, roleType);

    await hasRole(account, roleType, process.env.timelock_self_managed_contract_id);
}

async function cancel_operation(){
    let target = process.env.mock_token_contract_id;
    let fnName = "mint";
    let account = generateNewKeypair();
    let amount = 1000000000;
    let data = `[{"address":"${account}"},{"i128":[0,${amount}]}]`;
    let currentTimeStamp = new Date().getTime() + "";
    let salt = Buffer.from(currentTimeStamp).toString("hex").padStart(64, "0");
    let predecessor = null;
    let operationId = await scheduleOperation(
        target,
        fnName,
        data,
        salt,
        predecessor
    );
    await getOperationTimeLock(operationId);

    await cancelOperation(operationId);

    await getOperationTimeLock(operationId);
}

async function update_timelock_owner() {
    let target = process.env.timelock_contract_id;
    let fnName = "update_owner";
    let newOwnerLabel = users.deployer;
    let newOwner = process.env[`${newOwnerLabel}_pubkey`];
    let data = `[{"address":"${newOwner}"}]`;
    let currentTimeStamp = new Date().getTime() + "";
    let salt = Buffer.from(currentTimeStamp).toString("hex").padStart(64, "0");
    let predecessor = null;
    let operationId = await scheduleOperation(
        target,
        fnName,
        data,
        salt,
        predecessor
    );
    await getOperationTimeLock(operationId);
    
    await sleep(25000);
    await executeOperation(target, fnName, data, salt, predecessor);
    
    await getOperationTimeLock(operationId);

    // rollback to original owner
    let originalOwnerLabel = users.timelockOwner;
    let originalOwner = process.env[`${originalOwnerLabel}_pubkey`];
    let data2 = `[{"address":"${originalOwner}"}]`;
    let currentTimeStamp2 = new Date().getTime() + "";
    let salt2 = Buffer.from(currentTimeStamp2).toString("hex").padStart(64, "0");
    let operationId2 = await scheduleOperation(
        target,
        fnName,
        data2,
        salt2,
        predecessor,
        users.deployer
    );
    await getOperationTimeLock(operationId2);

    await sleep(25000);
    await executeOperation(target, fnName, data2, salt2, predecessor,users.deployer);

    await getOperationTimeLock(operationId2);
}

async function update_timelock_owner_self_managed() {
    let newOwnerLabel = users.deployer;
    let newOwner = process.env[`${newOwnerLabel}_pubkey`];

    await updateOwner(newOwner);

    // rollback to original owner
    let originalOwnerLabel = users.timelockOwner;
    let originalOwner = process.env[`${originalOwnerLabel}_pubkey`];
    await updateOwner(originalOwner, newOwnerLabel);
}

async function update_time_lock_min_delay() {
    let target = process.env.timelock_contract_id;
    let fnName = "update_min_delay";
    let newDelay = 6;
    let data = `[{"u64":${newDelay}}]`;
    let currentTimeStamp = new Date().getTime() + "";
    let salt = Buffer.from(currentTimeStamp).toString("hex").padStart(64, "0");
    let predecessor = null;
    let operationId = await scheduleOperation(
        target,
        fnName,
        data,
        salt,
        predecessor
    );
    await getOperationTimeLock(operationId);
    
    await sleep(25000);
    await executeOperation(target, fnName, data, salt, predecessor);
    
    await getOperationTimeLock(operationId);
}

async function update_time_lock_min_delay_self_managed() {
    let newDelay = 6;

    await updateMinDelay(newDelay);
}

async function main() {
  console.log("Running schedule_and_execute_token_mint ...")
  await schedule_and_execute_token_mint();
  console.log("Running schedule_and_execute_token_mint_with_predecessor ...")
  await schedule_and_execute_token_mint_with_predecessor();
  console.log("Running update_token_admin ...")
  await update_token_admin();
  console.log("Running add_new_role_to_timelock ...")
  let roleType = 3;
  let account = generateNewKeypair();
  await add_new_role_to_timelock(account, roleType);
  console.log("Running add_new_role_to_timelock_self_managed ...")
  await add_new_role_to_timelock_self_managed(account, roleType);
  console.log("Running revoke_role_from_timelock ...")
  await revoke_role_from_timelock(account, roleType);
  console.log("Running revoke_role_from_timelock_self_managed ...")
  await revoke_role_from_timelock_self_managed(account,roleType);
  console.log("Running cancel_operation ...")
  await cancel_operation();
  console.log("Running update_timelock_owner ...")
  await update_timelock_owner();
  console.log("Running update_timelock_owner_self_managed ...")
  await update_timelock_owner_self_managed();
  console.log("Running update_time_lock_min_delay ...")
  await update_time_lock_min_delay();
  console.log("Running update_time_lock_min_delay_self_managed ...")
  await update_time_lock_min_delay_self_managed();
}

main()
  .then(() => {
    process.exit(0);
  })
  .catch((e) => {
    console.error(e);
    process.exit(1);
  });
