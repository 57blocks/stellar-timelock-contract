require("dotenv").config();

const {
  nativeToScVal,
  Address,
  Keypair,
} = require("@stellar/stellar-sdk");

const { users, minDelay, sleep } = require("../common");
const {
  scheduleOperation,
  executeOperation,
  invokeContract,
} = require("./common");

async function scheduleAndExecuteRevokeRole(roleAccount,role) {
  let timeLockContractId = process.env.timelock_contract_id;
  let timeLockOwnerLabel = users.timelockOwner;
  let timeLockOwnerSecret = process.env[`${timeLockOwnerLabel}_secret`];
  let proposer = process.env[`${timeLockOwnerLabel}_pubkey`];
  let target = process.env.timelock_contract_id;
  let fnName = "revoke_role";
  let data = [nativeToScVal(roleAccount, { type: "address" }), nativeToScVal(role, { type: "u32" })];
  let currentTimeStamp = new Date().getTime() + "";
  let saltString = Buffer.from(currentTimeStamp)
    .toString("hex")
    .padStart(64, "0");
  console.log("saltString", saltString);
  let salt = Buffer.from(saltString, "hex");
  let delay = minDelay + 2;

  const scheduleParams = [
    nativeToScVal(Address.fromString(proposer), { type: "address" }),
    nativeToScVal(Address.fromString(target), { type: "address" }),
    nativeToScVal(fnName, { type: "symbol" }),
    nativeToScVal(nativeToScVal(data)),
    nativeToScVal(salt, { type: "bytes" }),
    nativeToScVal(null),
    nativeToScVal(delay, { type: "u64" }),
  ];

  const keyPair = Keypair.fromSecret(timeLockOwnerSecret);
  const {result} = await scheduleOperation(
    keyPair,
    timeLockContractId,
    scheduleParams
  );
  if (result) {
    await sleep(20000);
    const executeParams = scheduleParams.slice(0, scheduleParams.length - 1);
    let hasRole = await invokeContract(keyPair, timeLockContractId, "has_role", data);
    console.log(`before execute revoke role: ${hasRole}`);
    await executeOperation(keyPair, timeLockContractId, executeParams);
    hasRole = await invokeContract(keyPair, timeLockContractId, "has_role", data);
    console.log(`after execute revoke role: ${hasRole}`);
  }
}

async function updateRevokeRoleDirectly(roleAccount,role) {
  let timeLockOwnerLabel = users.timelockOwner;
  let timeLockOwnerSecret = process.env[`${timeLockOwnerLabel}_secret`];
  let timeLockContractId = process.env.timelock_self_managed_contract_id;
  let params = [nativeToScVal(roleAccount, { type: "address" }), nativeToScVal(role, { type: "u32" })];

  const keyPair = Keypair.fromSecret(timeLockOwnerSecret);

  let hasRole = await invokeContract(keyPair, timeLockContractId, "has_role", params);
  console.log(`before execute revoke role: ${hasRole}`);
  await invokeContract(keyPair, timeLockContractId, "revoke_role", params);
  hasRole = await invokeContract(keyPair, timeLockContractId, "has_role", params);
  console.log(`after execute revoke role: ${hasRole}`);
}

async function main(roleAccount,role) {
  await scheduleAndExecuteRevokeRole(roleAccount,role);
  await updateRevokeRoleDirectly(roleAccount,role);
}

module.exports = {
  revokeRoleFromTimeLock: main,
};
