require("dotenv").config();

const { cmd, networks, users, minDelay } = require("../common");

let timeLockContractId = process.env.timelock_contract_id;
let timeLockSelfManagedContractId = process.env.timelock_self_managed_contract_id;

async function getBalance(pubKey) {
  // const nativeId = await cmd(
  //   `soroban lab token wrap --asset native --network ${networks.name} --source ${users.deployer}`
  // );
  // console.log(`nativeId: ${nativeId}`);
  const nativeId = "CDMLFMKMMD7MWZP3FKUBZPVHTUEDLSX4BYGYKH4GCESXYHS3IHQ4EIG4";

  const balance = await cmd(
    `soroban contract invoke --id ${nativeId} --source ${users.deployer} --network ${networks.name} -- balance --id ${pubKey} `
  );
  console.log(`Balance: ${balance}`);
  return balance;
}

async function scheduleOperation(target, fnName, data, salt, predecessor) {
  let source = users.proposer;
  let proposer = `${source}_pubkey`;
  let command = `soroban contract invoke --id ${timeLockContractId} --source ${source} --network ${
    networks.name
  } -- schedule --proposer ${
    process.env[proposer]
  } --target ${target} --fn_name ${fnName} --data '${data}' --salt ${salt} --delay ${
    minDelay + 10
  } --predecessor ${predecessor}`;
  const mintOperationId = await cmd(command);
  console.log(`OperationId: ${mintOperationId}`);
  return mintOperationId;
}

async function executeOperation(target, fnName, data, salt, predecessor) {
  let source = users.executor;
  let executor = `${source}_pubkey`;
  const command = `soroban contract invoke --id ${timeLockContractId} --source ${source} --network ${networks.name} -- execute --executor ${process.env[executor]} --target ${target} --fn_name ${fnName} --data '${data}' --salt ${salt} --predecessor ${predecessor}`
  await cmd(
    command
  );
}

async function getOperationTimeLock(operationId) {
  const time = await cmd(
    `soroban contract invoke --id ${timeLockContractId} --source ${users.deployer} --network ${networks.name} -- get_schedule_lock_time --operation_id ${operationId}`
  );
  console.log(`operation ${operationId} lock time : ${time}`);
  return time;
}

async function cancelOperation(operationId) {
  let cancaller = `${users.proposer}_pubkey`;
  await cmd(
    `soroban contract invoke --id ${timeLockContractId} --source ${users.proposer} --network ${networks.name} -- cancel --canceller ${process.env[cancaller]} --operation_id ${operationId}`
  );
}

async function hasRole(pubKey, role, contractId = timeLockContractId) {
  const result = await cmd(
    `soroban contract invoke --id ${contractId} --source ${users.deployer} --network ${networks.name} -- has_role --account ${pubKey} --role ${role}`
  );
  console.log(`${pubKey} hasRole ${role} : ${result}`);
  return result;
}

async function grantRole(pubKey, role) {
  let result = await cmd(
    `soroban contract invoke --id ${timeLockSelfManagedContractId} --source ${users.timelockOwner} --network ${networks.name} -- grant_role --account ${pubKey} --role ${role}`
  );
  console.log(`grantRole ${role} to ${pubKey} : ${result}`);
  return result;
}

async function revokeRole(pubKey, role) {
  let result = await cmd(
    `soroban contract invoke --id ${timeLockSelfManagedContractId} --source ${users.timelockOwner} --network ${networks.name} -- revoke_role --account ${pubKey} --role ${role}`
  );
  console.log(`revokeRole ${role} from ${pubKey} : ${result}`);
  return result;
}

async function updateOwner(pubKey, timeLockOwner = users.timelockOwner) {
 await cmd(
    `soroban contract invoke --id ${timeLockSelfManagedContractId} --source ${timeLockOwner} --network ${networks.name} -- update_owner --owner ${pubKey}`
  );
  console.log(`updateOwner ${pubKey} done`);
}

async function updateMinDelay(newMinDelay) {
  await cmd(
    `soroban contract invoke --id ${timeLockSelfManagedContractId} --source ${users.timelockOwner} --network ${networks.name} -- update_min_delay --delay ${newMinDelay}`
  );
  console.log(`updateMinDelay ${newMinDelay} done`);
}

module.exports = {
  getBalance,
  scheduleOperation,
  executeOperation,
  getOperationTimeLock,
  cancelOperation,
  hasRole,
  grantRole,
  revokeRole,
  updateOwner,
  updateMinDelay,
};
